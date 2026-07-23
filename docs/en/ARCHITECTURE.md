# System Architecture (ARCHITECTURE.md)

**English** | [日本語版](../ja/ARCHITECTURE.md)

This document defines the architectural design, component structure, data flow, and modular separation strategy via conditional compilation (Cargo Features) for the ultra-lightweight CUI system monitor `MyNMON`.

---

## 1. Overall Structure and Module Separation

`MyNMON` is divided into four source files based on the Separation of Concerns principle:

```text
src/
├── main.rs      # Entry point, double-launch check, CLI argument parsing, main event loop
├── state.rs     # Application state struct MonitorState definition (feature-controlled)
├── ui.rs        # Terminal rendering functions using Crossterm (feature-controlled)
└── utils.rs     # Utility functions for time formatting, ASCII bar generation, character width, etc.
```

---

## 2. Cargo Features (Conditional Compilation) Architecture

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
