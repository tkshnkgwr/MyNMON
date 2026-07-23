# Changelog (CHANGELOG.md)

**English** | [日本語版](../ja/CHANGELOG.md)

All notable changes to this project will be documented in this file.

## [0.5.3] - 2026-07-21

### Added
- **1000-Line Code Limit Guidelines**:
  - Appended guideline Section 10 to `.agents/AGENTS.md`. Instructs AI agents to propose and perform modular refactoring when a single source file exceeds 1,000 lines.
- **Cargo Features Support for Component Isolation**:
  - Added a `[features]` section to `Cargo.toml` defining modular toggles (`default = ["cpu", "mem", "disk", "net", "proc", "diff"]`).
  - Added conditional compilation attributes (`#[cfg(feature = "...")]`) across `state.rs`, `ui.rs`, `main.rs`, and `utils.rs` to allow custom lightweight builds.
  - Verified that compiling with `--no-default-features --features cpu` successfully cuts release binary sizes from ~672 KB down to ~324 KB.

## [0.5.2] - 2026-07-16

### Changed
- **Unified Document Naming Conventions (UPPER_SNAKE_CASE)**:
  - Standardized all document filenames to use UPPER_SNAKE_CASE.
  - Renamed: `docs/Architecture.md` -> `docs/ARCHITECTURE.md`, `docs/Todo.md` -> `docs/TODO.md`, `.agents/Instructions.md` -> `.agents/INSTRUCTIONS.md` (and updated relative references inside files).

## [0.5.1] - 2026-07-16

### Added
- **System Architecture Document (`docs/ARCHITECTURE.md`)**:
  - Outlined component layout, technological stack, directory decisions, and metrics/flicker-free data flows.
- **AI Agent Guidelines (`.agents/INSTRUCTIONS.md`)**:
  - Established coding rules, error mitigation patterns, separation of concerns modules, and formatting expectations for AI agents.
- **Task Backlog (`docs/TODO.md`)**:
  - Organized completed milestones, short-term additions (millisecond polling), and future directions (NVIDIA NVML GPU support).

### Changed
- **Refactored AGENTS.md**:
  - Stripped out rules related to GUI frameworks (egui/eframe) and replaced them with CUI-specific guidelines.
- **Organized Document Placements**:
  - Moved tasks to `docs/TODO.md` and AI instructions to `.agents/INSTRUCTIONS.md`.

---

## [0.5.0] - 2026-07-14

### Changed
- **Module Segmentation Refactoring**:
  - Segmented the growing `src/main.rs` (~890 lines) into separate components:
    - `src/state.rs`: Managed state definitions (`MonitorState`).
    - `src/ui.rs`: Managed component layout drawings and custom `w_line!` macros.
    - `src/utils.rs`: Managed helper functions (padding, ASCII progress bars, process parsing) and unit tests.
    - `src/main.rs`: Retained only startup arguments, single-instance verification, and event loop logic.

---

## [0.4.2] - 2026-07-14

### Added
- **Detailed Rustdoc Documentation**:
  - Enriched source files with detailed Japanese Rustdoc comments.
- **Guideline Reinforcement**:
  - Stated in `AGENTS.md` that all source code comments and docstrings should be maintained in Japanese.

---

## [0.4.1] - 2026-07-13

### Changed
- **Refactored draw_ui Layouts**:
  - Segmented monolithic `draw_ui` procedures into separate layouts (header, core CPU, memory, network, etc.) to improve modularity and clean up compiler warnings.

---

## [0.4.0] - 2026-07-13

### Added
- **Terminal Dimension Protection**:
  - Guarded screen rendering by skipping outputs and displaying an 80x20 window warning if dimensions are too small.
- **Screenshots in Readme**:
  - Added demo capture to README files.
- **CI/CD Build Skipping**:
  - Skips GitHub Actions compilation steps if changes are limited only to `.md` files or `docs/`.

---

## [0.3.0] - 2026-07-13

### Added
- **Initial Welcome Help Screen**:
  - Centered commands list rendering at startup when all monitor sections are toggled off.
- **Interactive Refresh Rates**:
  - Enable keyboard shortcuts (`r` key) to dynamically alter refresh rate polling in seconds.

### Changed
- **Default Start Mode**:
  - Hides all monitor components on startup by default.

---

## [0.2.2] - 2026-07-13

### Changed
- **Network Interface Alphabetical Sort**:
  - Sorts active network interfaces alphabetically prior to drawing.

### Fixed
- **Align PID Columns**:
  - Formatted PID values as right-aligned strings (`{:>6}`) to stabilize column sizes.

---

## [0.2.1] - 2026-07-13

### Fixed
- **Stable Network & Process Layout Widths**:
  - Developed character-width helper `pad_or_truncate` to balance full-width/half-width characters in terminal alignment.
  - Aligned network labels to 16 columns and process names to 20 columns.

---

## [0.2.0] - 2026-07-13

### Added
- **Uptime Monitor**:
  - Retrieves `sysinfo::System::uptime()` to render system uptime in the header.

### Changed
- **Flicker-Free Upwrites**:
  - Deprecated full-screen clears (`Clear(All)`), replacing them with cursor reset upwrites combined with line-end and bottom clears.

---

## [0.1.9] - 2026-07-13

### Fixed
- **Windows Keystroke Toggles**:
  - Filtered keyboard poll listeners to process only `KeyEventKind::Press` events on Windows.

---

## [0.1.8] - 2026-07-10

### Changed
- **二重起動防止 (Double-Launch Prevention) API Update**:
  - Synced error handling after `common_lib::check_single_instance` was changed to return `Result`.

---

## [0.1.7] - 2026-07-06

### Changed
- **Stabilize GitHub Actions Actions**:
  - Pinned non-deprecated Actions versions (`actions/checkout@v4`, `softprops/action-gh-release@v2`) and automated parent directory clones for `common_lib` dependencies.
- **Fixed Clippy Warnings**:
  - Cleaned up manual clamps and empty `writeln!` warnings.

---

## [0.1.6] - 2026-07-03

### Added
- **Integrated common_lib Core**:
  - Used Named Mutexes, process match counts, and frame-by-frame diff calculations.
- **Process Filters**:
  - Added filter keyword inputs using the `f` key.
- **Diff Log Toggles**:
  - Added toggles (`g`/`l`) to show process change histories.

---

## [0.1.5] - 2026-06-30

### Added
- **README Status Badges**:
  - Added shields.io badges showing build status, Rust version requirements, OS supports, and licenses.

---

## [0.1.4] - 2026-06-30

### Changed
- **Rust Toolchain Sync**:
  - Upgraded requirements to Rust 1.96.0; logged benchmarks to footprint files.

---

## [0.1.3] - 2026-06-30

### Changed
- **Rename Project to MyNMON**:
  - Renamed package configurations, document listings, and system labels from `rust-nmon` to `MyNMON`.

---

## [0.1.2] - 2026-06-30

### Added
- **CLI Arguments**:
  - Added `-h`/`--help` and `-v`/`--version` options.

---

## [0.1.1] - 2026-06-30

### Added
- **MIT License**: Included LICENSE.
- **System Diagram**: Mermaid configurations in `docs/DIAGRAM.md`.
- **Benchmark report**: Initial size benchmarks in `docs/FOOTPRINTS.md`.
- **Unit Test**: Test assertions for `get_ascii_bar`.
