# 貢献ガイドライン (CONTRIBUTING.md) - MyNMON

[English](../en/CONTRIBUTING.md) | **日本語版**

`MyNMON` プロジェクトへの貢献に興味を持っていただきありがとうございます！
本ドキュメントでは、バグ報告、機能提案、プルリクエスト提出時のガイドラインを説明します。

---

## 1. 開発方針と重要原則

開発を行う際は、以下の基本方針を遵守してください。

1. **二重起動防止とCUI保護ルール**:
   - `crossterm` の上書き描画、画面サイズ確認、Windows Named Mutex 等の仕様を変更する際は、必ず挙動や例外時の安全性を維持するように配慮してください。
2. **多言語ドキュメントの同期**:
   - 仕様変更や機能追加を行う際は、`docs/ja/` および `docs/en/` の両方のドキュメントを更新し、内容の同期を保ってください。
3. **共有ライブラリ `common_lib` との連携**:
   - 本プロジェクトは同一親ディレクトリ内の `common_lib` に依存しています。共有機能（Mutexや差分計算等）の変更時には、整合性を確認してください。

---

## 2. 開発環境のセットアップ

1. **リポジトリのクローン**:
   ```bash
   # 同一親ディレクトリ内に両プロジェクトを配置
   git clone https://github.com/tkshnkgwr/common_lib.git
   git clone https://github.com/tkshnkgwr/MyNMON.git
   cd MyNMON
   ```
2. **動作確認**:
   ```bash
   cargo run --release
   ```

---

## 3. コミットおよびプルリクエスト手順

### コミットメッセージの規約
コミットメッセージには Conventional Commits 形式を使用してください：

- `feat:` 新機能の追加
- `fix:` バグ修正
- `docs:` ドキュメントの変更
- `refactor:` リファクタリング
- `perf:` パフォーマンス改善
- `test:` テストの追加・修正
- `chore:` ビルドスクリプトや設定の変更

### プルリクエスト作成前のチェックリスト
プルリクエストを送信する前に、以下のコマンドを実行し全て合格することを確認してください：

- [ ] `cargo test` （ユニットテスト合格）
- [ ] `cargo clippy --all-targets -- -D warnings` （静的解析の警告ゼロ）
- [ ] `cargo fmt --check` （コードフォーマット準拠）
- [ ] `cargo doc --no-deps --document-private-items` （ドキュメントビルドエラーゼロ）
