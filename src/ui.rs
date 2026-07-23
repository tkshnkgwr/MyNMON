use crate::state::MonitorState;
#[allow(unused_imports)]
use crate::utils::{format_uptime, get_ascii_bar, pad_or_truncate};
use crossterm::{cursor, execute, style::Stylize, terminal};
use std::io::{self, Write};
use std::time::Duration;
use sysinfo::{Disks, Networks, System};

/// 指定したライター（標準出力等）へフォーマットされた文字列を出力し、
/// 行末までをクリアしたうえで改行するヘルパーマクロ。
macro_rules! w_line {
    ($w:expr, $($arg:tt)*) => {
        {
            write!($w, $($arg)*)?;
            crossterm::queue!($w, crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine))?;
            writeln!($w)
        }
    };
}

/// システム情報および監視状態に基づき、ターミナル画面全体を描画する。
///
/// ターミナルサイズが 80x20 未満の場合はエラーメッセージを表示します。
/// また、画面のちらつきを防止するため、全画面クリアではなくカーソルを左上に移動して上書き描画を行います。
#[allow(unused_variables)]
pub fn draw_ui<W: Write>(
    w: &mut W,
    sys: &System,
    disks: &Disks,
    networks: &Networks,
    state: &MonitorState,
) -> io::Result<()> {
    // ターミナルサイズを最初にチェック（画面崩れおよび折り返し防止）
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 20));
    if width < 80 || height < 20 {
        execute!(w, cursor::MoveTo(0, 0))?;
        w_line!(w, "{}", "--- Terminal Size Error ---".bold().red())?;
        w_line!(w, "Current size: {}x{}", width, height)?;
        w_line!(w, "Please resize your terminal to at least 80x20.")?;
        execute!(w, terminal::Clear(terminal::ClearType::FromCursorDown))?;
        w_line!(w, "")?;
        w.flush()?;
        return Ok(());
    }

    // 画面全体をクリアする代わりにカーソルを左上に移動（ちらつき防止）
    execute!(w, cursor::MoveTo(0, 0))?;

    // 基本的なUIフレームを描画
    draw_header(w, width, state.tick_rate)?;
    draw_help(w)?;
    draw_status_bar(w, width, sys, state)?;

    // すべてのセクションが非表示の場合はウェルカム/ガイド画面を表示
    #[allow(unused_mut)]
    let mut all_hidden = true;
    #[cfg(feature = "cpu")]
    if state.show_cpu_total || state.show_cpu_cores {
        all_hidden = false;
    }
    #[cfg(feature = "mem")]
    if state.show_mem {
        all_hidden = false;
    }
    #[cfg(feature = "disk")]
    if state.show_disk {
        all_hidden = false;
    }
    #[cfg(feature = "net")]
    if state.show_net {
        all_hidden = false;
    }
    #[cfg(feature = "proc")]
    if state.show_proc {
        all_hidden = false;
    }
    #[cfg(feature = "diff")]
    if state.show_diff {
        all_hidden = false;
    }

    if all_hidden {
        draw_welcome_screen(w)?;
    }

    // 有効化された各セクションを描画
    #[cfg(feature = "cpu")]
    if state.show_cpu_total {
        draw_cpu_total(w, sys)?;
    }
    #[cfg(feature = "cpu")]
    if state.show_cpu_cores {
        draw_cpu_cores(w, sys)?;
    }
    #[cfg(feature = "mem")]
    if state.show_mem {
        draw_memory(w, sys)?;
    }
    #[cfg(feature = "disk")]
    if state.show_disk {
        draw_disk(w, disks)?;
    }
    #[cfg(feature = "net")]
    if state.show_net {
        draw_network(w, networks)?;
    }
    #[cfg(feature = "proc")]
    if state.show_proc {
        draw_processes(w, sys, state)?;
    }
    #[cfg(feature = "diff")]
    if state.show_diff {
        draw_diff_log(w, state)?;
    }

    // 前のフレームの残存行をクリア
    execute!(w, terminal::Clear(terminal::ClearType::FromCursorDown))?;

    w.flush()?;
    Ok(())
}

/// ホスト名、OS名、カーネルバージョン、アップタイム、更新間隔を表示するヘッダー部を描画する。
///
/// ターミナル幅（`width`）に応じて表示する情報を調整し、行の折り返しを防ぎます。
pub fn draw_header<W: Write>(w: &mut W, width: u16, tick_rate: Duration) -> io::Result<()> {
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime_secs = System::uptime();
    let uptime_str = format_uptime(uptime_secs);
    let interval_secs = tick_rate.as_secs();

    let version = env!("CARGO_PKG_VERSION");
    let header_title = format!(" MyNMON v{} ", version);

    // 折り返しを防ぐため、ターミナル幅に応じたレスポンシブなヘッダー表示
    let header_str = if width >= 105 {
        format!(
            "{} | Host: {} | OS: {} | Kernel: {} | Uptime: {} | Interval: {}s",
            header_title.bold().black().on_green(),
            hostname.cyan(),
            os_name.yellow(),
            kernel.magenta(),
            uptime_str.green(),
            interval_secs.to_string().yellow().bold()
        )
    } else if width >= 90 {
        format!(
            "{} | Host: {} | OS: {} | Uptime: {} | Interval: {}s",
            header_title.bold().black().on_green(),
            hostname.cyan(),
            os_name.yellow(),
            uptime_str.green(),
            interval_secs.to_string().yellow().bold()
        )
    } else {
        format!(
            "{} | Host: {} | Uptime: {} | Interval: {}s",
            header_title.bold().black().on_green(),
            hostname.cyan(),
            uptime_str.green(),
            interval_secs.to_string().yellow().bold()
        )
    };
    w_line!(w, "{}", header_str)?;
    w_line!(w, "{}", "-".repeat(width as usize).dark_grey())?;
    Ok(())
}

/// 操作可能なキーショートカットの案内（ヘルプ行）を描画する。
pub fn draw_help<W: Write>(w: &mut W) -> io::Result<()> {
    let mut shortcuts: Vec<String> = Vec::new();
    #[cfg(feature = "cpu")]
    shortcuts.push("[C]:CPU-Total  [c]:CPU-Cores".green().to_string());
    #[cfg(feature = "mem")]
    shortcuts.push("[m]:Mem".green().to_string());
    #[cfg(feature = "disk")]
    shortcuts.push("[d]:Disk".green().to_string());
    #[cfg(feature = "net")]
    shortcuts.push("[n]:Net".green().to_string());
    #[cfg(feature = "proc")]
    shortcuts.push("[p]:Proc".green().to_string());
    #[cfg(feature = "diff")]
    shortcuts.push("[g]:DiffLog".green().to_string());

    let shortcuts_str = if shortcuts.is_empty() {
        String::new()
    } else {
        format!("{} | ", shortcuts.join("  "))
    };

    let filter_str = if cfg!(any(feature = "proc", feature = "diff")) {
        format!("{} | ", "[f]:Filter".yellow().bold())
    } else {
        String::new()
    };

    w_line!(
        w,
        " {}{}{}{} to quit",
        shortcuts_str,
        filter_str,
        "[r]:Interval".cyan().bold(),
        "  [q]".red().bold()
    )
}

/// プロセスフィルター入力、更新間隔変更入力、または現在のフィルター状態を表示するステータスバーを描画する。
#[allow(unused_variables)]
pub fn draw_status_bar<W: Write>(
    w: &mut W,
    width: u16,
    sys: &System,
    state: &MonitorState,
) -> io::Result<()> {
    #[cfg(any(feature = "proc", feature = "diff"))]
    if state.is_filtering {
        w_line!(
            w,
            "{} {}",
            " FILTER INPUT (Enter/Esc to close): "
                .bold()
                .black()
                .on_yellow(),
            state.filter_query.clone().underlined()
        )?;
        w_line!(w, "{}", "-".repeat(width as usize).dark_grey())?;
        return Ok(());
    }

    if state.is_setting_interval {
        w_line!(
            w,
            "{} {}",
            " SET INTERVAL (seconds, Enter/Esc): "
                .bold()
                .black()
                .on_cyan(),
            state.interval_input.clone().underlined()
        )?;
        w_line!(w, "{}", "-".repeat(width as usize).dark_grey())?;
        return Ok(());
    }

    #[cfg(any(feature = "proc", feature = "diff"))]
    if !state.filter_query.is_empty() {
        // すべてのプロセス名からマッチする件数をカウント
        let all_proc_names = sys
            .processes()
            .values()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let matches_count = common_lib::count_occurrences(&all_proc_names, &state.filter_query);

        w_line!(
            w,
            "{} {} | Matches: {}",
            " Filter Active: ".bold().black().on_cyan(),
            state.filter_query.clone().cyan().underlined(),
            matches_count.to_string().yellow().bold()
        )?;
        w_line!(w, "{}", "-".repeat(width as usize).dark_grey())?;
        return Ok(());
    }

    w_line!(w, "{}", "=".repeat(width as usize).grey())?;
    Ok(())
}

/// アプリケーション起動時など、すべての監視セクションが非表示の場合に表示される、
/// ウェルカム画面と詳細なキー操作ガイドを描画する。
pub fn draw_welcome_screen<W: Write>(w: &mut W) -> io::Result<()> {
    w_line!(w, "")?;
    w_line!(w, "  {}", "Welcome to MyNMON!".bold().green())?;
    w_line!(
        w,
        "  {}",
        "A lightweight, cross-platform CLI system monitor inspired by nmon."
    )?;
    w_line!(
        w,
        "  {}",
        "It monitors CPU, Memory, Disk, Network, and Processes in real-time."
    )?;
    w_line!(w, "")?;
    w_line!(
        w,
        "  {}",
        "--- Interactive Keys to Show Sections ---".bold().yellow()
    )?;
    #[cfg(feature = "cpu")]
    w_line!(
        w,
        "    {} : Toggle Total CPU utilization display",
        "C".cyan().bold()
    )?;
    #[cfg(feature = "cpu")]
    w_line!(
        w,
        "    {} : Toggle Individual CPU Core utilization display",
        "c".cyan().bold()
    )?;
    #[cfg(feature = "mem")]
    w_line!(
        w,
        "    {} : Toggle Memory allocation display",
        "m".cyan().bold()
    )?;
    #[cfg(feature = "disk")]
    w_line!(
        w,
        "    {} : Toggle Disk mounts & space display",
        "d".cyan().bold()
    )?;
    #[cfg(feature = "net")]
    w_line!(
        w,
        "    {} : Toggle Network interface speed display",
        "n".cyan().bold()
    )?;
    #[cfg(feature = "proc")]
    w_line!(
        w,
        "    {} : Toggle Top processes display (also 't' key)",
        "p".cyan().bold()
    )?;
    #[cfg(feature = "diff")]
    w_line!(
        w,
        "    {} : Toggle Process Spawn/Exit history log (also 'l' key)",
        "g".cyan().bold()
    )?;
    #[cfg(any(feature = "proc", feature = "diff"))]
    w_line!(
        w,
        "    {} : Search/Filter processes by name (Enter/Esc to exit)",
        "f".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Set screen refresh interval in seconds",
        "r".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Quit the application (also 'Esc' key)",
        "q".cyan().bold()
    )?;
    w_line!(w, "")?;
    w_line!(
        w,
        "  {}",
        "Press any key above to start monitoring."
            .italic()
            .dark_grey()
    )?;
    w_line!(w, "")?;
    Ok(())
}

/// 全体のCPU使用率を取得し、ASCIIのプログレスバーを交えて描画する。
#[cfg(feature = "cpu")]
pub fn draw_cpu_total<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
    let global_cpu = sys.global_cpu_info();
    let load = global_cpu.cpu_usage();
    let bar = get_ascii_bar(load as f64, 25);
    w_line!(w, "{}", "--- CPU Utilization (Total) ---".bold().cyan())?;
    w_line!(
        w,
        "  Total CPU: {:5.1}% {}",
        load,
        if load > 80.0 {
            bar.red()
        } else if load > 40.0 {
            bar.yellow()
        } else {
            bar.green()
        }
    )?;
    w_line!(w, "")?;
    Ok(())
}

/// 個々のCPUコアの使用率を個別に取得し、それぞれASCIIプログレスバーとともに描画する。
#[cfg(feature = "cpu")]
pub fn draw_cpu_cores<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
    w_line!(
        w,
        "{}",
        "--- CPU Utilization (Individual Cores) ---".bold().cyan()
    )?;
    for (i, cpu) in sys.cpus().iter().enumerate() {
        let load = cpu.cpu_usage();
        let bar_width = 25;
        let bar = get_ascii_bar(load as f64, bar_width);
        w_line!(
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
    w_line!(w, "")?;
    Ok(())
}

/// 物理メモリ（RAM）およびスワップ領域（Windowsの場合はページファイル）の
/// 総量、使用量、空き容量を取得し、ASCIIバーとあわせて描画する。
#[cfg(feature = "mem")]
pub fn draw_memory<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
    let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let free_mem = sys.free_memory() as f64 / 1024.0 / 1024.0;
    let mem_pct = (used_mem / total_mem) * 100.0;
    let mem_bar = get_ascii_bar(mem_pct, 40);

    let total_swap = sys.total_swap() as f64 / 1024.0 / 1024.0;
    let used_swap = sys.used_swap() as f64 / 1024.0 / 1024.0;
    let free_swap = total_swap - used_swap;
    let swap_pct = if total_swap > 0.0 {
        (used_swap / total_swap) * 100.0
    } else {
        0.0
    };
    let swap_bar = get_ascii_bar(swap_pct, 40);

    w_line!(w, "{}", "--- Memory Allocation ---".bold().magenta())?;
    w_line!(
        w,
        "  Physical RAM : {:.2} GB Total | {:.2} GB Used | {:.2} GB Free",
        total_mem / 1024.0,
        used_mem / 1024.0,
        free_mem / 1024.0
    )?;
    w_line!(
        w,
        "  RAM Usage    : {:5.1}% {}",
        mem_pct,
        if mem_pct > 85.0 {
            mem_bar.red()
        } else {
            mem_bar.magenta()
        }
    )?;
    w_line!(
        w,
        "  Swap/Pagefile: {:.2} GB Total | {:.2} GB Used | {:.2} GB Free",
        total_swap / 1024.0,
        used_swap / 1024.0,
        free_swap / 1024.0
    )?;
    w_line!(
        w,
        "  Swap Usage   : {:5.1}% {}",
        swap_pct,
        if swap_pct > 85.0 {
            swap_bar.red()
        } else {
            swap_bar.magenta()
        }
    )?;
    w_line!(w, "")?;
    Ok(())
}

/// マウントされている各ディスクのファイルシステム、空き容量、総量、および使用率を描画する。
#[cfg(feature = "disk")]
pub fn draw_disk<W: Write>(w: &mut W, disks: &Disks) -> io::Result<()> {
    w_line!(w, "{}", "--- Disk Mounts & Space ---".bold().yellow())?;
    for disk in disks.list() {
        let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = total - available;
        let usage_pct = (used / total) * 100.0;
        w_line!(
            w,
            "  {:<12} ({:?}): {:.1}GB / {:.1}GB free ({:.1}% used)",
            disk.mount_point().to_string_lossy(),
            disk.file_system(),
            available,
            total,
            usage_pct
        )?;
    }
    w_line!(w, "")?;
    Ok(())
}

/// 各ネットワークインターフェースのデータの受信速度（Rx）および送信速度（Tx）をKB/s単位で描画する。
#[cfg(feature = "net")]
pub fn draw_network<W: Write>(w: &mut W, networks: &Networks) -> io::Result<()> {
    w_line!(
        w,
        "{}",
        "--- Network Interface I/O speeds ---".bold().blue()
    )?;
    let mut net_list: Vec<_> = networks.iter().collect();
    net_list.sort_by(|a, b| a.0.cmp(b.0));

    for (interface_name, data) in net_list {
        let rx = data.received() as f64 / 1024.0; // KB/秒
        let tx = data.transmitted() as f64 / 1024.0; // KB/秒
        let name_fixed = pad_or_truncate(interface_name, 16);
        w_line!(
            w,
            "  {} : Rx: {:8.2} KB/s | Tx: {:8.2} KB/s",
            name_fixed,
            rx,
            tx
        )?;
    }
    w_line!(w, "")?;
    Ok(())
}

/// プロセス一覧をCPU使用率順にソートし、上位8プロセスを表示する。
///
/// フィルタークエリが設定されている場合は、プロセス名にクエリが含まれるもののみを抽出します。
#[cfg(feature = "proc")]
pub fn draw_processes<W: Write>(w: &mut W, sys: &System, state: &MonitorState) -> io::Result<()> {
    w_line!(
        w,
        "{}",
        "--- Top Active Processes by CPU Usage ---"
            .bold()
            .dark_grey()
    )?;
    let mut processes: Vec<_> = sys.processes().values().collect();

    // プロセス名でフィルタリング
    if !state.filter_query.is_empty() {
        let query = state.filter_query.to_lowercase();
        processes.retain(|p| p.name().to_lowercase().contains(&query));
    }

    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

    w_line!(
        w,
        "  {:>6} {:<20} {:>10} {:>14}",
        "PID",
        "Process Name",
        "CPU %",
        "Memory (MB)"
    )?;
    for proc in processes.iter().take(8) {
        let mem_mb = proc.memory() as f64 / 1024.0 / 1024.0;
        let name_fixed = pad_or_truncate(proc.name(), 20);
        let cpu_val = format!("{:.1}%", proc.cpu_usage());
        let mem_val = format!("{:.1} MB", mem_mb);
        let pid_val = proc.pid().to_string();
        w_line!(
            w,
            "  {:>6} {} {:>10} {:>14}",
            pid_val,
            name_fixed,
            cpu_val,
            mem_val
        )?;
    }
    w_line!(w, "")?;
    Ok(())
}

/// プロセスの起動（+）および終了（-）の履歴ログを直近10件表示する。
///
/// フィルタークエリが設定されている場合は、ログ文字列にクエリが含まれるもののみを表示します。
#[cfg(feature = "diff")]
pub fn draw_diff_log<W: Write>(w: &mut W, state: &MonitorState) -> io::Result<()> {
    w_line!(
        w,
        "{}",
        "--- Process Spawn/Exit History Log ---".bold().magenta()
    )?;
    if state.spawn_exit_log.is_empty() {
        w_line!(w, "  No process changes detected yet.")?;
    } else {
        // ログに対してもフィルターを適用
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
            w_line!(w, "  No changes matching filter query.")?;
        } else {
            // 最新10件のログを表示
            let start_idx = filtered_logs.len().saturating_sub(10);
            for log in &filtered_logs[start_idx..] {
                if log.starts_with('+') {
                    w_line!(w, "  {}", log.clone().green())?;
                } else {
                    w_line!(w, "  {}", log.clone().red())?;
                }
            }
        }
    }
    w_line!(w, "")?;
    Ok(())
}
