# リリース手順書 (RELEASE.md) - MyNMON

[English](../en/RELEASE.md) | **日本語版**

本文書は、`MyNMON` プロジェクトのバージョン更新およびリリース作業の手順をまとめたマニュアルです。

---

## 1. リリース前の事前準備

リリース作業を行う前に、すべてのコードおよびドキュメントが最新の品質基準を満たしていることを確認します。

1. **品質検証コマンドの合格**:
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   cargo doc --no-deps --document-private-items
   ```
2. **ドキュメントの更新確認**:
   - `docs/ja/CHANGELOG.md` および `docs/en/CHANGELOG.md` にリリース内容が追記されていること。
   - `docs/ja/FOOTPRINTS.md` および `docs/en/FOOTPRINTS.md` に最新バイナリサイズ等の性能値が記録されていること。

---

## 2. バージョンの更新手順

1. **`Cargo.toml` のバージョン更新**:
   ```toml
   [package]
   name = "MyNMON"
   version = "X.Y.Z" # 新バージョンを指定
   ```
2. **`Cargo.lock` の同期**:
   ```bash
   cargo check
   ```
3. **`README.md` および `README_JA.md` のバッジ更新**:
   - リリースバージョン（`Latest Release` バッジ等）が最新になるよう、必要に応じてURLのタグ文字列を変更します。

---

## 3. ビルドとタグ打ち

1. **リリースバイナリのビルド**:
   ```bash
   cargo build --release
   ```
2. **Git コミットおよびタグ作成**:
   ```bash
   git add .
   git commit -m "chore: release vX.Y.Z"
   git tag -a vX.Y.Z -m "Release version X.Y.Z"
   ```
3. **リモートリポジトリへのプッシュ**:
   ```bash
   git push origin main --tags
   ```

---

## 4. リリース完了後の確認

- GitHub Actions (CI/CD) のビルドワークフローが成功することを確認します。
- GitHub Releaseページが自動作成され、各OS向けバイナリ（Windows, Linux, macOS 用のリリースアセット）が添付されているか確認します。
