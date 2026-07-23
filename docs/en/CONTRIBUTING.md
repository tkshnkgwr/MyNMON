# Contributing Guidelines (CONTRIBUTING.md) - MyNMON

**English** | [日本語版](../ja/CONTRIBUTING.md)

Thank you for your interest in contributing to the `MyNMON` project!
This document describes the guidelines for reporting bugs, proposing features, and submitting pull requests.

---

## 1. Development Policies and Principles

When developing, please adhere to the following basic policies:

1. **Double-Launch Prevention and CUI Protection**:
   - When modifying crossterm-based screen overwrites, terminal size checking, or Windows Named Mutex logic, ensure that terminal state safety and stability are maintained.
2. **Synchronize Multi-language Documents**:
   - When making changes to features or specifications, update documents in both `docs/ja/` and `docs/en/` to keep them synchronized.
3. **Collaboration with `common_lib`**:
   - This project depends on `common_lib` located in the same parent directory. When modifying shared functionalities (such as Mutex or difference computation), ensure integration remains correct.

---

## 2. Development Setup

1. **Cloning the Repositories**:
   ```bash
   # Both repositories must be cloned under the same parent directory
   git clone https://github.com/tkshnkgwr/common_lib.git
   git clone https://github.com/tkshnkgwr/MyNMON.git
   cd MyNMON
   ```
2. **Running the Application**:
   ```bash
   cargo run --release
   ```

---

## 3. Commits and Pull Requests

### Commit Message Conventions
Please use the Conventional Commits format for your commit messages:

- `feat:` Add new feature
- `fix:` Fix bug
- `docs:` Change documentation
- `refactor:` Refactor code
- `perf:` Improve performance
- `test:` Add or modify tests
- `chore:` Update build scripts or configurations

### Pull Request Checklist
Before submitting a pull request, run the following commands and verify that all pass successfully:

- [ ] `cargo test` (Unit tests pass)
- [ ] `cargo clippy --all-targets -- -D warnings` (Zero linter warnings/errors)
- [ ] `cargo fmt --check` (Formatting rules applied)
- [ ] `cargo doc --no-deps --document-private-items` (Documentation builds without errors/warnings)
