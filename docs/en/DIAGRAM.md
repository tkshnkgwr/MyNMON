# System Architecture Diagram (DIAGRAM.md)

**English** | [日本語版](../ja/DIAGRAM.md)

This document provides diagrams illustrating the threads, lifecycle, data collection pathways, and event control flow of `MyNMON`.

---

## 1. Application Lifecycle and Event Loop

`MyNMON` achieves low-latency key responsiveness and accurate periodic metric collection within a single thread by combining Crossterm event polling with elapsed-time tracking.

```mermaid
sequenceDiagram
    autonumber
    actor User as User
    participant App as MyNMON (Main)
    participant Common as common_lib
    participant Term as Terminal (crossterm)
    participant System as OS / Hardware (sysinfo)

    Note over App: Startup (fn main)
    App->>Common: Double-launch check (check_single_instance)
    App->>System: Initialize metrics (System / Disks / Networks)
    App->>Term: Create TerminalGuard (Raw Mode, Alternate Screen, Hide Cursor)
    App->>System: Initial refresh & short warm-up sleep

    loop Event Loop
        opt First iteration or tick_rate elapsed
            App->>System: Refresh metrics (CPU, RAM, Disks, Networks, Processes)
            System-->>App: Latest Metrics Data
            App->>Common: Process diff calculation (compute_diff)
            Common-->>App: Update Spawn/Exit Log
        end

        App->>Term: Render UI (draw_ui)
        Term-->>User: Update Screen Display

        Note over App: Calculate Poll Timeout<br/>(tick_rate - elapsed time since last refresh)

        alt Key input detected within timeout (KeyEventKind::Press)
            User->>Term: Press Key
            Term->>App: Detect Key Event (event::poll)
            alt 'f' key (Process Filter)
                App->>App: Enter filter input mode (is_filtering = true)
            else 'r' key (Change Refresh Interval)
                App->>App: Enter interval input mode (is_setting_interval = true)
            else Section toggle keys (C, c, m, d, n, p/t, g/l)
                App->>App: Toggle display state (MonitorState)
            end
        else Timeout exceeded
            Note over App: Trigger metrics refresh on next iteration
        end

        alt 'q' or 'Esc' pressed (normal mode)
            App->>Term: TerminalGuard::drop (Disable Raw Mode, Normal Screen, Show Cursor)
            Note over App: Terminate Process (exit 0)
            App->>User: Return Terminal Control
        end
    end
```

---

## 2. Data Flow and Rendering Path

The following diagram illustrates how metrics collected from the OS via `sysinfo` and analysis results from `common_lib` are processed and rendered to the terminal buffer:

```mermaid
graph TD
    subgraph "OS (Kernel Space)"
        ProcFS["Linux /proc"]
        WinAPI["Windows API"]
        macOS["macOS sysctl"]
    end

    subgraph "sysinfo Crate (Data Collection)"
        SysInst["System (CPU, RAM, Processes)"]
        DiskInst["Disks (Mounts, Space)"]
        NetInst["Networks (Rx/Tx bytes)"]
    end

    subgraph "common_lib Crate (Shared Core)"
        Mutex["check_single_instance (Named Mutex)"]
        Diff["compute_diff (LCS Process Diff)"]
        Count["count_occurrences (Filter Match Count)"]
    end

    subgraph "MyNMON Modules"
        State["state::MonitorState (toggles, filter, tick_rate)"]
        Draw["ui::draw_ui (UI Rendering Engine)"]
        Utils["utils (get_ascii_bar, pad_or_truncate, format_uptime)"]
    end

    subgraph "crossterm Crate (Rendering)"
        TermBuf["Terminal Alternate Buffer"]
    end

    %% Data Flow Connections
    ProcFS --> SysInst
    WinAPI --> SysInst
    macOS --> SysInst
    ProcFS --> DiskInst
    WinAPI --> DiskInst
    ProcFS --> NetInst
    WinAPI --> NetInst

    SysInst -->|Process List| Diff
    Diff -->|Spawn/Exit Log| State
    SysInst -->|Read Metrics| Draw
    DiskInst -->|Read Disk| Draw
    NetInst -->|Read I/O| Draw
    Count -->|Match Count| Draw
    State -->|Conditional Toggle| Draw
    Utils -->|Format & Bars| Draw
    Mutex -.->|Startup Guard| State

    Draw -->|crossterm::execute!| TermBuf
```
