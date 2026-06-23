use std::sync::atomic::AtomicU64;
use std::sync::Arc;

/// CloakBrowser 实例池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_active: u32,
    pub max_warm: u32,
    pub max_monitor: u32,
    pub memory_budget_pct: f64,
    pub reserve_gb: f64,
    pub per_active_gb: f64,
    pub per_monitor_gb: f64,
    pub demote_idle_secs: u64,
    pub warm_to_monitor_secs: u64,
    pub freeze_idle_secs: u64,
    pub poll_interval_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_active: 5,
            max_warm: 2,
            max_monitor: 20,
            memory_budget_pct: 0.75,
            reserve_gb: 1.5,
            per_active_gb: 0.4,
            per_monitor_gb: 0.035,
            demote_idle_secs: 300,
            warm_to_monitor_secs: 600,
            freeze_idle_secs: 1800,
            poll_interval_ms: 10000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub pool_memory_bytes: Arc<AtomicU64>,
}

impl SystemInfo {
    pub fn new() -> Self {
        let total = 8.0;
        Self {
            total_ram_gb: total,
            available_ram_gb: total * 0.7,
            pool_memory_bytes: Arc::new(AtomicU64::new(0)),
        }
    }
}

pub fn compute_max_active(system_ram_gb: f64, config: &PoolConfig) -> u32 {
    let budget_gb = system_ram_gb * config.memory_budget_pct;
    let available_gb = budget_gb - config.reserve_gb;
    if available_gb <= 0.0 { return 1; }
    let max_by_ram = (available_gb / config.per_active_gb).floor() as u32;
    max_by_ram.clamp(1, 8)
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
