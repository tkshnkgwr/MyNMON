# System Specification (SPEC.md)

**English** | [日本語版](../ja/SPEC.md)

This specification defines the functional requirements, non-functional requirements, and internal structure of `MyNMON`, a cross-platform, ultra-lightweight CUI system monitor.

---

## 1. System Overview

`MyNMON` is an ASCII User Interface (CUI) utility that monitors local computer hardware resources in real-time in the terminal.
Inspired by the classic Linux monitoring tool `nmon`, it is designed with the following principles:

- **Portability**: Compiles and runs unmodified across cross-platform environments such as Windows, Linux, and macOS.
- **Ultra-lightweight**: Minimizes its own consumption of CPU and memory resources to avoid placing an unnecessary load on the monitored system. (Cargo Features allow unused modules to be excluded from the binary).
- **Interactivity**: Allows dynamic section toggles via simple keypresses.

---

## 2. System Architecture

The system is a single executable binary that operates on a single-threaded event loop model.

```text
┌────────────────────────────────────────────────────────┐
│                        MyNMON                          │
│                                                        │
│  ┌───────────────────────────┐                         │
│  │       Event Loop          │                         │
│  │  (1-second interval or)   │                         │
│  │  (Keyboard interrupt)     │                         │
│  └─────────────┬─────────────┘                         │
│                │                                       │
│                ▼                                       │
│  ┌───────────────────────────┐                         │
│  │   Metrics Collection      │                         │
│  │  (sysinfo crate)          │                         │
│  │  - CPU Cores, Memory      │                         │
│  │  - Disks, Networks, Proc  │                         │
│  └─────────────┬─────────────┘                         │
│                │                                       │
│                ▼                                       │
│  ┌───────────────────────────┐                         │
│  │     Terminal Rendering    │                         │
│  │  (crossterm crate)        │                         │
│  │  - Color styling          │                         │
│  │  - ASCII progress bar     │                         │
│  └───────────────────────────┘                         │
└────────────────────────────────────────────────────────┘
```

### Key Libraries and Roles
- **`sysinfo`**: Obtains metrics for hardware, processes, OS, networks, and disks via an OS-independent abstract API. It contains platform-specific implementations such as the Windows API and the Linux `/proc` filesystem.
- **`crossterm`**: Manages terminal alternate screens, cursor visibility, raw mode control, keyboard event monitoring, and terminal control sequences.

---

## 3. Functional Requirements

### 3.1 Common Header Display
The top line of the screen always displays basic system information and status. To prevent layout corruption (unwanted line wrapping), items are dynamically omitted based on the current terminal width.
- **Full View (Width >= 105 chars)**: Program Name & Version (e.g., `MyNMON v0.5.0`) | Host: hostname | OS: OS Name | Kernel: Kernel Version | Uptime: Uptime | Interval: Refresh Interval
- **Medium View (Width 90–104 chars)**: Hides the Kernel Version, displaying the rest.
- **Small View (Width 80–89 chars)**: Hides the Kernel Version and OS Name, displaying the rest.
- **System Uptime**: Shows time elapsed since system boot in the format `Xd HH:MM:SS` (or `HH:MM:SS` if less than 24 hours), updating every second.
- **Refresh Interval**: Displays the screen refresh interval in seconds in the header (e.g., `Interval: 1s`).

### 3.2 Display and Section Control

#### 3.2.1 Welcome Screen (Initial Help)
On startup, all information sections are hidden by default (`false`).
If no sections are visible, a centered "Welcome Help Screen" containing keyboard shortcut commands is displayed in a large format.
Toggling any section ON will hide the welcome screen.

#### 3.2.2 Information Monitor Sections
The monitor includes the following information sections, which can be toggled on/off dynamically via keyboard shortcuts:

1. **Total CPU Section (`C` Key)**:
   - Displays overall CPU utilization (%).
   - Generates an ASCII progress bar (e.g., `[===>    ]`).
   - Color-coded by load level (Green: <= 40%, Yellow: 40% < load <= 80%, Red: > 80%).
2. **Individual CPU Cores Section (`c` Key)**:
   - Displays utilization (%) for each individual CPU core.
   - Generates individual ASCII progress bars.
   - Utilizes the same color-coding rules based on load levels.
3. **Memory Section (`m` Key)**:
   - Displays physical memory (RAM) and swap memory (Swap/Pagefile) total, used, and free capacities in GB.
   - Displays utilization (%) and ASCII bars for both RAM and swap (colored red if utilization exceeds 85%).
4. **Disk Section (`d` Key)**:
   - Displays mount points, filesystem formats (e.g., `"NTFS"`, `"ext4"`).
   - Displays free space / total capacity and utilization (%).
5. **Network Section (`n` Key)**:
   - Displays active network interface names (sorted alphabetically).
   - Displays Receive (Rx) and Transmit (Tx) speeds in KB/s. Interface names are calculated using character-width padding to prevent layout shifts.
6. **Process Section (`p` or `t` Key)**:
   - Displays the top 8 active processes sorted by CPU utilization.
   - Displays PID (right-aligned to 6 columns), Process Name (truncated/padded to 20 chars), CPU utilization (%), and Memory allocation (MB) in a neat tabular layout.
7. **Process Search/Filter (`f` Key)**:
   - Utilizes `common_lib::count_occurrences` to count and display the number of processes matching the input query.
   - Filters the process list display to show only processes starting with or containing the query string.
   - Exit search mode using `Enter` or `Esc`.
8. **Process Spawn/Exit History Log Section (`g` or `l` Key)**:
   - Utilizes `common_lib::compute_diff` to calculate differences in process lists between frames.
   - Keeps history logs of newly spawned processes (`+ Process (PID: xx)`) and exited processes (`- Process (PID: xx)`) up to 50 entries, displaying the latest 10 at the bottom (green for spawns, red for exits).

---

## 4. Cargo Features (Modular Compilation)

`MyNMON` supports conditional compilation (`#[cfg(feature = "...")]`) via Cargo Features to compile lightweight custom binaries.

### 4.1 Feature Flags

| Feature | Description | Dependencies |
| :--- | :--- | :--- |
| `default` | Full build with all monitoring features | `["cpu", "mem", "disk", "net", "proc", "diff"]` |
| `cpu` | Monitors total CPU and individual cores | None |
| `mem` | Monitors physical memory and swap | None |
| `disk` | Monitors disk mounts and space | None |
| `net` | Monitors network interface speeds | None |
| `proc` | Monitors top process lists and enables search/filter | None |
| `diff` | Monitors process spawn/exit logs | `proc` |

### 4.2 Custom Build Examples

- **All Features (Default)**:
  ```bash
  cargo build --release
  ```
- **CPU & Memory Only**:
  ```bash
  cargo build --release --no-default-features --features "cpu,mem"
  ```
- **Processes and Diff Log Only**:
  ```bash
  cargo build --release --no-default-features --features "proc,diff"
  ```

### 3.3 Keyboard Controls
- Pressing `q` or `Esc` restores the terminal state (disables raw mode, exits the alternate screen, restores the cursor) and exits with exit status `0`.
- Pressing `r` initiates the interactive refresh interval configuration. Enter a number and press `Enter` to apply a new tick rate (must be >= 1s), or press `Esc` to cancel.

### 3.4 Command-Line Options
You can supply flags at startup to execute specific operations and exit immediately:

1. **Help (`-h` or `--help`)**:
   - Prints application usage, options, and run-time shortcuts, then exits.
2. **Version (`-v` or `--version`)**:
   - Prints the dynamic version resolved from `Cargo.toml` at compile-time (e.g., `MyNMON v0.5.0`), then exits.
3. **Invalid Option Handling**:
   - Unknown arguments write an error (e.g., `Error: Unknown option '...'`) and usage directions to stderr and exit with status `1`.

### 3.5 Double-Launch Prevention (Windows Only)
- Integrates `common_lib::check_single_instance` to prevent running multiple instances concurrently.
- Checks a system-wide Named Mutex (`MyNMON_NamedMutex_Instance`) at startup. If active, prints `Error: Another instance of MyNMON is already running.` to stderr and exits with status `1`.

---

## 4. Non-Functional Requirements

- **Update Frequency**: Refreshes metrics and redraws the UI every 1 second (1000ms) by default. The interval can be configured dynamically using the `r` key (in seconds).
- **Latency**: Key interrupts trigger immediate UI updates and redraws without waiting for the timer tick.
- **Rendering Performance & Guard rails**:
  - Requires a minimum terminal size of **"80 columns by 20 rows"**. If smaller, rendering is skipped and a resize warning is displayed.
  - Avoids `Clear(All)` updates, rewriting the screen content from `(0, 0)` to eliminate terminal flickers.
  - Clears lines using `UntilNewLine` and clears the bottom of the screen using `FromCursorDown` to prevent trailing characters when data structures shrink.
- **Footprint**:
  - **Binary Size**: Under 350 KB (optimized release build).
  - **Memory Usage**: Under 25 MB (normal execution with process monitoring active).
- **Robustness**:
  - On panics, aborts execution immediately (`panic = 'abort'`) to prevent terminal lockups.
  - Uses fallback values (e.g., `"Unknown"`, `0`) if OS metric collection fails, ensuring the process remains running.
