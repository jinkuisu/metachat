// ═══════════════════════════════════════════════════
// installer.rs — CloakBrowser 安装器
// 合规方案：通过官方 CLI 安装，不自托管二进制
// ═══════════════════════════════════════════════════

use std::path::{Path, PathBuf};
use std::process::Command;

/// 查找已安装的 CloakBrowser
pub fn find_cloak_browser() -> Option<PathBuf> {
    // 1. 官方默认路径 ~/.cloakbrowser/
    if let Some(home) = dirs::home_dir() {
        let default_dir = home.join(".cloakbrowser");
        let exe = find_chrome_in_dir(&default_dir);
        if exe.is_some() { return exe; }
    }
    // 2. 开发环境 node_modules
    let dev_dir = PathBuf::from("node_modules/cloakbrowser");
    let exe = find_chrome_in_dir(&dev_dir);
    if exe.is_some() { return exe; }
    // 3. 环境变量
    if let Ok(path) = std::env::var("CLOAK_BROWSER_PATH") {
        let p = PathBuf::from(path);
        if p.exists() { return Some(p); }
    }
    None
}

fn find_chrome_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = if cfg!(target_os = "windows") {
        vec!["chrome.exe", "chromium.exe", "msedge.exe"]
    } else if cfg!(target_os = "macos") {
        vec!["Chromium", "chrome", "Google Chrome"]
    } else {
        vec!["chrome", "chromium", "google-chrome"]
    };
    for name in &candidates {
        let p = dir.join(name);
        if p.exists() { return Some(p); }
    }
    None
}

/// 安装结果
pub enum InstallResult {
    Found(PathBuf),
    Installing,
    RequiresNodeJs,
    RequiresPython,
    Error(String),
}

/// 检测系统可用的运行时，决定安装方式
pub fn check_install_readiness() -> InstallResult {
    // 已经安装了
    if let Some(path) = find_cloak_browser() {
        return InstallResult::Found(path);
    }
    // Node.js 可用
    if has_command("npx") || has_command("node") {
        return InstallResult::Installing;
    }
    // Python 可用
    if has_command("python") || has_command("python3") {
        return InstallResult::Installing;
    }
    // 都没有
    InstallResult::RequiresNodeJs
}

/// 运行官方 CLI 安装 CloakBrowser
pub async fn install_via_cli() -> Result<PathBuf, String> {
    // 优先 npx
    if has_command("npx") {
        run_install("npx", &["cloakbrowser", "install"]).await?;
    } else if has_command("python3") {
        run_install("python3", &["-m", "cloakbrowser", "install"]).await?;
    } else if has_command("python") {
        run_install("python", &["-m", "cloakbrowser", "install"]).await?;
    } else if has_command("node") {
        // node 没有 npx，尝试直接运行
        run_install("npx", &["--yes", "cloakbrowser", "install"]).await?;
    } else {
        return Err("未检测到 Node.js 或 Python".into());
    }

    // 等待安装完成后重新查找
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    find_cloak_browser()
        .ok_or_else(|| "安装完成但未找到 CloakBrowser".into())
}

async fn run_install(cmd: &str, args: &[&str]) -> Result<(), String> {
    let mut child = tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 {} 失败: {}", cmd, e))?;

    let status = child.wait().await
        .map_err(|e| format!("等待安装完成失败: {}", e))?;

    if !status.success() {
        return Err(format!("安装命令 {} 返回非零退出码", cmd));
    }
    Ok(())
}

fn has_command(cmd: &str) -> bool {
    let check = if cfg!(target_os = "windows") { "where" } else { "which" };
    Command::new(check)
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
