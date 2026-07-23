# AI Development Instructions (INSTRUCTIONS.md) - MyNMON

**English** | [日本語版](../ja/INSTRUCTIONS.md)

This document defines the guidelines and development standards that the AI Agent (Daikenja) must adhere to when modifying, maintaining, or adding features to the `MyNMON` codebase.

---

## 1. Development Principles

1. **CUI Rendering Quality & Flicker Prevention**:
   - Do not clear the entire screen (`terminal::Clear(ClearType::All)`). Instead, reset the cursor to `(0, 0)` and overwrite the screen buffer.
   - Use `UntilNewLine` for clearing to the end of each line and `FromCursorDown` at the end of rendering to prevent ghost characters.
   - If the terminal size falls below "80 columns by 20 rows", skip rendering and display a resize warning message.
2. **Double Launch Prevention**:
   - On Windows, ensure that the double-launch prevention logic using a Named Mutex via `common_lib::check_single_instance` is maintained and active.
3. **Release Size & Memory Optimization**:
   - Keep the memory footprint below 25MB and the binary size below 350KB.
   - When adding features, consider using optional Cargo Features (conditional compilation with `#[cfg(feature = "...")]`).
4. **Explanations Sensitive to Multi-language Developers**:
   - The user has development experience in other languages but is less familiar with Rust-specific terminology (ownership, traits, lifetimes, etc.). When explaining code changes, map Rust-specific terms to general programming concepts or provide clear explanations.

---

## 2. Coding Style Guidelines

- **Formatting**: Adhere to the default style enforced by `cargo fmt`.
- **Naming Conventions**:
  - Structs / Enums: `PascalCase` (e.g., `MonitorState`)
  - Functions / Variables / Modules: `snake_case` (e.g., `draw_cpu_total`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MUTEX_NAME`)
- **Error Handling**:
  - Avoid panicking (`panic!`, `unwrap()`). Use proper fallback values or return errors instead. In the event of a panic, the process is safely aborted immediately as per `panic = 'abort'`.

---

## 3. Testing and Verification Policy

- When adding new logic, write corresponding unit tests to maintain test coverage.
- After making changes, run the following standard commands and verify that there are zero warnings and errors:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `cargo doc --no-deps --document-private-items`

---

## 4. Module Separation and Refactoring Policy

- **1,000-Line Limit Rule**:
  - If a single program source file (e.g., `src/*.rs`) exceeds **1,000 lines**, propose and execute refactoring by splitting it into logical modules (e.g., `cpu.rs`, `mem.rs`, etc.) to maintain readability and maintainability.
