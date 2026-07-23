# Task Management (TODO.md)

**English** | [日本語版](../ja/TODO.md)

This document is used to track the progress of development tasks and future roadmap for `MyNMON`.

---

## 1. Completed Tasks (Done)

- [x] **Basic Monitor Features**: CPU (total/cores), memory, disk, networks, processes.
- [x] **CUI Layout Guard & Flicker Prevention**: 80x20 terminal size check, cursor reset rewrite, line-end clears.
- [x] **Double-Launch Prevention**: Windows Named Mutex control using `common_lib::check_single_instance`.
- [x] **Interactive Controls**: Display toggling, process name filtering, refresh interval updates.
- [x] **Module Separation**: Splitting code into `main.rs`, `state.rs`, `ui.rs`, and `utils.rs`.
- [x] **Modular Cargo Features**: Isolating components (`cpu`, `mem`, `disk`, `net`, `proc`, `diff`) for optional compilation.

---

## 2. Roadmap / Future Tasks (Todo)

- [ ] **Optional GPU Monitoring**: Add NVIDIA GPU (NVML) support under an optional feature flag.
- [ ] **Logging (CSV/JSON)**: Log metrics to files at configured intervals in the background.
