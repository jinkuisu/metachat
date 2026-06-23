// ═══════════════════════════════════════════════════════════════
// fingerprint.rs — CloakBrowser 指纹配置模板系统
// 每个账号创建时分配一个 seed，基于 seed 生成唯一指纹
// 通过 CDP 在浏览器启动时应用
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use super::config::now_ms;

// ── 代理配置 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: String,  // "http" | "https" | "socks5"
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ProxyConfig {
    pub fn to_cli_arg(&self) -> String {
        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            format!("{}://{}:{}@{}:{}", self.proxy_type, u, p, self.host, self.port)
        } else {
            format!("{}://{}:{}", self.proxy_type, self.host, self.port)
        }
    }
}

// ── 完整指纹配置 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub seed: String,              // 账号创建时分配，保证确定性
    pub template_id: String,      // 关联的模板 ID
    pub is_system: bool,          // 内置模板不可修改

    // Web 客户端身份 (Layer 1: 启动参数)
    pub user_agent: String,
    pub platform: String,          // "Win32" | "MacIntel" | "Linux x86_64"
    pub browser_version: String,   // "120.0.0.0"
    pub hardware_concurrency: u32,
    pub device_memory_gb: u32,

    // 时间 & 区域 (Layer 2: CDP Emulation)
    pub timezone: String,          // "Asia/Shanghai"
    pub locale: String,            // "zh-CN"
    pub languages: Vec<String>,    // ["zh-CN", "zh", "en"]
    pub accept_language: String,   // "zh-CN,zh;q=0.9,en;q=0.8"

    // 屏幕 (Layer 2: CDP Emulation)
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u32,          // 24 | 30 | 48
    pub pixel_ratio: f64,          // 1 | 1.25 | 1.5 | 2 | 2.5 | 3

    // GPU (Layer 2: CDP Emulation)
    pub gpu_vendor: String,
    pub gpu_renderer: String,

    // 指纹噪声 (Layer 3: JS Injection)
    pub canvas_noise: f64,         // 0.001 ~ 0.1
    pub webgl_noise: f64,          // 0.001 ~ 0.05
    pub audio_noise: f64,          // 0.0001 ~ 0.01

    // 字体
    pub fonts: Vec<String>,

    // 代理
    pub proxy: Option<ProxyConfig>,

    pub created_at: i64,
    pub updated_at: i64,
}

// ── 模板预设 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub platform: String,      // "windows" | "mac" | "linux"
    pub browser: String,       // "Chrome 120"
    pub is_system: bool,       // 内置 vs 用户自定义
    pub config: FingerprintConfig,
}

impl FingerprintTemplate {
    pub fn windows_chrome_120(seed: &str) -> Self {
        Self {
            id: uuid_v4(),
            name: "Windows Chrome 120".into(),
            description: "Windows 10 + Chrome 120 标准指纹".into(),
            platform: "windows".into(),
            browser: "Chrome 120".into(),
            is_system: true,
            config: FingerprintConfig::generate(seed, "windows", "Chrome 120"),
        }
    }

    pub fn mac_chrome_120(seed: &str) -> Self {
        Self {
            id: uuid_v4(),
            name: "macOS Chrome 120".into(),
            description: "macOS 14 Sonoma + Chrome 120 标准指纹".into(),
            platform: "mac".into(),
            browser: "Chrome 120".into(),
            is_system: true,
            config: FingerprintConfig::generate(seed, "mac", "Chrome 120"),
        }
    }

    pub fn windows_chrome_124(seed: &str) -> Self {
        Self {
            id: uuid_v4(),
            name: "Windows Chrome 124".into(),
            description: "Windows 11 + Chrome 124 新版本指纹".into(),
            platform: "windows".into(),
            browser: "Chrome 124".into(),
            is_system: true,
            config: FingerprintConfig::generate(seed, "windows", "Chrome 124"),
        }
    }

    /// 所有内置模板
    pub fn system_templates() -> Vec<FingerprintTemplate> {
        vec![
            Self::windows_chrome_120("system_default"),
            Self::windows_chrome_124("system_default"),
            Self::mac_chrome_120("system_default"),
        ]
    }
}

// ── 指纹生成引擎 ──

impl FingerprintConfig {
    /// 从 seed + 平台 + 浏览器版本生成完整指纹
    pub fn generate(seed: &str, platform: &str, browser: &str) -> Self {
        // SHA256(seed) → 确定性种子
        let hash = {
            let mut h = Sha256::new();
            h.update(seed.as_bytes());
            h.finalize()
        };
        let seed_val = i32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let rng = |min: f64, max: f64| -> f64 {
            let n = (seed_val as f64).sin().abs();
            min + (max - min) * (n * 100.0).fract()
        };

        // 屏幕分辨率 (从候选池确定选择)
        let (screen_w, screen_h) = match platform {
            "mac" => {
                let pool = [(1440, 900), (1680, 1050), (1728, 1117), (2560, 1600)];
                pool[seed_val.abs() as usize % pool.len()]
            }
            _ => {
                let pool = [(1366, 768), (1440, 900), (1536, 864), (1600, 900), (1920, 1080)];
                pool[seed_val.abs() as usize % pool.len()]
            }
        };

        // 浏览器版本
        let ver = match browser {
            "Chrome 124" => "124.0.6367.0",
            "Chrome 122" => "122.0.6261.0",
            _ => "120.0.6099.0",
        };

        Self {
            id: String::new(),
            name: format!("指纹 #{}", &seed[..seed.len().min(8)]),
            description: String::new(),
            seed: seed.to_string(),
            template_id: String::new(),
            is_system: false,

            user_agent: build_ua(platform, ver),
            platform: match platform {
                "mac" => "MacIntel".into(),
                "linux" => "Linux x86_64".into(),
                _ => "Win32".into(),
            },
            browser_version: ver.to_string(),
            hardware_concurrency: match platform {
                "mac" => [8, 10, 12][seed_val.abs() as usize % 3],
                _ => [4, 8, 16][seed_val.abs() as usize % 3],
            },
            device_memory_gb: match seed_val.abs() % 3 { 0 => 8, 1 => 16, _ => 32 },

            timezone: pick_timezone(seed_val),
            locale: "zh-CN".into(),
            languages: vec!["zh-CN".into(), "zh".into(), "en".into()],
            accept_language: "zh-CN,zh;q=0.9,en;q=0.8".into(),

            screen_width: screen_w,
            screen_height: screen_h,
            color_depth: 24,
            pixel_ratio: if screen_w >= 1920 { 1.25 } else { 1.0 },

            gpu_vendor: "Google Inc. (Intel)".into(),
            gpu_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0)".into(),

            canvas_noise: rng(0.003, 0.08),
            webgl_noise: rng(0.002, 0.04),
            audio_noise: rng(0.0005, 0.008),

            fonts: vec![
                "Arial".into(), "Calibri".into(), "Consolas".into(),
                "Georgia".into(), "Microsoft YaHei".into(), "Segoe UI".into(),
                "Tahoma".into(), "Times New Roman".into(), "Verdana".into(),
            ],
            proxy: None,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }

    /// 通过 CDP 应用指纹到 CloakBrowser 实例
    pub async fn apply_via_cdp(&self, cdp: &crate::cloak::cdp_client::CdpClient) -> Result<(), String> {
        // Layer 2: UA + 网络层
        cdp.call_method("Network.setUserAgentOverride", serde_json::json!({
            "userAgent": self.user_agent,
            "acceptLanguage": self.accept_language,
            "platform": self.platform,
        })).await.map_err(|e| format!("setUserAgent: {}", e))?;

        // Layer 2: 时区
        cdp.call_method("Emulation.setTimezoneOverride", serde_json::json!({
            "timezoneId": self.timezone,
        })).await.map_err(|e| format!("setTimezone: {}", e))?;

        // Layer 2: 语言
        cdp.call_method("Emulation.setLocaleOverride", serde_json::json!({
            "locale": self.locale,
        })).await.map_err(|e| format!("setLocale: {}", e))?;

        // Layer 2: 屏幕
        cdp.call_method("Emulation.setDeviceMetricsOverride", serde_json::json!({
            "width": self.screen_width,
            "height": self.screen_height,
            "deviceScaleFactor": self.pixel_ratio,
            "mobile": false,
        })).await.map_err(|e| format!("setDeviceMetrics: {}", e))?;

        // Layer 3: JS 注入 (Canvas/WebGL/Audio 噪声 + navigator 覆盖)
        let js = self.build_injection_script();
        cdp.call_method("Page.addScriptToEvaluateOnNewDocument", serde_json::json!({
            "source": js,
        })).await.map_err(|e| format!("injection: {}", e))?;

        log::info!("Fingerprint applied: {} (seed={})", self.name, &self.seed[..self.seed.len().min(8)]);
        Ok(())
    }

    /// 构建注入到页面的 JS 伪装代码
    fn build_injection_script(&self) -> String {
        format!(r#"
(function() {{
    'use strict';
    const C = {{}};

    // 1. 删除自动化痕迹
    delete window.__webdriver__;
    delete window.cdc_adoQpoasnfa76pfcZLmcfl_;
    if (window.chrome) {{
        delete window.chrome.loadTimes;
        delete window.chrome.csi;
    }}

    // 2. navigator 属性覆盖
    Object.defineProperty(navigator, 'webdriver',       {{ get: () => undefined, configurable: true }});
    Object.defineProperty(navigator, 'deviceMemory',    {{ get: () => {}, configurable: true }});
    Object.defineProperty(navigator, 'hardwareConcurrency', {{ get: () => {}, configurable: true }});
    Object.defineProperty(navigator, 'platform',        {{ get: () => '{}', configurable: true }});
    Object.defineProperty(navigator, 'languages',       {{ get: () => {}, configurable: true }});

    // 3. Canvas 指纹噪声
    const origCID = CanvasRenderingContext2D.prototype.getImageData;
    CanvasRenderingContext2D.prototype.getImageData = function() {{
        const img = origCID.apply(this, arguments);
        for (let i = 0; i < img.data.length; i += 4) {{
            img.data[i]     = Math.min(255, img.data[i]   + (Math.random() > 0.5 ? 1 : -1));
            img.data[i + 1] = Math.min(255, img.data[i+1] + (Math.random() > 0.5 ? 1 : -1));
            img.data[i + 2] = Math.min(255, img.data[i+2] + (Math.random() > 0.5 ? 1 : -1));
        }}
        return img;
    }};

    // 4. WebGL 指纹噪声
    const origGP = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(p) {{
        const v = origGP.call(this, p);
        if (p === 37445) return '{}';
        if (p === 37446) return '{}';
        return v;
    }};

    // 5. Audio 指纹噪声
    const origADC = AudioContext.prototype.createOscillator;
    AudioContext.prototype.createOscillator = function() {{
        const osc = origADC.call(this);
        const origGetFreq = osc.frequency.linearRampToValueAtTime;
        osc.frequency.linearRampToValueAtTime = function(v, t) {{
            return origGetFreq.call(this, v * (1 + (Math.random() - 0.5) * 0.001), t);
        }};
        return osc;
    }};
}})();
"#,
            self.device_memory_gb,
            self.hardware_concurrency,
            self.platform,
            serde_json::to_string(&self.languages).unwrap_or_default(),
            self.gpu_vendor,
            self.gpu_renderer,
        )
    }
}

// ── 辅助函数 ──

fn build_ua(platform: &str, version: &str) -> String {
    match platform {
        "mac" => format!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{version} Safari/537.36"),
        "linux" => format!("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{version} Safari/537.36"),
        _ => format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{version} Safari/537.36"),
    }
}

fn pick_timezone(seed: i32) -> String {
    const ZONES: &[&str] = &[
        "Asia/Shanghai", "Asia/Tokyo", "Asia/Seoul",
        "America/New_York", "America/Chicago", "America/Los_Angeles",
        "Europe/London", "Europe/Paris", "Europe/Berlin",
        "Australia/Sydney", "Asia/Singapore", "Asia/Hong_Kong",
    ];
    ZONES[seed.abs() as usize % ZONES.len()].to_string()
}

fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}
