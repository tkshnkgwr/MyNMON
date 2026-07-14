//! # MyNMON
//! 
//! `MyNMON` は、Windows および Linux 環境に対応した超軽量な CUI システムモニターです。
//! 伝統的な `nmon` にインスパイアされており、ターミナル上でリアルタイムにシステムの稼働状況を監視できます。
//! 
//! ## 主な機能
//! - CPU使用率の表示（全体およびコア個別）
//! - メモリおよびスワップ領域の割り当て状況表示
//! - ディスクマウント情報と空き容量の表示
//! - ネットワークインターフェースごとの送受信速度（I/O）表示
//! - CPU使用率順のプロセス一覧表示（名前でのフィルタリング機能付き）
//! - プロセスの起動・終了履歴のログ表示
//! - 画面更新間隔の動的変更

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

/// アプリケーションの監視状態や設定を管理する構造体。
struct MonitorState {
    /// 全体CPU使用率を表示するかどうか
    show_cpu_total: bool,
    /// 個々のCPUコア使用率を表示するかどうか
    show_cpu_cores: bool,
    /// メモリ使用状況を表示するかどうか
    show_mem: bool,
    /// ディスク空き容量を表示するかどうか
    show_disk: bool,
    /// ネットワーク速度を表示するかどうか
    show_net: bool,
    /// プロセス一覧を表示するかどうか
    show_proc: bool,
    /// プロセス起動・終了ログを表示するかどうか
    show_diff: bool,
    /// プロセス名フィルターのクエリ文字列
    filter_query: String,
    /// 現在プロセスフィルター入力モードであるかどうか
    is_filtering: bool,
    /// 前回のプロセス一覧の文字列表現（差分検出用）
    last_process_list: String,
    /// プロセスの起動（+）および終了（-）の履歴ログ
    spawn_exit_log: Vec<String>,
    /// 画面の更新レート（ティックレート）
    tick_rate: Duration,
    /// 現在更新間隔設定入力モードであるかどうか
    is_setting_interval: bool,
    /// 更新間隔入力用の一時文字列
    interval_input: String,
}

/// アプリケーションのエントリポイント。
/// 
/// 二重起動の防止、コマンドライン引数の解析、ターミナル設定の初期化、
/// およびメインのシステム監視イベントループの制御を行います。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Windows上での二重起動を防止
    if let Err(e) = common_lib::check_single_instance("MyNMON_NamedMutex_Instance", "MyNMON") {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // コマンドライン引数の解析
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

    // ターミナルの初期設定
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    let mut state = MonitorState {
        show_cpu_total: false,
        show_cpu_cores: false,
        show_mem: false,
        show_disk: false,
        show_net: false,
        show_proc: false,
        show_diff: false,
        filter_query: String::new(),
        is_filtering: false,
        last_process_list: String::new(),
        spawn_exit_log: Vec::new(),
        tick_rate: Duration::from_millis(1000),
        is_setting_interval: false,
        interval_input: String::new(),
    };

    let mut last_tick = Instant::now();

    // システム情報の初回更新
    sys.refresh_all();
    thread::sleep(Duration::from_millis(100));

    loop {
        // 各種メトリクスを更新
        sys.refresh_all();
        disks.refresh();
        networks.refresh();

        // common_lib::compute_diff を使用したプロセスの変更検知
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

        // ターミナルの描画実行
        draw_ui(&mut stdout, &sys, &disks, &networks, &state)?;

        // キーイベントのポーリング
        let timeout = state
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
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
                    } else if state.is_setting_interval {
                        match key.code {
                            KeyCode::Enter => {
                                if let Ok(secs) = state.interval_input.parse::<u64>() {
                                    if secs >= 1 {
                                        state.tick_rate = Duration::from_secs(secs);
                                    }
                                }
                                state.is_setting_interval = false;
                            }
                            KeyCode::Esc => {
                                state.is_setting_interval = false;
                            }
                            KeyCode::Backspace => {
                                state.interval_input.pop();
                            }
                            KeyCode::Char(c) if c.is_numeric() => {
                                state.interval_input.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('c') => state.show_cpu_cores = !state.show_cpu_cores,
                            KeyCode::Char('C') => state.show_cpu_total = !state.show_cpu_total,
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
                            KeyCode::Char('r') => {
                                state.interval_input = (state.tick_rate.as_secs()).to_string();
                                state.is_setting_interval = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= state.tick_rate {
            last_tick = Instant::now();
        }
    }

    // ターミナル状態の復元
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

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

/// システムの起動時間（秒）を、日・時間・分・秒の読みやすい形式の文字列にフォーマットする。
/// 
/// 1日以上の場合は `Xd HH:MM:SS`、1日未満の場合は `HH:MM:SS` となります。
fn format_uptime(uptime_secs: u64) -> String {
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    let seconds = uptime_secs % 60;
    if days > 0 {
        format!("{}d {:02}:{:02}:{:02}", days, hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// 全角・半角を考慮した文字幅（半角=1, 全角=2）で、文字列を指定された幅にパディングまたは切り詰める。
/// 
/// 指定幅を超えた部分は切り捨てられ、満たない部分はスペースで埋められます。
fn pad_or_truncate(s: &str, width: usize) -> String {
    let mut current_width = 0;
    let mut result = String::new();

    for c in s.chars() {
        let char_width = if c as u32 > 0x7f { 2 } else { 1 };
        if current_width + char_width > width {
            break;
        }
        result.push(c);
        current_width += char_width;
    }

    if current_width < width {
        result.push_str(&" ".repeat(width - current_width));
    }

    result
}

/// システム情報および監視状態に基づき、ターミナル画面全体を描画する。
/// 
/// ターミナルサイズが 80x20 未満の場合はエラーメッセージを表示します。
/// また、画面のちらつきを防止するため、全画面クリアではなくカーソルを左上に移動して上書き描画を行います。
fn draw_ui<W: Write>(
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
    let all_hidden = !state.show_cpu_total
        && !state.show_cpu_cores
        && !state.show_mem
        && !state.show_disk
        && !state.show_net
        && !state.show_proc
        && !state.show_diff;

    if all_hidden {
        draw_welcome_screen(w)?;
    }

    // 有効化された各セクションを描画
    if state.show_cpu_total {
        draw_cpu_total(w, sys)?;
    }
    if state.show_cpu_cores {
        draw_cpu_cores(w, sys)?;
    }
    if state.show_mem {
        draw_memory(w, sys)?;
    }
    if state.show_disk {
        draw_disk(w, disks)?;
    }
    if state.show_net {
        draw_network(w, networks)?;
    }
    if state.show_proc {
        draw_processes(w, sys, state)?;
    }
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
fn draw_header<W: Write>(w: &mut W, width: u16, tick_rate: Duration) -> io::Result<()> {
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
fn draw_help<W: Write>(w: &mut W) -> io::Result<()> {
    w_line!(
        w,
        " {} | {} | {} | {} to quit",
        "[C]:CPU-Total  [c]:CPU-Cores  [m]:Mem  [d]:Disk  [n]:Net  [p]:Proc  [g]:DiffLog".green(),
        "[f]:Filter".yellow().bold(),
        "[r]:Interval".cyan().bold(),
        "[q]".red().bold()
    )
}

/// プロセスフィルター入力、更新間隔変更入力、または現在のフィルター状態を表示するステータスバーを描画する。
fn draw_status_bar<W: Write>(
    w: &mut W,
    width: u16,
    sys: &System,
    state: &MonitorState,
) -> io::Result<()> {
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
    } else if state.is_setting_interval {
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
    } else if !state.filter_query.is_empty() {
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
    } else {
        w_line!(w, "{}", "=".repeat(width as usize).grey())?;
    }
    Ok(())
}

/// アプリケーション起動時など、すべての監視セクションが非表示の場合に表示される、
/// ウェルカム画面と詳細なキー操作ガイドを描画する。
fn draw_welcome_screen<W: Write>(w: &mut W) -> io::Result<()> {
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
    w_line!(
        w,
        "    {} : Toggle Total CPU utilization display",
        "C".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Individual CPU Core utilization display",
        "c".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Memory allocation display",
        "m".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Disk mounts & space display",
        "d".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Network interface speed display",
        "n".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Top processes display (also 't' key)",
        "p".cyan().bold()
    )?;
    w_line!(
        w,
        "    {} : Toggle Process Spawn/Exit history log (also 'l' key)",
        "g".cyan().bold()
    )?;
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
fn draw_cpu_total<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
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
fn draw_cpu_cores<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
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
fn draw_memory<W: Write>(w: &mut W, sys: &System) -> io::Result<()> {
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
fn draw_disk<W: Write>(w: &mut W, disks: &Disks) -> io::Result<()> {
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
fn draw_network<W: Write>(w: &mut W, networks: &Networks) -> io::Result<()> {
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
fn draw_processes<W: Write>(w: &mut W, sys: &System, state: &MonitorState) -> io::Result<()> {
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
fn draw_diff_log<W: Write>(w: &mut W, state: &MonitorState) -> io::Result<()> {
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

/// パーセンテージと幅に基づいて、ASCIIで表現されたプログレスバー（例: `[====>     ]`）を生成する。
/// 
/// # 引数
/// - `percent`: 割合（0.0 から 100.0 の範囲にクランプされます）
/// - `width`: バーの文字幅
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

/// `PID:プロセス名` 形式の文字列を解析し、`(PID, プロセス名)` のタプルを返すヘルパー関数。
fn parse_proc_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// アプリケーション起動時のコマンドラインヘルプ（`-h` または `--help` 指定時）を標準出力に表示する。
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
    println!("  C  Toggle Total CPU utilization display");
    println!("  c  Toggle Individual CPU Core utilization display");
    println!("  m  Toggle Memory allocation display");
    println!("  d  Toggle Disk mounts & space display");
    println!("  n  Toggle Network interface speed display");
    println!("  p  Toggle Top processes display (also 't' key)");
    println!("  g  Toggle Process Spawn/Exit history log (also 'l' key)");
    println!("  f  Search/Filter processes by name (Enter/Esc to exit search)");
    println!("  r  Set screen refresh interval in seconds (Enter/Esc to save/cancel)");
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
