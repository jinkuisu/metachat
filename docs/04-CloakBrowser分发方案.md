# CloakBrowser 分发方案 v2

## 合规原则

MetaChat 不打包、不托管、不分发 CloakBrowser 二进制文件。
用户主动触发下载 → 通过官方渠道获取 → 完全合规。

官方许可：free to use, no redistribution

## 技术方案

```
用户勾选 "下载 CloakBrowser" 或首次使用浏览器模式时
         ↓
Rust 后端检测可用运行时
  ├─ Node.js 可用  → npx cloakbrowser install
  ├─ Python 可用   → python -m cloakbrowser install
  └─ 都不可用       → 弹出提示引导用户安装 Node.js/Python
         ↓
官方 CLI 下载二进制到 ~/.cloakbrowser/
         ↓
MetaChat 检测到二进制存在 → 路径写入 AppConfig
         ↓
process.rs 启动 CloakBrowser + --remote-debugging-port
         ↓
cdp_client.rs 连接 → fingerprint.rs 设置指纹 → 打开平台页面
```

## AppConfig 配置

pub struct AppConfig {
    // ... 已有字段
    pub cloak_binary_path: Option<PathBuf>,  // ~/.cloakbrowser/ 下的二进制路径
    pub cloak_auto_install: bool,             // 是否自动安装
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cloak_binary_path: None,  // 初始无路径，下载后更新
            cloak_auto_install: true, // 首次使用自动提示安装
            // ...
        }
    }
}

## CloakBrowser 路径发现

优先顺序：
1. ~/.cloakbrowser/  (官方默认路径)
2. node_modules/cloakbrowser/  (开发环境)
3. 环境变量 CLOAK_BROWSER_PATH
4. 系统 PATH 中的 cloak-browser

检测代码 (Rust)：

pub fn find_cloak_browser() -> Option<PathBuf> {
    // 1. 官方默认路径
    let home = dirs::home_dir()?;
    let default = home.join(".cloakbrowser");
    let exe_name = if cfg!(windows) { "chrome.exe" } else { "chrome" };
    
    for dir in [&default, &PathBuf::from("node_modules/cloakbrowser")] {
        let exe = dir.join(exe_name);
        if exe.exists() { return Some(exe); }
    }
    
    // 2. 环境变量
    if let Ok(path) = std::env::var("CLOAK_BROWSER_PATH") {
        let p = PathBuf::from(path);
        if p.exists() { return Some(p); }
    }
    
    None
}

## 安装器实现 (Rust)

use std::process::Command;

pub async fn install_cloak_browser(
    on_progress: impl Fn(String)  // 状态回调给前端
) -> Result<PathBuf, String> {
    // 先检查是否已安装
    if let Some(path) = find_cloak_browser() {
        return Ok(path);
    }

    // 尝试 npx
    if which("npx").is_ok() {
        on_progress("正在通过 npm 安装 CloakBrowser (约 200MB)...".into());
        let output = Command::new("npx")
            .args(["cloakbrowser", "install"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 npx 失败: {}", e))?;
        
        // 等待完成
        // (实际用 tokio::process::Command 异步等待)
        on_progress("安装完成，正在验证...".into());
    }
    // 尝试 python
    else if which("python").is_ok() || which("python3").is_ok() {
        let python = if which("python3").is_ok() { "python3" } else { "python" };
        on_progress("正在通过 pip 安装 CloakBrowser...".into());
        Command::new(python)
            .args(["-m", "cloakbrowser", "install"])
            .spawn()
            .map_err(|e| format!("启动 python 失败: {}", e))?;
    }
    // 都没有
    else {
        return Err(
            "未检测到 Node.js 或 Python。请先安装 Node.js (https://nodejs.org) \
             或 Python (https://python.org)，然后重试。".into()
        );
    }

    // 等待安装完成后重新检查路径
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    find_cloak_browser()
        .ok_or_else(|| "安装完成但未找到 CloakBrowser 二进制文件".into())
}

// 检查命令是否存在
fn which(cmd: &str) -> Result<(), ()> {
    Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(cmd)
        .output()
        .map(|o| if o.status.success() { Ok(()) } else { Err(()) })
        .unwrap_or(Err(()))
}

## 前端 UI

在设置页面或首次使用浏览器模式时弹出：

┌──────────────────────────────────────────────┐
│                                                │
│     ┌────────────────────────────────┐        │
│     │  需要 CloakBrowser 引擎         │        │
│     │                                │        │
│     │  MetaChat 需要 CloakBrowser     │        │
│     │  来加载社交平台页面。            │        │
│     │                                │        │
│     │  CloakBrowser 是开源浏览器      │        │
│     │  专注于隐私和反指纹检测。         │        │
│     │                                │        │
│     │  文件大小: ~200MB               │        │
│     │  来源: CloakBrowser 官方       │        │
│     │                                │        │
│     │  [  下载并安装  ]  [  稍后再说 ]│        │
│     └────────────────────────────────┘        │
│                                                │
└──────────────────────────────────────────────┘

下载中：

┌──────────────────────────────────────────────┐
│     ┌────────────────────────────────┐        │
│     │  📥 正在下载 CloakBrowser...    │        │
│     │                                │        │
│     │  ████████████░░░░░░  65%       │        │
│     │  正在通过官方渠道下载...         │        │
│     │  约 200 MB                      │        │
│     └────────────────────────────────┘        │
└──────────────────────────────────────────────┘

## 开发环境 vs 生产环境

开发环境:
  npm install cloakbrowser (在 desktop/ 目录下)
  二进制在 node_modules/cloakbrowser/
  process.rs 直接从此路径启动

生产环境:
  用户通过设置页面触发安装
  二进制在 ~/.cloakbrowser/
  通过 npx/python 官方命令安装
  找到后 process.rs 从此路径启动
