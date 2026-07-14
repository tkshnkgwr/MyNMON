use std::time::Duration;

/// アプリケーションの監視状態や設定を管理する構造体。
pub struct MonitorState {
    /// 全体CPU使用率を表示するかどうか
    pub show_cpu_total: bool,
    /// 個々のCPUコア使用率を表示するかどうか
    pub show_cpu_cores: bool,
    /// メモリ使用状況を表示するかどうか
    pub show_mem: bool,
    /// ディスク空き容量を表示するかどうか
    pub show_disk: bool,
    /// ネットワーク速度を表示するかどうか
    pub show_net: bool,
    /// プロセス一覧を表示するかどうか
    pub show_proc: bool,
    /// プロセス起動・終了ログを表示するかどうか
    pub show_diff: bool,
    /// プロセス名フィルターのクエリ文字列
    pub filter_query: String,
    /// 現在プロセスフィルター入力モードであるかどうか
    pub is_filtering: bool,
    /// 前回のプロセス一覧の文字列表現（差分検出用）
    pub last_process_list: String,
    /// プロセスの起動（+）および終了（-）の履歴ログ
    pub spawn_exit_log: Vec<String>,
    /// 画面の更新レート（ティックレート）
    pub tick_rate: Duration,
    /// 現在更新間隔設定入力モードであるかどうか
    pub is_setting_interval: bool,
    /// 更新間隔入力用の一時文字列
    pub interval_input: String,
}
