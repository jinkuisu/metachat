use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

use super::config::{compute_max_active, now_ms, PoolConfig};
use super::instance::{CloakInstance, InstanceState};

pub struct InstancePool {
    pub instances: HashMap<String, CloakInstance>,
    pub config: PoolConfig,
    lru_active: Vec<String>,
    lru_warm: Vec<String>,
    lru_monitor: Vec<String>,
    pub total_ram_gb: f64,
    pub pool_memory_mb: u64,
}

impl InstancePool {
    pub fn new(config: PoolConfig) -> Self {
        let total_ram = 8.0;
        let mut pool = Self {
            instances: HashMap::new(),
            lru_active: Vec::new(),
            lru_warm: Vec::new(),
            lru_monitor: Vec::new(),
            total_ram_gb: total_ram,
            pool_memory_mb: 0,
            config,
        };
        pool.config.max_active = compute_max_active(total_ram, &pool.config);
        pool
    }

    pub fn add_instance(&mut self, instance: CloakInstance) {
        let id = instance.id.clone();
        self.instances.insert(id.clone(), instance);
        self.lru_monitor.push(id);
    }

    pub fn remove_instance(&mut self, id: &str) -> bool {
        if let Some(inst) = self.instances.remove(id) {
            self.remove_from_lru(id, &inst.state);
            true
        } else {
            false
        }
    }

    pub fn get_instance(&self, id: &str) -> Option<&CloakInstance> {
        self.instances.get(id)
    }

    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut CloakInstance> {
        self.instances.get_mut(id)
    }

    // ── 状态转换 (先读状态再修改，避免 double borrow) ──

    pub fn transition_to(&mut self, id: &str, new_state: InstanceState) {
        let prev = self.instances.get(id).map(|i| i.state.clone());
        let Some(ref old) = prev else { return };
        if *old == new_state { return; }
        self.remove_from_lru(id, old);
        if let Some(inst) = self.instances.get_mut(id) {
            inst.state = new_state.clone();
            inst.updated_at = now_ms();
        }
        self.push_to_lru(id, &new_state);
    }

    fn remove_from_lru(&mut self, id: &str, state: &InstanceState) {
        match state {
            InstanceState::Active => self.lru_active.retain(|x| x != id),
            InstanceState::Warming => self.lru_warm.retain(|x| x != id),
            InstanceState::Monitoring => self.lru_monitor.retain(|x| x != id),
            _ => {}
        }
    }

    fn push_to_lru(&mut self, id: &str, state: &InstanceState) {
        match state {
            InstanceState::Active => self.lru_active.push(id.to_string()),
            InstanceState::Warming => self.lru_warm.push(id.to_string()),
            InstanceState::Monitoring => self.lru_monitor.push(id.to_string()),
            _ => {}
        }
    }

    pub fn touch(&mut self, id: &str) {
        let should_touch = self.instances.get(id).is_some();
        if should_touch {
            if let Some(inst) = self.instances.get_mut(id) {
                inst.last_activity_at = now_ms();
                let state = inst.state.clone();
                self.remove_from_lru(id, &state);
                self.push_to_lru(id, &state);
            }
        }
    }

    pub fn notify_message(&mut self, id: &str, count: u32) {
        if let Some(inst) = self.instances.get_mut(id) {
            inst.unread_count += count;
            inst.last_message_at = now_ms();
        }
    }

    /// 检查 MONITOR 中未读达标的自动升级为 WARM
    pub fn evaluate_promote(&mut self) -> Vec<String> {
        let mut promoted = Vec::new();
        let ids: Vec<String> = self.instances
            .iter()
            .filter(|(_, inst)| inst.state == InstanceState::Monitoring && inst.unread_count >= 3)
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            self.transition_to(&id, InstanceState::Warming);
            promoted.push(id);
        }
        promoted
    }

    pub fn mark_read(&mut self, id: &str) {
        if let Some(inst) = self.instances.get_mut(id) {
            inst.unread_count = 0;
        }
    }

    pub fn set_pinned(&mut self, id: &str, pinned: bool) {
        if let Some(inst) = self.instances.get_mut(id) {
            inst.is_pinned = pinned;
        }
    }

    // ── 自适应调度 ──

    pub fn evaluate(&mut self) -> Vec<String> {
        self.update_memory_estimate();

        let active_count = self.count_state(&InstanceState::Active);
        let warm_count = self.count_state(&InstanceState::Warming);
        let total_heavy = active_count + warm_count;
        let capacity = self.config.max_active as usize;

        if total_heavy <= capacity {
            return Vec::new();
        }

        let total_unread: u32 = self.instances.values().map(|i| i.unread_count).sum();
        let message_density = total_unread as f64 / capacity.max(1) as f64;
        let pressure = total_heavy as f64 / capacity.max(1) as f64;

        let (_mode, alpha, beta, gamma) = self.determine_mode(pressure, message_density);
        let demote_count = total_heavy - capacity;
        self.demote_lowest(demote_count, alpha, beta, gamma)
    }

    fn determine_mode(&self, pressure: f64, message_density: f64) -> (u8, f64, f64, f64) {
        if pressure > 0.8 {
            (4, 1.0, 0.0, 0.1)
        } else if message_density < 2.0 {
            (1, 0.0, 1.0, 0.1)
        } else if message_density >= 5.0 {
            (3, 0.8, 0.2, 0.1)
        } else {
            (2, 0.6, 0.4, 0.1)
        }
    }

    fn demote_lowest(&mut self, count: usize, alpha: f64, beta: f64, gamma: f64) -> Vec<String> {
        let ids: Vec<(String, f64, bool)> = {
            let max_u = self.instances.values().map(|i| i.unread_count).max().unwrap_or(1).max(1);
            let mut scored = Vec::new();
            for (id, inst) in &self.instances {
                if matches!(inst.state, InstanceState::Active | InstanceState::Warming) {
                    let score = inst.calc_priority(alpha, beta, gamma, max_u, 30.0);
                    scored.push((id.clone(), score, inst.is_pinned));
                }
            }
            scored.sort_by(|a, b| {
                if a.2 != b.2 {
                    if a.2 { Ordering::Greater } else { Ordering::Less }
                } else {
                    a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal)
                }
            });
            scored
        };

        let mut demoted = Vec::new();
        for (id, _score, pinned) in ids.iter().take(count) {
            // 根据当前状态决定降级方案
            let current = self.instances.get(id).map(|i| i.state.clone()).unwrap_or(InstanceState::Offline);
            match current {
                InstanceState::Active => {
                    self.transition_to(id, InstanceState::Warming);
                }
                InstanceState::Warming => {
                    let target = if *pinned { InstanceState::Monitoring } else { InstanceState::Monitoring };
                    self.transition_to(id, target);
                }
                _ => {}
            }
            demoted.push(id.clone());
        }
        demoted
    }

    pub fn evaluate_freeze(&mut self) -> Vec<String> {
        let threshold = (self.config.freeze_idle_secs * 1000) as i64;
        let now = now_ms();

        let ids: Vec<String> = self.instances
            .iter()
            .filter(|(_, inst)| inst.state == InstanceState::Monitoring)
            .filter(|(_, inst)| !inst.is_pinned)
            .filter(|(_, inst)| inst.unread_count == 0)
            .filter(|(_, inst)| (now - inst.last_message_at) > threshold)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &ids {
            self.transition_to(id, InstanceState::Frozen);
        }
        ids
    }

    fn count_state(&self, state: &InstanceState) -> usize {
        self.instances.values().filter(|i| i.state == *state).count()
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active: self.count_state(&InstanceState::Active) as u32,
            warm: self.count_state(&InstanceState::Warming) as u32,
            monitor: self.count_state(&InstanceState::Monitoring) as u32,
            frozen: self.count_state(&InstanceState::Frozen) as u32,
            total: self.instances.len() as u32,
            total_memory_mb: self.pool_memory_mb,
            max_active: self.config.max_active,
            total_ram_gb: self.total_ram_gb,
        }
    }

    fn update_memory_estimate(&mut self) {
        self.pool_memory_mb = self.instances.values().map(|inst| match inst.state {
            InstanceState::Active | InstanceState::Warming => 400u64,
            InstanceState::Monitoring => 35,
            _ => 0,
        }).sum();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub active: u32,
    pub warm: u32,
    pub monitor: u32,
    pub frozen: u32,
    pub total: u32,
    pub total_memory_mb: u64,
    pub max_active: u32,
    pub total_ram_gb: f64,
}
