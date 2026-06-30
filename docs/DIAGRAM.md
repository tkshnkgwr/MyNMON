# システム構成図 (DIAGRAM.md)

本ドキュメントは、`rust-nmon` のスレッド、ライフサイクル、データ取得経路、およびイベント制御フローをダイアグラムで示したものです。

---

## 1. アプリケーションのライフサイクルとイベントループ

`rust-nmon` は、crosstermの非同期ポーリングとタイマー時間算出を組み合わせることで、低レイテンシーな描画とキーイベント応答を単一スレッドで両立しています。

```mermaid
sequenceDiagram
    autonumber
    actor User as ユーザー
    participant App as rust-nmon (Main)
    participant Term as ターミナル (crossterm)
    participant System as OS / ハードウェア (sysinfo)

    Note over App: 起動処理 (fn main)
    App->>Term: 生モード有効化 & オルタネート画面切り替え & カーソル非表示
    App->>System: メトリクスの初期化・初回リフレッシュ

    loop 1秒間隔のメインループ
        App->>System: 最新のメトリクス情報をリフレッシュ (CPU, RAM, Disks, Networks, Processes)
        System-->>App: メトリクスデータ
        App->>Term: UIを描画 (draw_ui)
        Term-->>User: 画面表示更新

        Note over App: キーポーリングの待機時間算出<br/>(1秒 - 前回のループ経過時間)
        
        alt 待機時間内にキー入力あり
            User->>Term: キーを押す (c, m, d, n, p, t 等)
            Term->>App: イベント検出 (event::poll)
            App->>App: 表示状態 (MonitorState) をトグル更新
        else 待機時間超過 (タイムアウト)
            Note over App: 何もせず次のループへ進む
        end
        
        alt 'q' または 'Esc' が押された場合
            App->>Term: 生モード解除 & 通常画面へ復帰 & カーソル再表示
            Note over App: プロセス正常終了
            App->>User: ターミナル制御を返却
        end
    end
```

---

## 2. データの取得と描画フロー

`sysinfo` を介してOSから収集したシステムデータが、どのように処理されてターミナルに描画されるかのデータ経路です。

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

    subgraph "rust-nmon Logic"
        State["MonitorState (show_cpu, show_mem, etc.)"]
        Draw["draw_ui Function"]
        AscBar["get_ascii_bar (ASCII Bar Engine)"]
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

    SysInst -->|Read Metrics| Draw
    DiskInst -->|Read Disk| Draw
    NetInst -->|Read I/O| Draw
    State -->|Conditional Toggle| Draw
    AscBar -->|Generate [===> ]| Draw

    Draw -->|crossterm::execute!| TermBuf
```
