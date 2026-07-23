# Testing Policy & Execution Guide (TESTING.md) - MyNMON

**English** | [日本語版](../ja/TESTING.md)

This document outlines the testing policies, manual validation steps, and quality check procedures for the `MyNMON` project.

---

## 1. Testing Overview

`MyNMON` is a lightweight CUI system monitor written in Rust. To ensure peak performance and terminal rendering integrity, we validate the system against the following criteria:

Key Test Perspectives:
- **Unit Tests**: Verifying algorithms such as ASCII progress bar generation logic.
- **Section Toggling & State Management**: Ensuring that display toggles work correctly in response to key presses.
- **CUI Integrity & Flicker Prevention**: Verifying flicker-free screen updates and size checks across various terminal sizes.
- **Double-Launch Prevention (Windows Only)**: Verifying Named Mutex-based detection of parallel instances.
- **Conditional Compilation**: Verifying build validity when utilizing different Cargo Feature combinations (e.g., full features vs. cpu-only).

---

## 2. Running Tests

Follow the steps below to run tests in your local development environment:

### Run Unit Tests
```bash
cargo test
```

### Run a Specific Test Case
```bash
cargo test test_get_ascii_bar
```

### Verification & Quality Check Commands
Before finalizing any code modifications, verify that the following standard commands pass without any warnings or errors:

```bash
# 1. Run unit tests across all features
cargo test --all-features

# 2. Static Analysis (Clippy)
cargo clippy --all-targets -- -D warnings

# 3. Check Code Formatting
cargo fmt --check

# 4. Validate Rustdoc Build
cargo doc --no-deps --document-private-items
```

---

## 3. Guidelines for Writing Tests

1. **Inline Module Tests**:
   - Write tests in a `#[cfg(test)] mod tests` block at the bottom of the source file (e.g., in `src/utils.rs`) to test underlying algorithms.
2. **Robustness Testing**:
   - Verify that the application recovers gracefully (using fallbacks or returning errors) rather than panicking when faced with invalid inputs or system states.
3. **Handling Cargo Features**:
   - Ensure that code and tests enclosed under `#[cfg(feature = "...")]` remain compilable even when target features are disabled.
