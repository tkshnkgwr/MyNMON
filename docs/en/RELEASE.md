# Release Procedures (RELEASE.md) - MyNMON

**English** | [日本語版](../ja/RELEASE.md)

This document is a guide explaining the procedures for updating versions and publishing releases for the `MyNMON` project.

---

## 1. Pre-release Preparation

Before carrying out release tasks, verify that all code and documentation meet the latest quality standards.

1. **Verify Quality Command Outputs**:
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   cargo doc --no-deps --document-private-items
   ```
2. **Verify Document Status**:
   - Check that the release details are appended to `docs/ja/CHANGELOG.md` and `docs/en/CHANGELOG.md`.
   - Check that the latest binary sizes and footprint metrics are logged in `docs/ja/FOOTPRINTS.md` and `docs/en/FOOTPRINTS.md`.

---

## 2. Version Update Steps

1. **Update Version in `Cargo.toml`**:
   ```toml
   [package]
   name = "MyNMON"
   version = "X.Y.Z" # Specify the new version
   ```
2. **Synchronize `Cargo.lock`**:
   ```bash
   cargo check
   ```
3. **Update Badges in `README.md` and `README_JA.md`**:
   - Update tag strings in badge image URLs (e.g., `Latest Release` badge) if necessary.

---

## 3. Build & Tagging

1. **Build Release Binary**:
   ```bash
   cargo build --release
   ```
2. **Commit Changes & Create Tag**:
   ```bash
   git add .
   git commit -m "chore: release vX.Y.Z"
   git tag -a vX.Y.Z -m "Release version X.Y.Z"
   ```
3. **Push to Remote Repository**:
   ```bash
   git push origin main --tags
   ```

---

## 4. Post-release Verification

- Confirm that the GitHub Actions (CI/CD) workflow builds successfully.
- Ensure that the GitHub Release page is automatically generated and binaries for each platform (Windows, Linux, macOS) are attached.
