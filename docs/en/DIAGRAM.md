# System Architecture Diagram (DIAGRAM.md)

**English** | [日本語版](../ja/DIAGRAM.md)

This document provides diagrams illustrating the threads, lifecycle, data collection pathways, and event control flow of `MyNMON`.

---

## 1. Application Lifecycle and Event Loop

`MyNMON` achieves low-latency rendering and rapid key event responses within a single thread by combining asynchronous polling from Crossterm with elapsed-time calculation.

```mermaid
sequenceDiagram
    autonumber
    actor User as User
    participant App as MyNMON (Main)
    participant Term as Terminal (crossterm)
    participant System as OS / Hardware (sysinfo)

    Note over App: Startup (fn main)
    App->>Term: Enable Raw Mode & Alternate Screen & Hide Cursor
    App->>System: Initialize and perform initial metrics refresh

    loop Event Loop (At configured refresh interval)
        App->>System: Refresh latest metrics (CPU, RAM, Disks, Networks, Processes)
        System-->>App: Metrics Data
        App->>Term: Render UI (draw_ui)
        Term-->>User: Update Screen Display

        Note over App: Calculate Poll Timeout<br/>(configured interval - elapsed loop time)
        
        alt Key input detected within timeout
            User->>Term: Press Key
            Term->>App: Detect Key Event (event::poll)
            alt 'r' key (Change Refresh Interval)
                App->>App: Start Input Mode (is_setting_interval = true)
                User->>Term: Enter seconds + Enter
                Term->>App: Key Event (Enter)
                App->>App: Update tick_rate and exit input mode
            else Other toggle keys
                App->>App: Toggle display state (MonitorState)
            end
        else Timeout exceeded
            Note over App: Proceed to next loop iteration
        end
        
        alt 'q' or 'Esc' pressed
            App->>Term: Disable Raw Mode & Restore Normal Screen & Show Cursor
            Note over App: Terminate Process (exit 0)
            App->>User: Return Terminal Control
        end
    end
```

---

## 2. Data Flow and Rendering Path

The following diagram illustrates how metrics collected from the OS via `sysinfo` are processed and outputted to the terminal buffer:

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

    subgraph "MyNMON Modules"
        State["state::MonitorState (show_cpu, show_mem, etc.)"]
        Draw["ui::draw_ui (UI Rendering Engine)"]
        AscBar["utils::get_ascii_bar (ASCII Bar Engine)"]
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

    SysInst -->|Read Metrics| Draw
    DiskInst -->|Read Disk| Draw
    NetInst -->|Read I/O| Draw
    State -->|Conditional Toggle| Draw
    AscBar -->|Generate [===> ]| Draw

    Draw -->|crossterm::execute!| TermBuf
```
