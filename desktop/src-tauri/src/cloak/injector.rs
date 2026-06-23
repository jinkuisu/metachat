use super::cdp_client::CdpClient;

/// 注入通知系统 UI 到 CloakBrowser 页面
pub async fn inject_notification_system(cdp: &CdpClient) -> Result<(), String> {
    let js = build_injection_js();
    cdp.call_method("Page.addScriptToEvaluateOnNewDocument", serde_json::json!({
        "source": js,
    })).await.map_err(|e| format!("注入通知系统失败: {}", e))?;
    log::info!("通知系统已注入到页面");
    Ok(())
}

/// 注入通知系统 UI 到当前已加载的页面 (通过 CDP Runtime.evaluate)
/// 与 inject_notification_system 不同，这个在页面加载后调用
pub async fn inject_notification_system_now(cdp: &CdpClient) -> Result<(), String> {
    let js = build_injection_js();
    cdp.call_method("Runtime.evaluate", serde_json::json!({
        "expression": js,
    })).await.map_err(|e| format!("注入通知系统(当前页)失败: {}", e))?;
    log::info!("通知系统已注入到当前页面");
    Ok(())
}

/// 更新已注入页面的通知状态 (通过 CDP Runtime.evaluate)
pub async fn update_beads_state(cdp: &CdpClient, accounts_json: &str) -> Result<(), String> {
    let js = format!(
        "window.__metachat?.updateBeads && window.__metachat.updateBeads({})",
        accounts_json
    );
    cdp.evaluate(&js).await.map_err(|e| format!("更新通知状态失败: {}", e))?;
    Ok(())
}

fn build_injection_js() -> String {
    r##"
(function() {
    'use strict';
    if (window.__metachat) return; // 防止重复注入

    // CDP binding hook — 被 Runtime.addBinding 覆盖后变成 Rust 通信通道
    window.__metachat_switch = window.__metachat_switch || function(id) {
        console.log("MetaChat switch fallback:", id);
    };
    const MC = window.__metachat = {};
    const doc = document;

    // ── 样式 ──
    const style = doc.createElement('style');
    style.textContent = `
        .mc-beads { position:fixed; left:8px; top:50%; transform:translateY(-50%); z-index:99999; display:flex; flex-direction:column; gap:6px; pointer-events:none; }
        .mc-bead { width:6px; height:6px; border-radius:50%; background:rgba(0,0,0,0.12); cursor:pointer; transition:all 0.3s; pointer-events:auto; }
        .mc-bead:hover { width:180px; height:28px; border-radius:8px; background:rgba(255,255,255,0.92); backdrop-filter:blur(12px); padding:0 8px; display:flex; align-items:center; }
        .mc-bead.has-unread { background:#25D366; box-shadow:0 0 6px #25D366; animation:bead-pulse 2s ease infinite; }
        @keyframes bead-pulse { 0%,100%{opacity:1;transform:scale(1)} 50%{opacity:0.5;transform:scale(1.4)} }
        .mc-toolbar { position:fixed; top:12px; right:12px; z-index:99999; display:flex; gap:2px; background:rgba(255,255,255,0.4); backdrop-filter:blur(12px); padding:4px; border-radius:14px; pointer-events:auto; }
        .mc-btn { width:28px; height:28px; border-radius:10px; border:none; background:transparent; cursor:pointer; display:flex; align-items:center; justify-content:center; font-size:14px; }
        .mc-btn:hover { background:rgba(0,0,0,0.06); }
        .mc-back { position:fixed; top:12px; left:12px; z-index:99999; width:32px; height:32px; border-radius:10px; border:none; background:rgba(255,255,255,0.3); backdrop-filter:blur(8px); cursor:pointer; display:flex; align-items:center; justify-content:center; font-size:18px; opacity:0.15; transition:opacity 0.2s; pointer-events:auto; }
        .mc-back:hover { opacity:0.8; background:rgba(255,255,255,0.85); }
    `;
    doc.head.appendChild(style);

    // ── 返回按钮 ──
    const back = doc.createElement('button');
    back.className = 'mc-back';
    back.textContent = '←';
    back.onclick = () => {
        // 通过 CDP 通信通知 Rust (通过自定义事件)
        const evt = new CustomEvent('__metachat_back');
        doc.dispatchEvent(evt);
    };
    doc.body.appendChild(back);

    // ── 工具岛 ──
    const toolbar = doc.createElement('div');
    toolbar.className = 'mc-toolbar';
    ['🤖','⚡','🌐'].forEach(icon => {
        const btn = doc.createElement('button');
        btn.className = 'mc-btn';
        btn.textContent = icon;
        toolbar.appendChild(btn);
    });
    doc.body.appendChild(toolbar);

    // ── Ember Beads 容器 ──
    // ── 更新 Beads 的方法（提前定义，body 无关）──
    MC.updateBeads = function(accounts) {
        // 容器由 initUI 创建，如果还没创建就跳过
        const container = ensureBeadsContainer();
        if (!container) return;
        container.innerHTML = '';
        (accounts || []).forEach(acc => {
        const container = doc.getElementById('mc-beads');
        if (!container) return;
        container.innerHTML = '';
        (accounts || []).forEach(acc => {
            const bead = doc.createElement('div');
            bead.className = 'mc-bead' + (acc.unread > 0 ? ' has-unread' : '');
            bead.title = acc.name + (acc.unread > 0 ? ' (' + acc.unread + ' 未读)' : '');
            bead.onclick = () => {
                // 通过 CDP Runtime.addBinding 通知 Rust 切换账号
                if (typeof window.__metachat_switch === "function") {
                    window.__metachat_switch(acc.id);
                }
            };
            container.appendChild(bead);
        });
    };

    console.log('MetaChat UI injected');
})();
"##.to_string()
}
