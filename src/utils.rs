/// システムの起動時間（秒）を、日・時間・分・秒の読みやすい形式の文字列にフォーマットする。
///
/// 1日以上の場合は `Xd HH:MM:SS`、1日未満の場合は `HH:MM:SS` となります。
pub fn format_uptime(uptime_secs: u64) -> String {
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
#[allow(dead_code)]
pub fn pad_or_truncate(s: &str, width: usize) -> String {
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

/// パーセンテージと幅に基づいて、ASCIIで表現されたプログレスバー（例: `[====>     ]`）を生成する。
///
/// # 引数
/// - `percent`: 割合（0.0 から 100.0 の範囲にクランプされます）
/// - `width`: バーの文字幅
#[allow(dead_code)]
pub fn get_ascii_bar(percent: f64, width: usize) -> String {
    if width == 0 {
        return "[]".to_string();
    }
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
#[allow(dead_code)]
pub fn parse_proc_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// アプリケーションのヘルプ情報（コマンドラインオプションや起動中の操作キー説明）を標準出力に表示する。
pub fn print_help() {
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
    #[cfg(feature = "cpu")]
    println!("  C  Toggle Total CPU utilization display");
    #[cfg(feature = "cpu")]
    println!("  c  Toggle Individual CPU Core utilization display");
    #[cfg(feature = "mem")]
    println!("  m  Toggle Memory allocation display");
    #[cfg(feature = "disk")]
    println!("  d  Toggle Disk mounts & space display");
    #[cfg(feature = "net")]
    println!("  n  Toggle Network interface speed display");
    #[cfg(feature = "proc")]
    println!("  p  Toggle Top processes display (also 't' key)");
    #[cfg(feature = "diff")]
    println!("  g  Toggle Process Spawn/Exit history log (also 'l' key)");
    #[cfg(any(feature = "proc", feature = "diff"))]
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
        // 範囲外の値や幅 0 のエッジケースでもパニックしないこと
        assert_eq!(get_ascii_bar(-10.0, 10), "[          ]");
        assert_eq!(get_ascii_bar(150.0, 10), "[=========>]");
        assert_eq!(get_ascii_bar(50.0, 0), "[]");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(0), "00:00:00");
        assert_eq!(format_uptime(3661), "01:01:01");
        assert_eq!(format_uptime(86400 + 3661), "1d 01:01:01");
    }

    #[test]
    fn test_pad_or_truncate() {
        assert_eq!(pad_or_truncate("eth0", 8), "eth0    ");
        assert_eq!(pad_or_truncate("abcdefghij", 5), "abcde");
        // 全角文字（幅2）を含む場合の幅計算と切り詰め
        assert_eq!(pad_or_truncate("あいう", 6), "あいう");
        assert_eq!(pad_or_truncate("あいう", 5), "あい ");
        assert_eq!(pad_or_truncate("aい", 5), "aい  ");
    }

    #[test]
    fn test_parse_proc_line() {
        assert_eq!(
            parse_proc_line("1234:chrome.exe"),
            Some(("1234".to_string(), "chrome.exe".to_string()))
        );
        assert_eq!(
            parse_proc_line("42:name:with:colon"),
            Some(("42".to_string(), "name:with:colon".to_string()))
        );
        assert_eq!(parse_proc_line("invalid_line_without_colon"), None);
    }
}
