use std::process::Stdio;
use std::sync::Mutex;
use std::collections::HashMap;

// Instance tracker: account_id -> (pid, cdp_port)
// Updated by open_session_browser, read by switch_browser_session
static BROWSER_INSTANCES: std::sync::OnceLock<Mutex<HashMap<String, (u32, u16)>>> = std::sync::OnceLock::new();

fn instances() -> &'static Mutex<HashMap<String, (u32, u16)>> {
    BROWSER_INSTANCES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_platform_url(platform: &str) -> &str {
    match platform {
        "whatsapp" => "https://web.whatsapp.com",
        "telegram" => "https://web.telegram.org",
        "facebook" => "https://www.facebook.com",
        "instagram" => "https://www.instagram.com",
        "x" => "https://x.com",
        "line" => "https://access.line.me",
        _ => "https://web.whatsapp.com",
    }
}

/// 启动 CloakBrowser + 指纹 + 注入 + 隐藏 MetaChat
#[tauri::command]
pub async fn open_session_browser(
    window: tauri::Window,
    platform: String,
    account_id: String,
) -> Result<u16, String> {
    let cloak_path = crate::cloak::process::find_cloak_browser()
        .ok_or_else(|| "CloakBrowser 未安装".to_string())?;
    let data_dir = dirs::data_dir()
        .ok_or("找不到用户数据目录")?
        .join("metachat").join("profiles").join(&account_id);
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("创建 profile 失败: {}", e))?;
    let port = 9222;
    log::info!("Launching {}", account_id);

    // 直接指定最终位置启动，不做任何二次移动
    let mut child = std::process::Command::new(&cloak_path)
        .arg(format!("--remote-debugging-port={}", port))
        .arg(format!("--user-data-dir={}", data_dir.display()))
        .arg("--no-first-run").arg("--no-default-browser-check")
        .arg("--disable-extensions")
        .arg("--window-position=320,140")
        .arg("--window-size=1280,800")
        .arg(format!("--app={}", get_platform_url(&platform)))
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().map_err(|e| format!("启动失败: {}", e))?;

    let pid = child.id();
    log::info!("CloakBrowser started PID={}", pid);
    let _ = crate::cloak::job::assign(pid);
    // 记录实例到全局跟踪表
    {
        let mut map = instances().lock().unwrap();
        map.insert(account_id.clone(), (pid, port));
    }

    // 等 CDP 就绪
    let page_ws = loop {
        let url = format!("http://127.0.0.1:{}/json", port);
        if let Ok(resp) = reqwest::get(&url).await {
            if let Ok(body) = resp.text().await {
                if let Ok(targets) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                    if let Some(page) = targets.iter().find(|t| t["type"] == "page") {
                        if let Some(ws) = page["webSocketDebuggerUrl"].as_str() {
                            break ws.to_string();
                        }
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    };
    log::info!("CDP ready");

    // 去掉标题栏/关闭按钮，纯内容模式
    crate::cloak::position::remove_window_chrome(pid);

    // 隐藏 MetaChat
    window.hide().map_err(|e| format!("隐藏窗口失败: {}", e))?;

    // 指纹 + 导航 + 注入
    match crate::cloak::cdp_client::CdpClient::connect(&page_ws).await {
        Ok(cdp) => {
            // 注册 Ember Beads 切换绑定
            let _ = cdp.call_method("Runtime.addBinding", serde_json::json!({
                "name": "__metachat_switch"
            })).await;

            // 监听珠子点击事件 → 切换账号
            let mut event_rx = cdp.subscribe();
            let _switch_pid = pid;
            let _platform = platform.clone();
            tokio::spawn(async move {
                loop {
                    match event_rx.recv().await {
                        Ok(evt) => {
                            if evt.method == "Runtime.bindingCalled" {
                                if let Some(aid) = evt.params.get("payload").and_then(|v| v.as_str()) {
                                    log::info!("Bead switch to: {}", aid);
                                    let _ = crate::commands::browser::switch_browser_session(
                                        _platform.clone(), aid.to_string()
                                    ).await;
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            let fp = crate::cloak::fingerprint::FingerprintConfig::generate(&account_id, "windows", "Chrome 146");
            if let Err(e) = fp.apply_via_cdp(&cdp).await {
                log::warn!("指纹: {}", e);
            }
            // 注入在前，导航在后 → 脚本在导航加载的新页面上生效
            if let Err(e) = cdp.navigate(get_platform_url(&platform)).await {
                log::warn!("导航: {}", e);
            }
            // 等待页面加载后注入 UI + 推送珠子数据
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            let _ = crate::cloak::injector::inject_notification_system_now(&cdp).await;
            let beads_json = format!(r#"[{{"id":"{}","name":"{}","unread":0,"platform":"{}"}}]"#, 
                account_id, account_id, platform);
            let _ = crate::cloak::injector::update_beads_state(&cdp, &beads_json).await;
        }
        Err(e) => log::warn!("CDP: {}", e),
    }

    // 后台监控浏览器退出时恢复 MetaChat
    use tauri::Manager;
    let app_handle = window.app_handle().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            match child.try_wait() {
                Ok(Some(_)) => {
                    log::info!("浏览器退出，恢复 MetaChat");
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.show();
                    }
                    break;
                }
                Ok(None) => continue,
                Err(_) => break,
            }
        }
    });

    Ok(port)
}

/// 切换账号 — 把当前账号窗口移出屏幕，目标账号窗口移入屏幕
#[tauri::command]
pub async fn switch_browser_session(
    platform: String, account_id: String,
) -> Result<u16, String> {
    let map = instances().lock().map_err(|e| format!("锁定失败: {}", e))?;

    // 查找目标账号
    let (target_pid, target_port) = map.get(&account_id)
        .ok_or_else(|| format!("账号 {} 未启动，请先打开", account_id))?
        .clone();

    // 隐藏所有其他窗口
    for (aid, (pid, _)) in map.iter() {
        if *aid != account_id {
            let _ = crate::cloak::position::hide_window(*pid);
        }
    }

    // 显示目标窗口到正确位置
    crate::cloak::position::show_window(target_pid, 320, 140, 1280, 800);
    log::info!("Switched to {} / {} (PID={})", platform, account_id, target_pid);
    drop(map);

    Ok(target_port)
}

/// 关闭浏览器 + 显示 MetaChat
#[tauri::command]
pub fn close_browser(window: tauri::Window) -> Result<(), String> {
    #[cfg(windows)] {
        std::process::Command::new("taskkill")
            .args(["/f", "/im", "chrome.exe"])
            .output().ok();
    }
    window.show().map_err(|e| format!("显示失败: {}", e))?;
    Ok(())
}