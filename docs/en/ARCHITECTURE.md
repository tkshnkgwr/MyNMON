# System Architecture (ARCHITECTURE.md)

**English** | [日本語版](../ja/ARCHITECTURE.md)

This document defines the architectural design, component structure, technology stack, and modular separation strategy via conditional compilation (Cargo Features) for the ultra-lightweight CUI system monitor `MyNMON`.

---

## 1. Overall Structure and Module Separation

`MyNMON` is divided into four source files based on the Separation of Concerns principle:

```text
src/
├── main.rs      # Entry point, double-launch check, CLI argument parsing, TerminalGuard (RAII), main event loop
├── state.rs     # Application state struct MonitorState definition (feature-controlled)
├── ui.rs        # Terminal rendering functions using Crossterm (feature-controlled)
└── utils.rs     # Utility functions for time formatting, ASCII bar generation, character width, and unit tests
```

### Design Highlights for Safety & Responsiveness

- **RAII Terminal State Restoration**: `TerminalGuard` implements `Drop` so that raw mode disablement, alternate screen exit, and cursor restoration are guaranteed both on normal exit and on early error returns.
- **Decoupled Metrics Refresh & UI Rendering**: System metric collection (`sys.refresh_all()`, `disks.refresh(true)`, `networks.refresh(true)`) and process diff detection execute only when `tick_rate` elapses, while keypresses trigger immediate UI redraws. This achieves both responsive interactive input and accurate per-second network throughput (KB/s) calculation.

---

## 2. Technology Stack and Shared Library Integration

| Crate        | Version  | Role                                                                                          |
| :----------- | :------- | :-------------------------------------------------------------------------------------------- |
| `sysinfo`    | `0.40.x` | OS-independent metric collection for CPU, memory, disks, network interfaces, and processes    |
| `crossterm`  | `0.29.x` | Raw mode control, alternate screen switching, styled output, and non-blocking event polling   |
| `common_lib` | `0.4.x`  | Shared core crate (`../common_lib`) providing single-instance lock, diffing, and match counts |

---

## 3. Cargo Features (Conditional Compilation) Architecture

To support binary size optimization for specific use cases or embedded environments, monitoring components are isolated into Feature flags:

```text
               ┌─────────────────────────────┐
               │    default = ["cpu", ...]   │
               └──────────────┬──────────────┘
                              │
  ┌──────────┬──────────┬─────┴────┬──────────┬──────────┐
  │          │          │          │          │          │
  ▼          ▼          ▼          ▼          ▼          ▼
[cpu]      [mem]      [disk]     [net]      [proc] ◄── [diff]
```

- Each feature (`cpu`, `mem`, `disk`, `net`, `proc`, `diff`) has its data collection, rendering logic, and key toggle events compiled conditionally via `#[cfg(feature = "...")]`.
- The `diff` (process spawn/exit logs) feature automatically depends on the `proc` feature to enable process list comparison.
