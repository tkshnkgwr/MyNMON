# rust-nmon

A lightweight, cross-platform CLI system monitor inspired by the classic `nmon` utility, written in Rust. It utilizes `sysinfo` for retrieving system metrics and `crossterm` for rendering a terminal-based user interface.

## Features

- **Multi-section display**: Easily toggle display sections dynamically.
- **CPU Utilization**: View usage percentages and visual bars for individual CPU cores.
- **Memory Allocation**: Real-time stats on physical RAM (total, used, and free).
- **Disk Mounts & Space**: Monitor disk mounts, filesystems, and available space.
- **Network Interface I/O**: Track Rx/Tx speeds in KB/s for all active interfaces.
- **Top Active Processes**: Monitor top processes sorted by CPU usage.
- **Interactive Control**: Toggle components using simple keyboard shortcuts.
- **Highly Optimized**: Small binary footprint (~300 KB) and low memory usage (~18 MB).

## Keyboard Shortcuts

Press these keys while the application is running to toggle sections or quit:

- `c` : Toggle CPU Core utilization section
- `m` : Toggle Memory allocation section
- `d` : Toggle Disk mounts & space section
- `n` : Toggle Network interface speed section
- `p` or `t` : Toggle Top processes section
- `q` or `Esc` : Quit the application

## Getting Started

### Prerequisites

Ensure you have Rust and Cargo installed. (Rust 1.70.0 or higher is recommended)

### Build and Run

Clone this repository and run the following command in the project directory:

```bash
cargo run --release
```

To build a standalone executable:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/rust-nmon` (or `target/release/rust-nmon.exe` on Windows).

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
