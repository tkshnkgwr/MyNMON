# システム構成図 (DIAGRAM.md)

[English](../en/DIAGRAM.md) | **日本語版**

本ドキュメントは、`MyNMON` のスレッド、ライフサイクル、データ取得経路、およびイベント制御フローをダイアグラムで示したものです。

---

## 1. アプリケーションのライフサイクルとイベントループ

`MyNMON` は、crossterm のイベントポーリングと経過時間算出を組み合わせることで、低レイテンシーなキー応答と正確な定期メトリクス更新を単一スレッドで両立しています。

```mermaid
sequenceDiagram
    autonumber
    actor User as ユーザー
    participant App as MyNMON (Main)
    participant Common as common_lib
    participant Term as ターミナル (crossterm)
    participant System as OS / ハードウェア (sysinfo)

    Note over App: 起動処理 (fn main)
    App->>Common: 二重起動チェック (check_single_instance)
    App->>System: メトリクス初期化 (System / Disks / Networks)
    App->>Term: TerminalGuard 生成 (rawモード・代替画面・カーソル非表示)
    App->>System: 初回リフレッシュ & 短い待機

    loop メインループ
        opt 初回 または tick_rate 経過時
            App->>System: メトリクス更新 (CPU, RAM, Disks, Networks, Processes)
            System-->>App: 最新メトリクスデータ
            App->>Common: プロセス差分計算 (compute_diff)
            Common-->>App: 起動/終了ログ更新
        end

        App->>Term: UIを描画 (draw_ui)
        Term-->>User: 画面表示更新

        Note over App: キーポーリング待機時間算出<br/>(tick_rate - 前回更新からの経過時間)

        alt 待機時間内にキー入力あり (KeyEventKind::Press)
            User->>Term: キーを押す
            Term->>App: イベント検出 (event::poll)
            alt 'f' キー (プロセス検索フィルタ)
                App->>App: フィルタ入力モード (is_filtering = true)
            else 'r' キー (リフレッシュ間隔変更)
                App->>App: 間隔入力モード (is_setting_interval = true)
            else その他のトグルキー (C, c, m, d, n, p/t, g/l)
                App->>App: 表示状態 (MonitorState) をトグル更新
            end
        else 待機時間超過 (タイムアウト)
            Note over App: 次ループでメトリクス更新を実行
        end

        alt 'q' または 'Esc' が押された場合 (通常モード)
            App->>Term: TerminalGuard::drop (rawモード解除・通常画面復帰・カーソル再表示)
            Note over App: プロセス正常終了 (exit 0)
            App->>User: ターミナル制御を返却
        end
    end
```

---

## 2. データの取得と描画フロー

`sysinfo` を介して OS から収集したシステムデータおよび `common_lib` の解析結果が、どのように処理されてターミナルに描画されるかのデータ経路です。

```mermaid
graph TD
    subgraph "OS (Kernel Space)"
        ProcFS["Linux /proc"]
        WinAPI["Windows API"]
        macOS["macOS sysctl"]
    end

    subgraph "sysinfo Crate (Data Collection)"
        SysInst["System (CPU, RAM, Processes)"]
        DiskInst["Disks (Mounts, Space)"]
        NetInst["Networks (Rx/Tx bytes)"]
    end

    subgraph "common_lib Crate (Shared Core)"
        Mutex["check_single_instance (Named Mutex)"]
        Diff["compute_diff (LCS Process Diff)"]
        Count["count_occurrences (Filter Match Count)"]
    end

    subgraph "MyNMON Modules"
        State["state::MonitorState (toggles, filter, tick_rate)"]
        Draw["ui::draw_ui (UI Rendering Engine)"]
        Utils["utils (get_ascii_bar, pad_or_truncate, format_uptime)"]
    end

    subgraph "crossterm Crate (Rendering)"
        TermBuf["Terminal Alternate Buffer"]
    end

    %% データフロー接続
    ProcFS --> SysInst
    WinAPI --> SysInst
    macOS --> SysInst
    ProcFS --> DiskInst
    WinAPI --> DiskInst
    ProcFS --> NetInst
    WinAPI --> NetInst

    SysInst -->|Process List| Diff
    Diff -->|Spawn/Exit Log| State
    SysInst -->|Read Metrics| Draw
    DiskInst -->|Read Disk| Draw
    NetInst -->|Read I/O| Draw
    Count -->|Match Count| Draw
    State -->|Conditional Toggle| Draw
    Utils -->|Format & Bars| Draw
    Mutex -.->|Startup Guard| State

    Draw -->|crossterm::execute!| TermBuf
```
