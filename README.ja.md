# MyNMON

[English (英語版)](README.md)

![CI Status](https://github.com/tkshnkgwr/MyNMON/actions/workflows/ci.yml/badge.svg)
![Latest Release](https://img.shields.io/github/v/release/tkshnkgwr/MyNMON)
![Rust Version](https://img.shields.io/badge/rust-1.96.0%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

`MyNMON` は、古典的なシステム監視ツール「nmon」にインスパイアされた、Rust製のクロスプラットフォーム対応・超軽量CUIシステムモニターです。`sysinfo` クレートを用いて各種メトリクスを取得し、`crossterm` クレートを用いてターミナル上にリアルタイムにシステム状態を描画します。

## 主な機能

- **マルチセクション表示**: 必要なセクションだけを動的に表示/非表示できます。
- **CPU 使用率**: コアごとの使用率パーセンテージと、視覚的なアスキーバーを表示します。
- **メモリ割り当て**: 物理メモリ（RAM）の総量、使用量、空き容量をリアルタイム表示します。
- **ディスクマウントと容量**: マウントポイント、ファイルシステムの種類、空き容量を監視します。
- **ネットワーク I/O 速度**: 有効なインターフェースごとの受信（Rx）/送信（Tx）速度を KB/s 単位で追跡します。
- **プロセス監視**: CPU 使用率の高い順に上位プロセスを一覧表示します。
- **プロセス検索フィルタ**: プロセス名によるリアルタイム絞り込み検索（マッチ数カウント表示付き）が行えます。
- **プロセス変化の履歴ログ**: 起動されたプロセス（`+`）や終了したプロセス（`-`）のリアルタイム差分ログを表示します。
- **多重起動防止 (Windows)**: Named Mutex による Windows 上での二重起動を防止し、画面描画の競合を防ぎます。
- **インタラクティブな制御**: キーボードショートカットで表示セクションを即座に切り替え可能です。
- **徹底した最適化**: リリースバイナリサイズは約 308 KB、実行時のメモリ使用量は約 20.8 MB と極めて軽量です。

## キーボードショートカット

アプリケーションの実行中に以下のキーを押すことで、表示セクションの切り替えや終了を行えます。

- `c` : CPU使用率セクションの表示/非表示を切り替え
- `m` : メモリ割り当てセクションの表示/非表示を切り替え
- `d` : ディスク容量・マウントセクションの表示/非表示を切り替え
- `n` : ネットワークインターフェース速度セクションの表示/非表示を切り替え
- `p` または `t` : プロセス一覧セクションの表示/非表示を切り替え
- `g` または `l` : プロセス起動・終了履歴ログセクションの表示/非表示を切り替え
- `f` : プロセス名の検索・フィルタ入力モードを開始（`Enter` または `Esc` で検索を終了して通常モードへ復帰）
- `q` または `Esc` : アプリケーションを終了（検索モード中以外）

## コマンドライン引数

`MyNMON` は起動時に以下のコマンドライン引数を指定できます。

- `-h`, `--help` : ヘルプメッセージ（使用方法とオプション一覧）を表示して終了します。
- `-v`, `--version` : `Cargo.toml` から解決された動的なバージョン情報を表示して終了します。

使用例：
```bash
./MyNMON --help
./MyNMON --version
```

## クイックスタート

### 前提条件

1. Rust および Cargo がインストールされていることを確認してください。(Rust 1.96.0 以上を推奨)
2. 本プロジェクトは、相対パス (`../common_lib`) を通じて共有ライブラリ `common_lib` に依存しています。同一の親ディレクトリ内に両方のリポジトリをクローンする必要があります。

```bash
# 共有ライブラリをクローン
git clone https://github.com/tkshnkgwr/common_lib.git

# メインプロジェクト（MyNMON）をクローン
git clone https://github.com/tkshnkgwr/MyNMON.git
```

以下のようなディレクトリ構成にする必要があります：
```text
parent_directory/
├── common_lib/
└── MyNMON/
```

### ビルドと実行

リポジトリをクローンまたはコピーし、プロジェクトのルートディレクトリで以下のコマンドを実行します：

```bash
cargo run --release
```

実行バイナリのみを作成する場合：

```bash
cargo build --release
```

ビルドされたバイナリは `target/release/MyNMON`（Windowsの場合は `target/release/MyNMON.exe`）に出力されます。

## ディレクトリ構成

```text
.
├── Cargo.toml            # プロジェクト設定および依存ライブラリ設定
├── LICENSE               # MITライセンス
├── README.md             # プロジェクト概要（英語版）
├── README.ja.md          # プロジェクト概要（日本語版・本ファイル）
├── src/
│   └── main.rs           # アプリケーションのメインソースコード
└── docs/
    ├── SPEC.md           # システム設計仕様書（日本語）
    ├── DIAGRAM.md        # システム構成図（日本語）
    ├── FOOTPRINTS.md     # バイナリサイズ・メモリ使用量測定レポート（日本語）
    └── TEST_REPORT.md    # 動作検証・テストレポート（日本語）
```

## リリース最適化設定

バイナリサイズおよびメモリ/CPU負荷を最小化するため、`Cargo.toml` の `[profile.release]` に以下の設定を適用しています：

- `opt-level = 'z'` (サイズ優先の最適化)
- `lto = true` (リンク時最適化)
- `codegen-units = 1` (最適化処理を1ユニットに統合)
- `panic = 'abort'` (パニック時のスタック展開を無効化)
- `strip = true` (デバッグ情報とシンボル情報を削除)

## ライセンス

本プロジェクトは [MIT License](LICENSE) のもとで公開されています。
