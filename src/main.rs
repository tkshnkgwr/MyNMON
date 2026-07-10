use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::Stylize,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Disks, Networks, System};

struct MonitorState {
    show_cpu: bool,
    show_mem: bool,
    show_disk: bool,
    show_net: bool,
    show_proc: bool,
    show_diff: bool,
    filter_query: String,
    is_filtering: bool,
    last_process_list: String,
    spawn_exit_log: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prevent double launch on Windows
    if let Err(e) = common_lib::check_single_instance("MyNMON_NamedMutex_Instance", "MyNMON") {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => {
                println!("MyNMON v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            other => {
                eprintln!("Error: Unknown option '{}'", other);
                eprintln!("Usage: {} [-h | --help] [-v | --version]", args[0]);
                std::process::exit(1);
            }
        }
    }

    // Terminal setup
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    let mut state = MonitorState {
        show_cpu: true,
        show_mem: true,
        show_disk: true,
        show_net: true,
        show_proc: true,
        show_diff: true,
        filter_query: String::new(),
        is_filtering: false,
        last_process_list: String::new(),
        spawn_exit_log: Vec::new(),
    };

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(1000);

    // Initial system refresh
    sys.refresh_all();
    thread::sleep(Duration::from_millis(100));

    loop {
        // Refresh metrics
        sys.refresh_all();
        disks.refresh();
        networks.refresh();

        // Process change detection using common_lib::compute_diff
        {
            let mut current_processes: Vec<_> = sys.processes().values().collect();
            current_processes.sort_by_key(|p| p.pid().as_u32());
            let current_proc_str = current_processes
                .iter()
                .map(|p| format!("{}:{}", p.pid(), p.name()))
                .collect::<Vec<_>>()
                .join("\n");

            if state.last_process_list.is_empty() {
                state.last_process_list = current_proc_str;
            } else {
                let diffs = common_lib::compute_diff(&state.last_process_list, &current_proc_str);
                for diff in diffs {
                    match diff.diff_type {
                        common_lib::DiffType::Added => {
                            if let Some((pid, name)) = parse_proc_line(&diff.value) {
                                state
                                    .spawn_exit_log
                                    .push(format!("+ {} (PID: {})", name, pid));
                            }
                        }
                        common_lib::DiffType::Removed => {
                            if let Some((pid, name)) = parse_proc_line(&diff.value) {
                                state
                                    .spawn_exit_log
                                    .push(format!("- {} (PID: {})", name, pid));
                            }
                        }
                        common_lib::DiffType::Unchanged => {}
                    }
                }
                if state.spawn_exit_log.len() > 50 {
                    let drain_len = state.spawn_exit_log.len() - 50;
                    state.spawn_exit_log.drain(0..drain_len);
                }
                state.last_process_list = current_proc_str;
            }
        }

        // Draw terminal
        draw_ui(&mut stdout, &sys, &disks, &networks, &state)?;

        // Key event polling
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if state.is_filtering {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            state.is_filtering = false;
                        }
                        KeyCode::Backspace => {
                            state.filter_query.pop();
                        }
                        KeyCode::Char(c) => {
                            state.filter_query.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') => state.show_cpu = !state.show_cpu,
                        KeyCode::Char('m') => state.show_mem = !state.show_mem,
                        KeyCode::Char('d') => state.show_disk = !state.show_disk,
                        KeyCode::Char('n') => state.show_net = !state.show_net,
                        KeyCode::Char('p') | KeyCode::Char('t') => {
                            state.show_proc = !state.show_proc
                        }
                        KeyCode::Char('g') | KeyCode::Char('l') => {
                            state.show_diff = !state.show_diff
                        }
                        KeyCode::Char('f') => {
                            state.is_filtering = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_ui<W: Write>(
    w: &mut W,
    sys: &System,
    disks: &Disks,
    networks: &Networks,
    state: &MonitorState,
) -> io::Result<()> {
    // Clear terminal screen
    execute!(
        w,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    // Draw header
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    let version = env!("CARGO_PKG_VERSION");
    let header_title = format!(" MyNMON v{} ", version);
    writeln!(
        w,
        "{} | Host: {} | OS: {} | Kernel: {}",
        header_title.bold().black().on_green(),
        hostname.cyan(),
        os_name.yellow(),
        kernel.magenta()
    )?;
    writeln!(w, "{}", "-".repeat(80).dark_grey())?;

    // Help line
    writeln!(
        w,
        " {} | {} | {} to quit",
        "[c]:CPU  [m]:Mem  [d]:Disk  [n]:Net  [p]:Proc  [g]:DiffLog".green(),
        "[f]:Filter".yellow().bold(),
        "[q]".red().bold()
    )?;

    // Filter indicator
    if state.is_filtering {
        writeln!(
            w,
            "{} {}",
            " FILTER INPUT (Enter/Esc to close): "
                .bold()
                .black()
                .on_yellow(),
            state.filter_query.clone().underlined()
        )?;
        writeln!(w, "{}", "-".repeat(80).dark_grey())?;
    } else if !state.filter_query.is_empty() {
        // count_occurrences to find matches across all process names
        let all_proc_names = sys
            .processes()
            .values()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let matches_count = common_lib::count_occurrences(&all_proc_names, &state.filter_query);

        writeln!(
            w,
            "{} {} | Matches: {}",
            " Filter Active: ".bold().black().on_cyan(),
            state.filter_query.clone().cyan().underlined(),
            matches_count.to_string().yellow().bold()
        )?;
        writeln!(w, "{}", "-".repeat(80).dark_grey())?;
    } else {
        writeln!(w, "{}", "=".repeat(80).grey())?;
    }

    // CPU Section
    if state.show_cpu {
        writeln!(
            w,
            "{}",
            "--- CPU Utilization (Individual Cores) ---".bold().cyan()
        )?;
        for (i, cpu) in sys.cpus().iter().enumerate() {
            let load = cpu.cpu_usage();
            let bar_width = 25;
            let bar = get_ascii_bar(load as f64, bar_width);
            writeln!(
                w,
                "  Core {:2}: {:5.1}% {}",
                i,
                load,
                if load > 80.0 {
                    bar.red()
                } else if load > 40.0 {
                    bar.yellow()
                } else {
                    bar.green()
                }
            )?;
        }
        writeln!(w)?;
    }

    // Memory Section
    if state.show_mem {
        let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0;
        let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0;
        let free_mem = sys.free_memory() as f64 / 1024.0 / 1024.0;
        let mem_pct = (used_mem / total_mem) * 100.0;
        let bar = get_ascii_bar(mem_pct, 40);

        writeln!(w, "{}", "--- Memory Allocation ---".bold().magenta())?;
        writeln!(
            w,
            "  Physical RAM: {:.2} GB Total | {:.2} GB Used | {:.2} GB Free",
            total_mem / 1024.0,
            used_mem / 1024.0,
            free_mem / 1024.0
        )?;
        writeln!(
            w,
            "  Memory Usage: {:5.1}% {}",
            mem_pct,
            if mem_pct > 85.0 {
                bar.red()
            } else {
                bar.magenta()
            }
        )?;
        writeln!(w)?;
    }

    // Disk Section
    if state.show_disk {
        writeln!(w, "{}", "--- Disk Mounts & Space ---".bold().yellow())?;
        for disk in disks.list() {
            let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used = total - available;
            let usage_pct = (used / total) * 100.0;
            writeln!(
                w,
                "  {:<12} ({:?}): {:.1}GB / {:.1}GB free ({:.1}% used)",
                disk.mount_point().to_string_lossy(),
                disk.file_system(),
                available,
                total,
                usage_pct
            )?;
        }
        writeln!(w)?;
    }

    // Network Section
    if state.show_net {
        writeln!(
            w,
            "{}",
            "--- Network Interface I/O speeds ---".bold().blue()
        )?;
        for (interface_name, data) in networks.iter() {
            let rx = data.received() as f64 / 1024.0; // KB/s
            let tx = data.transmitted() as f64 / 1024.0; // KB/s
            writeln!(
                w,
                "  {:<12} Rx: {:8.2} KB/s | Tx: {:8.2} KB/s",
                interface_name, rx, tx
            )?;
        }
        writeln!(w)?;
    }

    // Process Section
    if state.show_proc {
        writeln!(
            w,
            "{}",
            "--- Top Active Processes by CPU Usage ---"
                .bold()
                .dark_grey()
        )?;
        let mut processes: Vec<_> = sys.processes().values().collect();

        // Filter processes by name
        if !state.filter_query.is_empty() {
            let query = state.filter_query.to_lowercase();
            processes.retain(|p| p.name().to_lowercase().contains(&query));
        }

        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        writeln!(
            w,
            "  {:>6} {:<18} {:>10} {:>12}",
            "PID", "Process Name", "CPU %", "Memory (MB)"
        )?;
        for proc in processes.iter().take(8) {
            let mem_mb = proc.memory() as f64 / 1024.0 / 1024.0;
            writeln!(
                w,
                "  {:>6} {:<18} {:>9.1}% {:>10.1} MB",
                proc.pid(),
                proc.name(),
                proc.cpu_usage(),
                mem_mb
            )?;
        }
        writeln!(w)?;
    }

    // Diff Log Section
    if state.show_diff {
        writeln!(
            w,
            "{}",
            "--- Process Spawn/Exit History Log ---".bold().magenta()
        )?;
        if state.spawn_exit_log.is_empty() {
            writeln!(w, "  No process changes detected yet.")?;
        } else {
            // Apply filter to logs as well
            let filtered_logs: Vec<String> = if !state.filter_query.is_empty() {
                let query = state.filter_query.to_lowercase();
                state
                    .spawn_exit_log
                    .iter()
                    .filter(|log| log.to_lowercase().contains(&query))
                    .cloned()
                    .collect()
            } else {
                state.spawn_exit_log.clone()
            };

            if filtered_logs.is_empty() {
                writeln!(w, "  No changes matching filter query.")?;
            } else {
                // Show last 10 logs
                let start_idx = filtered_logs.len().saturating_sub(10);
                for log in &filtered_logs[start_idx..] {
                    if log.starts_with('+') {
                        writeln!(w, "  {}", log.clone().green())?;
                    } else {
                        writeln!(w, "  {}", log.clone().red())?;
                    }
                }
            }
        }
        writeln!(w)?;
    }

    w.flush()?;
    Ok(())
}

fn get_ascii_bar(percent: f64, width: usize) -> String {
    let pct = percent.clamp(0.0, 100.0);
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    if filled == 0 {
        format!("[{}]", " ".repeat(width))
    } else if filled >= width {
        format!("[{}>]", "=".repeat(width - 1))
    } else {
        format!(
            "[{}>{}]",
            "=".repeat(filled - 1),
            " ".repeat(width - filled)
        )
    }
}

fn parse_proc_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn print_help() {
    println!("MyNMON v{}", env!("CARGO_PKG_VERSION"));
    println!("A lightweight, cross-platform CLI system monitor inspired by nmon.");
    println!();
    println!("Usage:");
    println!("  MyNMON [options]");
    println!();
    println!("Options:");
    println!("  -h, --help     Print this help message");
    println!("  -v, --version  Print version information");
    println!();
    println!("Interactive Keys (while running):");
    println!("  c  Toggle CPU Core utilization display");
    println!("  m  Toggle Memory allocation display");
    println!("  d  Toggle Disk mounts & space display");
    println!("  n  Toggle Network interface speed display");
    println!("  p  Toggle Top processes display (also 't' key)");
    println!("  g  Toggle Process Spawn/Exit history log (also 'l' key)");
    println!("  f  Search/Filter processes by name (Enter/Esc to exit search)");
    println!("  q  Quit the application (also 'Esc' key)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ascii_bar() {
        assert_eq!(get_ascii_bar(0.0, 10), "[          ]");
        assert_eq!(get_ascii_bar(50.0, 10), "[====>     ]");
        assert_eq!(get_ascii_bar(100.0, 10), "[=========>]");
    }
}
