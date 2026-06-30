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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

        // Draw terminal
        draw_ui(&mut stdout, &sys, &disks, &networks, &state)?;

        // Key event polling
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') => state.show_cpu = !state.show_cpu,
                    KeyCode::Char('m') => state.show_mem = !state.show_mem,
                    KeyCode::Char('d') => state.show_disk = !state.show_disk,
                    KeyCode::Char('n') => state.show_net = !state.show_net,
                    KeyCode::Char('p') | KeyCode::Char('t') => state.show_proc = !state.show_proc,
                    _ => {}
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
    execute!(w, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

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
        " {} (Toggle display Sections) | {} to quit",
        "[c]:CPU  [m]:Mem  [d]:Disk  [n]:Net  [p]:Process".green(),
        "[q]".red().bold()
    )?;
    writeln!(w, "{}", "=".repeat(80).grey())?;

    // CPU Section
    if state.show_cpu {
        writeln!(w, "{}", "--- CPU Utilization (Individual Cores) ---".bold().cyan())?;
        for (i, cpu) in sys.cpus().iter().enumerate() {
            let load = cpu.cpu_usage();
            let bar_width = 25;
            let bar = get_ascii_bar(load as f64, bar_width);
            writeln!(
                w,
                "  Core {:2}: {:5.1}% {}",
                i,
                load,
                if load > 80.0 { bar.red() } else if load > 40.0 { bar.yellow() } else { bar.green() }
            )?;
        }
        writeln!(w, "")?;
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
            if mem_pct > 85.0 { bar.red() } else { bar.magenta() }
        )?;
        writeln!(w, "")?;
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
        writeln!(w, "")?;
    }

    // Network Section
    if state.show_net {
        writeln!(w, "{}", "--- Network Interface I/O speeds ---".bold().blue())?;
        for (interface_name, data) in networks.iter() {
            let rx = data.received() as f64 / 1024.0; // KB/s
            let tx = data.transmitted() as f64 / 1024.0; // KB/s
            writeln!(
                w,
                "  {:<12} Rx: {:8.2} KB/s | Tx: {:8.2} KB/s",
                interface_name, rx, tx
            )?;
        }
        writeln!(w, "")?;
    }

    // Process Section
    if state.show_proc {
        writeln!(w, "{}", "--- Top Active Processes by CPU Usage ---".bold().dark_grey())?;
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        writeln!(w, "  {:>6} {:<18} {:>10} {:>12}", "PID", "Process Name", "CPU %", "Memory (MB)")?;
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
    }

    w.flush()?;
    Ok(())
}

fn get_ascii_bar(percent: f64, width: usize) -> String {
    let pct = percent.max(0.0).min(100.0);
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    if filled == 0 {
        format!("[{}]", " ".repeat(width))
    } else if filled >= width {
        format!("[{}>]", "=".repeat(width - 1))
    } else {
        format!("[{}>{}]", "=".repeat(filled - 1), " ".repeat(width - filled))
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
