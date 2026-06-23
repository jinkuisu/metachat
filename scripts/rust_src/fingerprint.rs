// ═══════════════════════════════════════════════════════════════
// fingerprint.rs — CloakBrowser 指纹配置模板系统
// 每个账号创建时分配一个 seed，基于 seed 生成唯一指纹
// 通过 CDP 在浏览器启动时应用
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use super::config::now_ms;

/// 指纹包含的所有维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    pub id: String,
    pub name: String,
    pub seed: String,

    // ── 浏览器身份 ──
    pub user_agent: String,
    pub platform: String,         // "Win32" | "MacIntel" | "Linux x86_64"
    pub browser_version: String,  // eg "120.0.0.0"

    // ── 时间 & 位置 ──
    pub timezone: String,         // "Asia/Shanghai"
    pub locale: String,           // "zh-CN", "en-US"
    pub languages: Vec<String>,   // ["zh-CN", "zh", "en"]

    // ── 屏幕 ──
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u32,         // 24 | 30 | 48
    pub pixel_ratio: f64,         // 1 | 1.25 | 1.5 | 2 | 2.5 | 3

    // ── 硬件 ──
    pub cpu_cores: u32,
    pub device_memory_gb: u32,    // navigator.deviceMemory
    pub gpu_vendor: String,
    pub gpu_renderer: String,

    // ── 指纹噪声 (0.0 = 关, 1.0 = 最大) ──
    pub canvas_noise_level: f64,
    pub webgl_noise_level: f64,
    pub audio_noise_level: f64,

    // ── 字体 ──
    pub fonts: Vec<String>,

    // ── 代理 ──
    pub proxy: Option<ProxyInfo>,

    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub proxy_type: String, // "http" | "https" | "socks5"
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

// ── 模板预设 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintTemplate {
    pub id: String,
    pub name: String,
    pub platform: String,  // "windows" | "mac" | "linux"
    pub browser: String,   // "Chrome 120", "Chrome 122"
    pub is_system: bool,   // 内置模板 vs 用户自定义
    pub config: FingerprintConfig,
}

impl FingerprintTemplate {
    /// 内置 "Windows Chrome 120" 模板
    pub fn windows_chrome_120(seed: &str) -> Self {
        Self {
            id: uuid_v4(),
            name: "Windows Chrome 120".into(),
            platform: "windows".into(),
            browser: "Chrome 120".into(),
            is_system: true,
            config: FingerprintConfig::generate(seed, "windows"),
        }
    }

    /// 内置 "macOS Chrome 120" 模板
    pub fn mac_chrome_120(seed: &str) -> Self {
        Self {
            id: uuid_v4(),
            name: "macOS Chrome 120".into(),
            platform: "mac".into(),
            browser: "Chrome 120".into(),
            is_system: true,
            config: FingerprintConfig::generate(seed, "mac"),
        }
    }
}

// ── 指纹生成 ──

impl FingerprintConfig {
    /// 从 seed + 平台生成一套完整指纹
    /// seed = account_id + salt，保证同一账号每次启动指纹不变
    pub fn generate(seed: &str, platform: &str) -> Self {
        use sha2::{Sha256, Digest};
        let hash = {
            let mut h = Sha256::new();
            h.update(seed.as_bytes());
            let r = h.finalize();
            r[..4].to_vec()
        };
        let seed_i32 = i32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let rng = |min: f64, max: f64| -> f64 {
            let n = (seed_i32 as f64 * 0.0001).sin().abs();
            min + (max - min) * (n * 100.0).fract()
        };

        let screen_w = match platform {
            "mac" => [1440, 1680, 1728, 1920][seed_i32.abs() as usize % 4],
            _ => [1366, 1440, 1536, 1600, 1920][seed_i32.abs() as usize % 5],
        };
        let screen_h = match screen_w {
            1920 | 1600 | 1440 => 900,
            1728 => 1117,
            1680 => 1050,
            1536 => 864,
            _ => 768,
        };

        Self {
            id: String::new(),
            name: format!("指纹 #{}", &seed[..8.min(seed.len())]),
            seed: seed.to_string(),

            user_agent: build_ua(platform),
            platform: match platform {
                "mac" => "MacIntel".into(),
                "linux" => "Linux x86_64".into(),
                _ => "Win32".into(),
            },
            browser_version: "120.0.0.0".into(),

            timezone: match platform {
                "mac" => "America/New_York".into(),
                _ => "Asia/Shanghai".into(),
            },
            locale: "zh-CN".into(),
            languages: vec!["zh-CN".into(), "zh".into(), "en".into()],

            screen_width: screen_w,
            screen_height: screen_h,
            color_depth: 24,
            pixel_ratio: if screen_w >= 1920 { 1.25 } else { 1.0 },

            cpu_cores: match platform {
                "mac" => [8, 10, 12][seed_i32.abs() as usize % 3],
                _ => [4, 8, 16][seed_i32.abs() as usize % 3],
            },
            device_memory_gb: match seed_i32.abs() % 3 { 0 => 8, 1 => 16, _ => 32 },
            gpu_vendor: "Google Inc. (Intel)".into(),
            gpu_renderer: "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0)".into(),

            canvas_noise_level: rng(0.01, 0.08),
            webgl_noise_level: rng(0.005, 0.03),
            audio_noise_level: rng(0.001, 0.01),

            fonts: vec![
                "Arial".into(), "Calibri".into(), "Cambria".into(),
                "Consolas".into(), "Georgia".into(), "Microsoft YaHei".into(),
                "Segoe UI".into(), "Tahoma".into(), "Times New Roman".into(),
                "Verdana".into(),
            ],
            proxy: None,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }
}

/// 构建 Chrome-aligned User-Agent
fn build_ua(platform: &str) -> String {
    let ver = "120.0.0.0";
    match platform {
        "mac" => format!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ver} Safari/537.36"),
        "linux" => format!("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ver} Safari/537.36"),
        _ => format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{ver} Safari/537.36"),
    }
}

/// 通过 CDP 应用指纹到 CloakBrowser 实例
impl FingerprintConfig {
    pub async fn apply_via_cdp(&self, cdp: &crate::cloak::cdp_client::CdpClient) -> Result<(), String> {
        // 1. UA + 平台
        cdp.call_method("Network.setUserAgentOverride", serde_json::json!({
            "userAgent": self.user_agent,
            "acceptLanguage": self.locale,
            "platform": self.platform,
        })).await.map_err(|e| format!("setUserAgent: {}", e))?;

        // 2. 时区
        cdp.call_method("Emulation.setTimezoneOverride", serde_json::json!({
            "timezoneId": self.timezone,
        })).await.map_err(|e| format!("setTimezone: {}", e))?;

        // 3. 语言
        cdp.call_method("Emulation.setLocaleOverride", serde_json::json!({
            "locale": self.locale,
        })).await.map_err(|e| format!("setLocale: {}", e))?;

        // 4. 屏幕 + 缩放
        cdp.call_method("Emulation.setDeviceMetricsOverride", serde_json::json!({
            "width": self.screen_width,
            "height": self.screen_height,
            "deviceScaleFactor": self.pixel_ratio,
            "mobile": false,
        })).await.map_err(|e| format!("setDeviceMetrics: {}", e))?;

        // 5. 注入 Canvas/WebGL/Audio 伪装脚本
        let injection_js = self.build_injection_script();
        cdp.call_method("Page.addScriptToEvaluateOnNewDocument", serde_json::json!({
            "source": injection_js,
        })).await.map_err(|e| format!("injection: {}", e))?;

        log::info!("Fingerprint applied: {} (seed={})", self.name, &self.seed[..8.min(self.seed.len())]);
        Ok(())
    }

    /// 构建注入到页面的 JS 伪装代码
    fn build_injection_script(&self) -> String {
        format!(r#"
(function() {{
    // 删除自动化痕迹
    delete window.__webdriver__;
    delete window.chrome?.loadTimes;
    delete window.chrome?.csi;

    // 伪造 navigator 属性
    Object.defineProperty(navigator, 'webdriver', {{ get: () => undefined }});
    Object.defineProperty(navigator, 'deviceMemory', {{ get: () => {} }});
    Object.defineProperty(navigator, 'hardwareConcurrency', {{ get: () => {} }});
    Object.defineProperty(navigator, 'platform', {{ get: () => '{}' }});
    Object.defineProperty(navigator, 'languages', {{ get: () => {} }});

    // Canvas 指纹噪声
    const origGetImageData = CanvasRenderingContext2D.prototype.getImageData;
    CanvasRenderingContext2D.prototype.getImageData = function(...args) {{
        const imageData = origGetImageData.apply(this, args);
        const noise = {};
        for (let i = 0; i < imageData.data.length; i += 4) {{
            imageData.data[i] = Math.min(255, imageData.data[i] + (Math.random() > 0.5 ? 1 : -1));
        }}
        return imageData;
    }};

    // WebGL 指纹噪声
    const origGetParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(param) {{
        const value = origGetParameter.call(this, param);
        if (param === 37445) return '{}';
        if (param === 37446) return '{}';
        return value;
    }};
}})();
"#,
            self.device_memory_gb,
            self.cpu_cores,
            self.platform,
            serde_json::to_string(&self.languages).unwrap_or_default(),
            self.canvas_noise_level * 100.0,
            self.gpu_vendor,
            self.gpu_renderer,
        )
    }
}

fn uuid_v4() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}
