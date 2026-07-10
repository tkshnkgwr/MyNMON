# MyNMON

[日本語 (Japanese)](README.ja.md)

![CI Status](https://github.com/tkshnkgwr/MyNMON/actions/workflows/ci.yml/badge.svg)
![Latest Release](https://img.shields.io/github/v/release/tkshnkgwr/MyNMON)
![Rust Version](https://img.shields.io/badge/rust-1.96.0%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A lightweight, cross-platform CLI system monitor inspired by the classic `nmon` utility, written in Rust. It utilizes `sysinfo` for retrieving system metrics and `crossterm` for rendering a terminal-based user interface.

## Features

- **Multi-section display**: Easily toggle display sections dynamically.
- **CPU Utilization**: View usage percentages and visual bars for individual CPU cores.
- **Memory Allocation**: Real-time stats on physical RAM (total, used, and free).
- **Disk Mounts & Space**: Monitor disk mounts, filesystems, and available space.
- **Network Interface I/O**: Track Rx/Tx speeds in KB/s for all active interfaces.
- **Top Active Processes**: Monitor top processes sorted by CPU usage.
- **Process Search/Filter**: Real-time filtering of the process list by name, with active match counts.
- **Process Change Log**: Real-time spawn/exit logs of processes (+ for start, - for exit) showing recent changes.
- **Double Launch Prevention**: Named Mutex protection on Windows to prevent screen rendering conflicts.
- **Interactive Control**: Toggle components using simple keyboard shortcuts.
- **Highly Optimized**: Small binary footprint (~308 KB) and low memory usage (~20.8 MB).

## Keyboard Shortcuts

Press these keys while the application is running to toggle sections or quit:

- `c` : Toggle CPU Core utilization section
- `m` : Toggle Memory allocation section
- `d` : Toggle Disk mounts & space section
- `n` : Toggle Network interface speed section
- `p` or `t` : Toggle Top processes section
- `g` or `l` : Toggle Process spawn/exit history log section
- `f` : Start process search/filter mode (Press `Enter` or `Esc` to apply/exit search mode)
- `q` or `Esc` : Quit the application (when not in search input mode)

## Command-Line Options

You can run `MyNMON` with the following command-line flags:

- `-h`, `--help` : Print the help message containing command usage and options, then exit.
- `-v`, `--version` : Print the dynamically resolved application version (from `Cargo.toml`), then exit.

Example usage:
```bash
./MyNMON --help
./MyNMON --version
```

## Getting Started

### Prerequisites

1. Ensure you have Rust and Cargo installed. (Rust 1.96.0 or higher is recommended)
2. This project depends on a shared library `common_lib` via a relative path (`../common_lib`). You need to clone both repositories in the same parent directory:

```bash
# Clone the shared library
git clone https://github.com/tkshnkgwr/common_lib.git

# Clone the main project (MyNMON)
git clone https://github.com/tkshnkgwr/MyNMON.git
```

Your directory structure should look like this:
```text
parent_directory/
├── common_lib/
└── MyNMON/
```

### Build and Run

Clone this repository and run the following command in the project directory:

```bash
cargo run --release
```

To build a standalone executable:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/MyNMON` (or `target/release/MyNMON.exe` on Windows).

## Directory Structure

```text
.
├── Cargo.toml            # Project configuration and dependency settings
├── LICENSE               # MIT License
├── README.md             # Project overview (English)
├── README.ja.md          # Project overview (Japanese)
├── src/
│   └── main.rs           # Core application source code
└── docs/
    ├── SPEC.md           # System Specification (Japanese)
    ├── DIAGRAM.md        # System Architecture Diagram (Japanese)
    ├── FOOTPRINTS.md     # Binary size and memory footprint report (Japanese)
    └── TEST_REPORT.md    # Verification and testing report (Japanese)
```

## Optimization Settings

This project includes release profile optimizations in `Cargo.toml` to minimize the binary size and CPU/memory footprint:

- `opt-level = 'z'` (optimizes for size)
- `lto = true` (Link-Time Optimization)
- `codegen-units = 1` (improves optimization and binary size)
- `panic = 'abort'` (disables stack unwinding on panic)
- `strip = true` (strips symbols and debug information)

## License

This project is licensed under the [MIT License](LICENSE).
