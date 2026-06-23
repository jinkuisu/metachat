/// CloakBrowser 进程管理: 启动/停止/CDP 端口发现
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct CloakProcess {
    child: Option<tokio::process::Child>,
    pub port: u16,
    pub data_dir: PathBuf,
}

impl CloakProcess {
    pub async fn start(cloak_path: &Path, data_dir: &Path, port: u16) -> Result<Self, String> {
        let data_dir = data_dir.to_path_buf();
        let child = tokio::process::Command::new(cloak_path)
            .arg(format!("--remote-debugging-port={}", port))
            .arg(format!("--user-data-dir={}", data_dir.display()))
            .arg("--no-first-run").arg("--no-default-browser-check")
            .arg("--disable-extensions").arg("--about:blank")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start CloakBrowser: {}", e))?;
        let pid = child.id().unwrap_or(0);
        log::info!("CloakBrowser started on port {} (PID: {})", port, pid);
        let _ = crate::cloak::job::assign(pid);
        Ok(Self { child: Some(child), port, data_dir })
    }

    pub async fn start_headless(cloak_path: &Path, data_dir: &Path, port: u16) -> Result<Self, String> {
        let data_dir = data_dir.to_path_buf();
        let child = tokio::process::Command::new(cloak_path)
            .arg(format!("--remote-debugging-port={}", port))
            .arg(format!("--user-data-dir={}", data_dir.display()))
            .arg("--headless=new").arg("--disable-gpu")
            .arg("--no-first-run").arg("--about:blank")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start headless CloakBrowser: {}", e))?;
        let pid = child.id().unwrap_or(0);
        log::info!("Headless CloakBrowser started on port {} (PID: {})", port, pid);
        let _ = crate::cloak::job::assign(pid);
        Ok(Self { child: Some(child), port, data_dir })
    }

    pub async fn wait_for_cdp(&self, timeout_secs: u64) -> Result<String, String> {
        let url = format!("http://127.0.0.1:{}/json/version", self.port);
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        loop {
            if start.elapsed() > timeout {
                return Err(format!("CDP port {} not ready", self.port));
            }
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if let Ok(body) = resp.text().await {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(ws_url) = json["webSocketDebuggerUrl"].as_str() {
                                return Ok(ws_url.to_string());
                            }
                        }
                    }
                }
                Err(_) => {}
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    pub async fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let pid = child.id().unwrap_or(0);
            match child.kill().await {
                Ok(_) => { let _ = child.wait().await; log::info!("Stopped PID {}", pid); }
                Err(e) => log::warn!("Failed to kill PID {}: {}", pid, e),
            }
        }
    }

    pub fn is_running(&mut self) -> bool {
        self.child.as_mut().map_or(false, |c| c.try_wait().map_or(true, |s| s.is_none()))
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(|c| c.id())
    }
}

impl Drop for CloakProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() { let _ = child.start_kill(); }
    }
}
/// 查找已安装的 CloakBrowser 二进制路径
pub fn find_cloak_browser() -> Option<PathBuf> {
    // 1. 生产路径: {app_data}\metachat\cloak\chrome.exe
    if let Some(data) = dirs::data_dir() {
        let cloak_dir = data.join("metachat").join("cloak");
        let exe = cloak_dir.join("chrome.exe");
        if exe.exists() { return Some(exe); }
        // also search chrome-win64/ (GitHub Release zip structure)
        if let Some(found) = find_chrome_in_dir(&cloak_dir) {
            return Some(found);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let d = home.join(".cloakbrowser");
        if d.join("chrome.exe").exists() { return Some(d.join("chrome.exe")); }
        // 版本目录: chromium-{version}/chrome-win/chrome.exe
        if let Ok(entries) = std::fs::read_dir(&d) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && p.file_name().unwrap_or_default().to_str().unwrap_or("").starts_with("chromium-") {
                    for subdir in &["", "chrome-win", "chrome-linux", "chrome-mac"] {
                        let exe = p.join(subdir).join("chrome.exe");
                        if exe.exists() { return Some(exe); }
                        let exe2 = p.join(subdir).join("chrome");
                        if exe2.exists() { return Some(exe2); }
                    }
                }
            }
        }
    }
    for p in &[PathBuf::from("../node_modules/cloakbrowser"), PathBuf::from("node_modules/cloakbrowser")] {
        if let Some(f) = find_chrome_in_dir(p) { return Some(f); }
    }
    if let Ok(path) = std::env::var("CLOAK_BROWSER_PATH") {
        let p = PathBuf::from(path);
        if p.exists() { return Some(p); }
    }
    None
}

fn find_chrome_in_dir(dir: &Path) -> Option<PathBuf> {
    for n in &["chrome.exe","chromium.exe","chrome","chromium","google-chrome","Chromium"] {
        let p = dir.join(n);
        if p.exists() { return Some(p); }
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let sub = entry.path();
            if sub.is_dir() {
                if let Some(f) = find_chrome_in_dir(&sub) { return Some(f); }
            }
        }
    }
    None
}

pub fn has_command(cmd: &str) -> bool {
    let check = if cfg!(windows) { "where" } else { "which" };
    std::process::Command::new(check).arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}