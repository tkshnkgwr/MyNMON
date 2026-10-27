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

## 3. Documentation and Update Rules

When adding, modifying, or refactoring code or dependencies, keep both the **English (`docs/en/`) and Japanese (`docs/ja/`) documentation** synchronized.
*Note: For Markdown-only (`.md`) edits, automated documentation rules and verification steps are omitted to allow prompt responses.*

| Document (EN / JA)              | Role and Update Rules                                                                                                     |
| :------------------------------ | :------------------------------------------------------------------------------------------------------------------------ |
| `CHANGELOG.md`                  | Change history. Add date `## [X.Y.Z] - YYYY-MM-DD` and category (`### Added`, `### Changed`, etc.) with latest on top.   |
| `SPEC.md`                       | Key bindings, CLI arguments, display rules, metrics, and technical stack definitions.                                     |
| `DIAGRAM.md`                    | System architecture, event loop order, and data flow Mermaid diagrams.                                                   |
| `README.md` / `README_JA.md`    | Overview, build/run steps, and prerequisites. Update on CLI option or requirement changes.                                |
| `FOOTPRINTS.md`                 | Performance, binary size, and memory usage measurement records.                                                           |
| `ARCHITECTURE.md`               | Architecture overview, module structure, data flows, and Cargo Features compilation boundaries.                           |
| `INSTRUCTIONS.md`               | AI agent coding standards, error handling, modularization, and response formatting rules.                                 |
| `TODO.md`                       | Task backlog (Done / Todo / Backlog).                                                                                     |
| `TESTING.md` / `TEST_REPORT.md`| Test policy and execution procedures / historical verification and automated/manual test reports.                         |
| `RELEASE.md` / `SECURITY.md`    | Release workflows / security design principles, support policy, and vulnerability reporting.                              |
| `CONTRIBUTING.md`               | Contributor guidelines and workflow.                                                                                      |
| Rustdoc (in-source comments)    | Detailed `///` and `//!` doc comments. Synchronize comments on any API or signature changes.                              |

---

## 4. Quality Control & Verification Rules

- **1,000-Line Limit Rule**:
  - If a single program source file (e.g., `src/*.rs`) exceeds **1,000 lines**, propose and execute modular refactoring to maintain clean separation of concerns.
- **Verification Workflow**:
  - **For Markdown-only (`.md`) edits, skip Rust verification steps (`cargo fmt`, `cargo check`, `cargo test`, etc.) to respond swiftly.**
  - When modifying source code (`.rs`, `Cargo.toml`, etc.), verify zero warnings and errors:
    1. `cargo test`
    2. `cargo clippy --all-targets -- -D warnings`
    3. `cargo fmt --check`
    4. `cargo doc --no-deps --document-private-items`
