# 変更履歴 (CHANGELOG.md)

このプロジェクトのすべての重要な変更は、このファイルに記録されます。

---

## [0.1.1] - 2026-06-30

### 追加
- **MITライセンス**: プロジェクトルートに `LICENSE` ファイルを追加。
- **多言語対応**: 日本語版の概要説明書 `README.ja.md` を新規作成。
- **システム構成図**: Mermaidによる制御フロー・データフローを記述した `docs/DIAGRAM.md` を新規作成。
- **フットプリントレポート**: 実行バイナリサイズおよびメモリ使用量の実測値をまとめた `docs/FOOTPRINTS.md` を新規作成。
- **自動テスト**: アスキーバー生成関数 `get_ascii_bar` に対する単体テストを追加。
- **リリースビルド最適化**: `Cargo.toml` にリリースプロファイル設定（`opt-level = 'z'`, `lto`, `codegen-units`, `panic = 'abort'`, `strip`）を追加。

### 変更
- **英語版README**: `README.md` を現在の Rust CUI システムモニターに特化した構成にリファクタリング。
- **システム仕様書**: `docs/SPEC.md` を Rust CUI システムモニターに特化した内容へリファクタリング。
- **テスト報告書**: `docs/test_report.md` を大文字の `docs/TEST_REPORT.md` へ改名し、今回の自動テスト・手動検証結果に即した内容へアップデート。

### 修正
- **コンパイルエラー**: `sysinfo` クレート v0.30 への移行に伴うAPIの不整合を修正。
  - 拡張トレイト（`CpuExt`, `DiskExt`, `NetworkExt`, `NetworksExt`, `ProcessExt`, `SystemExt`）のインポートを削除し、直接のメソッド呼び出しへ変更。
  - `Disks` および `Networks` 構造体の導入によるライフサイクルとリフレッシュ処理の分離。
  - ディスク情報取得時の `disk.format()` から `disk.file_system()` への変更。
  - 各セクションごとの refresh メソッド呼び出しから不要な bool 引数（`true`）を削除。
