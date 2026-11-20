//! # MyNMON
//!
//! `MyNMON` は、Windows および Linux 環境に対応した超軽量な CUI システムモニターです。
//! 伝統的な `nmon` にインスパイアされており、ターミナル上でリアルタイムにシステムの稼働状況を監視できます。
//!
//! ## 主な機能
//! - CPU使用率の表示（全体およびコア個別）
//! - メモリおよびスワップ領域の割り当て状況表示
//! - ディスクマウント情報と空き容量の表示
//! - ネットワーク速度の表示
//! - CPU使用率順のプロセス一覧表示（名前でのフィルタリング機能付き）
//! - プロセスの起動・終了履歴のログ表示
//! - 画面更新間隔の動的変更

mod state;
mod ui;
mod utils;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io, thread,
    time::{Duration, Instant},
};
use sysinfo::{Disks, Networks, System};

use state::MonitorState;

/// ターミナルの raw モードおよびオルタネートスクリーンを管理する RAII ガード。
///
/// 正常終了時はもちろん、エラーによる早期リターン時にも自動的にターミナル状態を復元します。
struct TerminalGuard;

impl TerminalGuard {
    /// raw モードを有効化し、オルタネートスクリーンへ切り替えてカーソルを非表示にします。
    fn enter(stdout: &mut io::Stdout) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
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
                println!(
                    "MyNMON v{}\nCopyright (c) 2026 tkshnkgwr\nLicense: GPL-3.0-or-later",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(());
            }
            "-h" | "--help" => {
                utils::print_help();
                return Ok(());
            }
            other => {
                eprintln!("Error: Unknown option '{}'", other);
                eprintln!("Usage: {} [-h | --help] [-v | --version]", args[0]);
                std::process::exit(1);
            }
        }
    }

    // システム情報の初期化（ターミナル切り替え前に実施し、初期化失敗時はそのままエラー終了）
    let mut sys = System::new_all()?;
    #[allow(unused_variables, unused_mut)]
    let mut disks = Disks::new_with_refreshed_list()?;
    #[allow(unused_variables, unused_mut)]
    let mut networks = Networks::new_with_refreshed_list()?;

    // ターミナルの初期設定（スコープ離脱時に TerminalGuard::drop が自動で復元）
    let mut stdout = io::stdout();
    let _terminal_guard = TerminalGuard::enter(&mut stdout)?;

    let mut state = MonitorState {
        #[cfg(feature = "cpu")]
        show_cpu_total: false,
        #[cfg(feature = "cpu")]
        show_cpu_cores: false,
        #[cfg(feature = "mem")]
        show_mem: false,
        #[cfg(feature = "disk")]
        show_disk: false,
        #[cfg(feature = "net")]
        show_net: false,
        #[cfg(feature = "proc")]
        show_proc: false,
        #[cfg(feature = "diff")]
        show_diff: false,
        #[cfg(any(feature = "proc", feature = "diff"))]
        filter_query: String::new(),
        #[cfg(any(feature = "proc", feature = "diff"))]
        is_filtering: false,
        #[cfg(feature = "diff")]
        last_process_list: String::new(),
        #[cfg(feature = "diff")]
        spawn_exit_log: Vec::new(),
        tick_rate: Duration::from_millis(1000),
        #[cfg(feature = "net")]
        last_refresh_secs: 1.0,
        is_setting_interval: false,
        interval_input: String::new(),
    };

    // システム情報の初回更新（CPU使用率の初回差分取得のため短い待機を挟む）
    sys.refresh_all();
    thread::sleep(Duration::from_millis(100));

    let mut last_tick = Instant::now();
    let mut need_refresh = true;

    loop {
        // 初回または更新間隔経過時のみ各種メトリクスを更新（キー入力時の不要な負荷・速度計算誤差を防止）
        if need_refresh || last_tick.elapsed() >= state.tick_rate {
            #[cfg(feature = "net")]
            {
                let elapsed = last_tick.elapsed().as_secs_f64();
                state.last_refresh_secs = if need_refresh {
                    1.0
                } else {
                    elapsed.max(0.001)
                };
            }

            sys.refresh_all();
            #[cfg(feature = "disk")]
            disks.refresh(true);
            #[cfg(feature = "net")]
            networks.refresh(true);

            // common_lib::compute_diff を使用したプロセスの変更検知
            #[cfg(feature = "diff")]
            {
                let mut current_processes: Vec<_> = sys.processes().values().collect();
                current_processes.sort_by_key(|p| p.pid().as_u32());
                let current_proc_str = current_processes
                    .iter()
                    .map(|p| format!("{}:{}", p.pid(), p.name().to_string_lossy()))
                    .collect::<Vec<_>>()
                    .join("\n");

                if state.last_process_list.is_empty() {
                    state.last_process_list = current_proc_str;
                } else {
                    let diffs =
                        common_lib::compute_diff(&state.last_process_list, &current_proc_str);
                    for diff in diffs {
                        match diff.diff_type {
                            common_lib::DiffType::Added => {
                                if let Some((pid, name)) = utils::parse_proc_line(&diff.value) {
                                    state
                                        .spawn_exit_log
                                        .push(format!("+ {} (PID: {})", name, pid));
                                }
                            }
                            common_lib::DiffType::Removed => {
                                if let Some((pid, name)) = utils::parse_proc_line(&diff.value) {
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

            last_tick = Instant::now();
            need_refresh = false;
        }

        // ターミナルの描画実行
        ui::draw_ui(&mut stdout, &sys, &disks, &networks, &state)?;

        // キーイベントのポーリング
        let timeout = state
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    #[allow(unused_mut)]
                    let mut handled_filter = false;
                    #[cfg(any(feature = "proc", feature = "diff"))]
                    if state.is_filtering {
                        handled_filter = true;
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
                    }

                    if !handled_filter {
                        if state.is_setting_interval {
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
                                #[cfg(feature = "cpu")]
                                KeyCode::Char('c') => state.show_cpu_cores = !state.show_cpu_cores,
                                #[cfg(feature = "cpu")]
                                KeyCode::Char('C') => state.show_cpu_total = !state.show_cpu_total,
                                #[cfg(feature = "mem")]
                                KeyCode::Char('m') => state.show_mem = !state.show_mem,
                                #[cfg(feature = "disk")]
                                KeyCode::Char('d') => state.show_disk = !state.show_disk,
                                #[cfg(feature = "net")]
                                KeyCode::Char('n') => state.show_net = !state.show_net,
                                #[cfg(feature = "proc")]
                                KeyCode::Char('p') | KeyCode::Char('t') => {
                                    state.show_proc = !state.show_proc
                                }
                                #[cfg(feature = "diff")]
                                KeyCode::Char('g') | KeyCode::Char('l') => {
                                    state.show_diff = !state.show_diff
                                }
                                #[cfg(any(feature = "proc", feature = "diff"))]
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
        }
    }

    // TerminalGuard がスコープを抜ける際にターミナル状態を自動復元する
    Ok(())
}
