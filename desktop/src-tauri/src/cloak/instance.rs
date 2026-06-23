use serde::{Deserialize, Serialize};

/// CloakBrowser 实例状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceState {
    /// 未启动 (仅数据库元数据)
    Offline,
    /// 正在连接 (CDP 握手)
    Connecting,
    /// 监听中 (headless, CDP-Only, ~30MB)
    Monitoring,
    /// 预渲染完成 (全渲染但隐藏, 瞬间切换)
    Warming,
    /// 活跃 (全渲染, 用户可见)
    Active,
    /// 已冻结 (已杀进程, Cookie 存 SQLite)
    Frozen,
}

impl InstanceState {
    pub fn numeric(&self) -> u8 {
        match self {
            Self::Offline => 0,
            Self::Connecting => 1,
            Self::Monitoring => 2,
            Self::Warming => 3,
            Self::Active => 4,
            Self::Frozen => 5,
        }
    }

    /// 是否正在消耗大量内存
    pub fn is_heavy(&self) -> bool {
        matches!(self, Self::Active | Self::Warming)
    }

    /// 是否在消耗任何资源
    pub fn is_alive(&self) -> bool {
        !matches!(self, Self::Offline | Self::Frozen)
    }
}

/// 单个 CloakBrowser 实例
#[derive(Debug, Clone)]
pub struct CloakInstance {
    pub id: String,
    pub account_id: String,
    pub platform: String,
    pub state: InstanceState,
    pub process_id: Option<u32>,
    pub cdp_port: Option<u16>,
    pub memory_bytes: u64,
    pub unread_count: u32,
    pub last_activity_at: i64,   // 用户最后交互时间
    pub last_message_at: i64,    // 最后收到消息时间
    pub priority_score: f64,
    pub is_pinned: bool,
    pub user_data_dir: String,
    pub cookies: Option<Vec<u8>>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl CloakInstance {
    pub fn new(id: String, account_id: String, platform: String, user_data_dir: String) -> Self {
        let now = crate::cloak::config::now_ms();
        Self {
            id,
            account_id,
            platform,
            state: InstanceState::Offline,
            process_id: None,
            cdp_port: None,
            memory_bytes: 0,
            unread_count: 0,
            last_activity_at: now,
            last_message_at: now,
            priority_score: 0.0,
            is_pinned: false,
            user_data_dir,
            cookies: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 距离最后活动的时间 (秒)
    pub fn idle_secs(&self) -> f64 {
        let now = crate::cloak::config::now_ms();
        (now - self.last_activity_at) as f64 / 1000.0
    }

    /// 距离最后消息的时间 (分钟)
    pub fn minutes_since_message(&self) -> f64 {
        let now = crate::cloak::config::now_ms();
        (now - self.last_message_at) as f64 / 60000.0
    }

    /// 规一化未读数 (0.0-1.0)，用于跨账号比较
    pub fn normalized_unread(&self, max_unread: u32) -> f64 {
        if max_unread == 0 { return 0.0; }
        self.unread_count as f64 / max_unread as f64
    }

    /// 规一化近因分数 (0.0-1.0)，数值越高表示越近
    pub fn normalized_recency(&self, window_minutes: f64) -> f64 {
        let since = self.minutes_since_message();
        let score = 1.0 - (since / window_minutes);
        score.clamp(0.0, 1.0)
    }

    /// 自适应优先级分数 (由 pool 的算法设置 weights)
    pub fn calc_priority(
        &self,
        alpha: f64,   // unread 权重
        beta: f64,    // recency 权重
        gamma: f64,   // pin 权重
        max_unread: u32,
        recency_window: f64,
    ) -> f64 {
        let unread_score = self.normalized_unread(max_unread);
        let recency_score = self.normalized_recency(recency_window);
        let pin_score = if self.is_pinned { 1.0 } else { 0.0 };
        alpha * unread_score + beta * recency_score + gamma * pin_score
    }
}
