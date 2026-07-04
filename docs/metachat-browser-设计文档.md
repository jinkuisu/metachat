﻿# MetaChat Browser · 技术设计方案

> 版本：v1.0
> 日期：2026-06-24

---

## 一、项目概述

MetaChat Browser 是一个基于 Chromium 源码的多账号隔离浏览器，核心能力：
- 基于 Chromium 150.0.7844.0 + ChromiumFish 21 个反指纹补丁
- 每个账号独立指纹（C++ 引擎级别，非 JS 注入）
- 每个账号独立进程隔离（Cookie/缓存/代理完全隔离）
- 悬浮式 UI，沉浸式多账号管理体验
- 消息总线架构，引擎层与 UI 层松耦合

---

## 二、技术路线

### 2.1 核心架构

放弃 CEF 方案，直接编译 chrome.exe：

`
Chromium 150.0.7844.0 源码
  + ChromiumFish 21 个补丁（指纹引擎）
  + MetaChat 扩展（UI/chrome://metachat/ 注册，其余在 UI 层实现）
  = MetaChat Browser (metachat.exe)
`

### 2.2 分层设计

| 层级 | 技术 | 职责 | 稳定度 |
|------|------|------|--------|
| 引擎层 | C++ patches/ | per-WebContents 指纹、WebContents 管理、Overlay 容器、CDP 桥 | ⭐ 稳定不改 |
| UI 层 | WebUI (HTML/CSS/JS) ui/ | 所有界面逻辑、账号管理、搜索、通知 | 🔄 可独立迭代 |
| 通信层 | 消息总线 | WebUI ↔ C++ 统一通信通道 | ⭐ 稳定不改 |

---

## 三、引擎层设计（C++ Patches）

### 3.1 补丁清单

`
metachat/patches/
├── 0001-branding.patch              # 品牌更名
├── 0002-switches.patch              # --metachat-mode 启动参数
├── 0003-hide-chrome-ui.patch        # 隐藏 tab strip/地址栏/菜单
├── 0004-fingerprint-per-tab.patch   # per-WebContents 指纹隔离
├── 0005-web-contents-manager.patch  # WebContents 创建/切换/visibility
├── 0006-overlay-container.patch     # 通用 Overlay 容器
├── 0007-cdp-bridge.patch            # CDP 代理桥
└── 0008-startup.patch               # 默认启动 chrome://metachat/
`

### 3.2 启动参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| --metachat-mode | 开关 | 无 | 启用 MetaChat 模式（隐藏原生 UI） |
| --metachat-mode-allowlist | 字符串 | 空 | 逗号分隔的 IDC 命令 ID，即使在默认拦截列表中也放行 |
| --metachat-mode-blocklist | 字符串 | 空 | 逗号分隔的 IDC 命令 ID，额外拦截 |
| --metachat-accounts-config | 路径 | accounts.json | 账号配置文件路径 |

启动示例：
```bash
MetaChat.exe --metachat-mode --metachat-accounts-config=C:/Users/xxx/metachat-accounts.json
```

### 3.3 核心机制：per-profile 指纹隔离

#### 3.3.1 背景

ChromiumFish 的 `fingerprinting::Current()` 是 **进程级单例**，所有 WebContents 共享同一个指纹。
MetaChat 需要**每个账号独立指纹**，不同账号的指纹必须完全隔离（C++ 引擎级别，非 JS 注入）。

#### 3.3.2 方案：每账号独立 Profile + AppendExtraCommandLineSwitches 注入

```
browser 进程
  MetaChatAccountStore (管理 accounts.json)
    |- Account A -> user_data_dir=Profile_A, persona_seed=6847...
    |- Account B -> user_data_dir=Profile_B, persona_seed=1928...
    |- Account C -> user_data_dir=Profile_C, persona_seed=8374...

  ChromeContentBrowserClient::AppendExtraCommandLineSwitches()
    |- child_process_id -> RenderProcessHost::FromID() OK（构造函数已注册）
    |- RPH -> GetBrowserContext() -> GetPath() -> profile 路径
    |- 查 MetaChatAccountStore 拿到 persona_seed
       -> command_line->AppendSwitchASCII(persona-seed, seed)

renderer 进程 A                renderer 进程 B
  persona-seed=A               persona-seed=B
  DeriveFromSeed               DeriveFromSeed
  -> 指纹 A                    -> 指纹 B
```

**关键保证：**

- **不同 profile 的 renderer 进程天然隔离** -- Chromium 不会让不同 BrowserContext 的 WebContents 复用同一 renderer 进程
- **RenderProcessHost::FromID() 在 AppendExtraCommandLineSwitches 调用时已可用** -- RegisterHost() 在构造函数中调用，早于 Init() 中 AppendExtraCommandLineSwitches 的调用
- **Fish 的 kSwitchNames[] 仍保留 kPersonaSeed** -- 作为全局 fallback（Spare RenderProcessHost 等场景）

#### 3.3.3 补丁修改清单

| 补丁 | 修改内容 |
|------|---------|
| persona-seed.patch（Fish） | 不修改。保留 kPersonaSeed 在 kSwitchNames[] 中作为 fallback |
| 0004-profile-seed.patch（新） | ChromeContentBrowserClient::AppendExtraCommandLineSwitches() 中通过 child_process_id -> RPH -> BrowserContext -> persona_seed 覆盖注入 |

#### 3.3.4 accounts.json 格式

```json
{
  "version": 1,
  "accounts": [
    {
      "id": "wa_work",
      "name": "工作号",
      "platform": "whatsapp",
      "url": "https://web.whatsapp.com",
      "user_data_dir": "Profile_WhatsApp_Work",
      "persona_seed": "684729103847201",
      "proxy": null
    },
    {
      "id": "tg_personal",
      "name": "个人号",
      "platform": "telegram",
      "url": "https://web.telegram.org",
      "user_data_dir": "Profile_Telegram_Personal",
      "persona_seed": "192837465012384",
      "proxy": null
    }
  ]
}
```

> **MVP 阶段只开放 persona_seed**。用户只需要给不同账号不同 seed，Fish 的 DeriveFromSeed() 自动生成全套一致的指纹（UA、WebGL、音频、Canvas、屏幕等 30+ 维度）。
>
> **Phase 2** 可扩展 persona_overrides 字段，支持精细控制特定维度，同时保持 Fish 默认池的一致性保证。

#### 3.3.5 后台账号管理

后台账号的 WebContents 设为 OCCLUDED 状态：
- JS 继续运行（能接收消息）
- 停止渲染/动画
- 释放显存
- 切换到前台时立即恢复全功率

#### 3.3.6 Phase 2 扩展：persona_overrides

```json
{
  "accounts": [
    {
      "id": "wa_1",
      "persona_seed": "111",
      "persona_overrides": {
        "os": "mac",
        "screen_width": 2560,
        "screen_height": 1600,
        "hardware_concurrency": 10,
        "webgl_renderer": "Apple M2",
        "timezone": "America/New_York",
        "locale": "en-US"
      }
    }
  ]
}
```

支持按 **指纹模板** 管理模板库，多账号共享同一模板。### 3.4 通用 Overlay 容器

引擎层只提供一个 Overlay 容器（Widget），用于承载所有悬浮 UI：

`
┌───────────────────────────────────────────────┐
│  BrowserView                                  │
│                                               │
│  ┌─────────────────────────────────────────┐  │
│  │  ContentsWebView (WebContents)          │  │
│  └─────────────────────────────────────────┘  │
│                                               │
│  ┌── OverlayContainer ──────────────────────┐  │
│  │  Layer 1: 悬浮面板                       │  │
│  │  Layer 2: 弹窗/对话框                    │  │
│  │  Layer 3: 消息通知                       │  │
│  │  Layer 4: 悬浮球（最顶层）               │  │
│  └──────────────────────────────────────────┘  │
└───────────────────────────────────────────────┘
`

Overlay 支持两种模式：
- **浮动覆盖**：独立 Widget 层，盖在网页上方，可拖动
- **固定侧边**：网页右移，面板固定在左侧

---

## 四、UI 层设计

### 4.1 目录结构

`
metachat/ui/
├── manifest.json                    # UI 模块注册
├── core/
│   ├── message-bus.js               # 消息总线（C++ ↔ WebUI 通信）
│   └── storage.js                   # 本地存储封装
├── pages/
│   ├── home/                        # 首页（平台卡片总览）
│   │   ├── home.html
│   │   ├── home.css
│   │   └── home.js
│   └── workspace/                   # 工作台（悬浮面板 + 通知）
│       ├── workspace.html
│       ├── workspace.css
│       └── workspace.js
├── components/                      # 可复用组件
│   ├── floating-ball/               # 悬浮球
│   ├── account-panel/               # 悬浮面板（平台+账号列表）
│   ├── notification-toast/          # 消息通知弹窗
│   ├── search-panel/                # Cmd+K 搜索面板
│   ├── platform-bar/                # 平台横排切换
│   └── account-list/                # 竖排账号列表
└── styles/
    ├── variables.css                # CSS 变量
    └── theme.css                    # 主题样式
`

### 4.2 页面流程

`
启动 → chrome://metachat/（首页）
  → 用户看到平台卡片总览（WhatsApp/Telegram/Facebook...）
    → 点击平台卡片
      → 显示该平台下的账号列表
        → 点击账号
          → 引擎层创建 WebContents
          → 设置指纹 seed
          → 切换到工作台视图
          → 账号网页全屏显示
          → 悬浮球出现

工作台视图交互：
  ├── 点击悬浮球 → 展开悬浮面板
  │   ├── 查看所有平台及其未读统计
  │   ├── 切换账号
  │   ├── Cmd+K 全局搜索
  │   └── 📌 切换固定/浮动模式
  ├── 新消息 → 右下角通知弹窗（淡入淡出）
  ├── 固定模式 → 面板固定在左侧，网页右移
  └── 点击悬浮球收起 → 面板关闭，回到全屏
`

### 4.3 首页设计

`
┌──────────────────────────────────────────────────────┐
│                                                       │
│                    MetaChat                           │
│                                                       │
│         ┌────────────┐   ┌────────────┐              │
│         │  WhatsApp   │   │  Telegram   │             │
│         │   3个账号    │   │   2个账号   │             │
│         │   2条未读    │   │   0条未读   │             │
│         └────────────┘   └────────────┘              │
│                                                       │
│         ┌────────────┐   ┌────────────┐              │
│         │  Facebook   │   │    X       │              │
│         │   1个账号    │   │   2个账号   │             │
│         │   5条未读    │   │   1条未读   │             │
│         └────────────┘   └────────────┘              │
│                                                       │
│                   [+ 添加平台]                        │
│                                                       │
└──────────────────────────────────────────────────────┘
`

用户点击平台卡片 → 进入该平台工作台。

### 4.4 工作台交互

`
工作台视图（点击账号进入后）：
┌──────────────────────────────────────────────────┐
│                                                    │
│             当前账号网页（全屏沉浸）                  │
│                                                    │
│                                                    │
│                                                    │
│                                                    │
│                     🟣 （悬浮球）                  │
│                                                    │
└────────────────────────────────────────────────────┘

点击悬浮球展开面板：
┌──────────────────────────────────────────────────┐
│  🏠 首页    📌 固定    🗑 关闭                     │
│  🔍 搜索账号/备注/联系人...                       │
│                                                    │
│  📱 WhatsApp        3在线 · 2未读                 │
│  ✈ Telegram        2在线 · 3未读                 │
│  📘 Facebook        1离线                         │
│                                                    │
│  ── 账号列表 ──                                  │
│  ○ 账号A  ← 当前                   2未读          │
│  ○ 账号B                              1未读       │
│  ○ 账号C                                            │
│  ○ 账号D                              3未读       │
│  ○ 账号E                                            │
│                                                    │
│  [+ 添加账号]                                     │
└────────────────────────────────────────────────────┘

鼠标悬停账号卡片展开详情：
┌─────────────────┐
│  👤 账号A        │
│  ─────────────  │
│  未读: 2条      │
│  状态: 在线      │
│  最后活动: 刚刚  │
│  实时: 录音中    │
└─────────────────┘

右下角消息通知：
┌──────────────────────┐
│ 📱 账号A              │
│ 张三: 好的，马上到    │
│ 刚刚                  │
└──────────────────────┘
`

---

## 五、消息总线设计

### 5.1 架构

WebUI 与 C++ 之间不通过多个零散的 chrome.send() 调用，而是统一走一个消息总线。

`
WebUI (MessageBus.js)
  │
  │ chrome.send('metachat.dispatch', [channel, action, payload, requestId, meta])
  ▼
C++ (MetaChatMessageBus::HandleDispatch)
  │
  │ 根据 channel 路由到对应 Handler
  ▼
AccountsHandler / WebContentsHandler / CdpBridge / ConfigHandler
  │
  │ CallJavascriptFunction('__metachat.onMessage', { channel, event, payload, meta })
  ▼
WebUI (__metachat.onMessage)
`

### 5.2 消息格式

`	ypescript
// ====== 请求（WebUI → C++） ======
interface MessageRequest {
    channel: string;          // 路由通道
    action: string;           // 操作名
    payload?: any;            // 参数
    requestId: string;       // 请求 ID（自动生成，用于匹配响应）
    meta?: {
        timestamp: number;         // 发送时间
        source?: string;           // 来源标识（可选）
        traceId?: string;          // 追踪链 ID
        sessionId?: string;        // 会话 ID（可选）
        debug?: boolean;           // 调试日志开关
        note?: string;             // 备注说明
    };
}

// ====== 响应（C++ → WebUI） ======
interface MessageResponse {
    channel: string;
    action: string;
    payload?: any;
    requestId: string;
    error?: string;
    meta?: {
        timestamp: number;
        processingTime: number;    // 处理耗时（ms）
        source?: string;
        debug?: boolean;
        note?: string;
    };
}

// ====== 事件推送（C++ → WebUI，主动推送） ======
interface MessageEvent {
    channel: string;
    event: string;               // 事件名
    payload: any;
    meta?: {
        timestamp: number;
        severity?: 'info' | 'warn' | 'error';
        source?: string;
        note?: string;
        retryCount?: number;
    };
}
`

### 5.3 通道定义

| channel | 方向 | 说明 | 主要 actions |
|---------|------|------|-------------|
| ccounts | 双向 | 账号管理 | list create emove update switch |
| webs | 双向 | WebContents 管理 | create switch destroy set-visibility notify |
| ingerprint | 双向 | 指纹管理 | set-seed get-profile |
| cdp | 双向 | CDP 代理 | evaluate subscribe unsubscribe call-function |
| notification | 单向(C++→JS) | 推送通知 | (event only) |
| config | 双向 | 配置存储 | save load delete |
| system | 双向 | 系统操作 | get-info open-url quit |

### 5.4 MessageBus.js（WebUI 端）

`javascript
// ui/core/message-bus.js

class MessageBus {
    constructor() {
        this._handlers = {};          // 事件监听器
        this._pending = {};           // 待响应请求
        this._requestCounter = 0;
    }

    // 发送请求并等待响应
    send(channel, action, payload = {}, meta = {}) {
        const requestId = eq_;
        chrome.send('message', [{
            channel, action, payload, requestId,
            meta: { timestamp: Date.now(), ...meta }
        }]);
        return new Promise((resolve, reject) => {
            this._pending[requestId] = { resolve, reject, startTime: Date.now() };
            setTimeout(() => {
                if (this._pending[requestId]) {
                    reject(new Error(Request  timeout));
                    delete this._pending[requestId];
                }
            }, 30000);
        });
    }

    // 只发送，不等待响应
    notify(channel, action, payload = {}, meta = {}) {
        chrome.send('message', [{
            channel, action, payload,
            meta: { timestamp: Date.now(), ...meta }
        }]);
    }

    // 监听事件
    on(channel, event, handler) {
        const key = ${channel}:;
        if (!this._handlers[key]) this._handlers[key] = [];
        this._handlers[key].push(handler);
    }

    off(channel, event, handler) {
        const key = ${channel}:;
        this._handlers[key] = this._handlers[key]?.filter(h => h !== handler);
    }

    // C++ 回调入口
    onEvent(msg) {
        const { channel, event, payload, requestId, meta } = msg;

        // 如果是请求响应
        if (requestId && this._pending[requestId]) {
            const { resolve, reject } = this._pending[requestId];
            delete this._pending[requestId];
            if (msg.success) {
                resolve({ payload, meta });
            } else {
                reject(new Error(msg.error || 'Unknown error'));
            }
            return;
        }

        // 如果是事件推送
        const key = ${channel}:;
        const handlers = this._handlers[key];
        if (handlers) {
            handlers.forEach(h => h({ payload, meta }));
        }
    }
}

window.MessageBus = new MessageBus();
`

### 5.5 C++ 端（MetaChatMessageBus）

`cpp
// chrome/browser/ui/webui/metachat/message_bus.h

class MetaChatMessageBus : public content::WebUIMessageHandler {
 public:
  void RegisterMessages() override;
  void DispatchEvent(const std::string& channel,
                     const std::string& event,
                     base::Value payload,
                     base::Value::Dict meta = {});
 private:
  void HandleDispatch(const base::Value::List& args);
  std::unique_ptr<AccountsHandler> accounts_handler_;
  std::unique_ptr<WebContentsHandler> web_contents_handler_;
  std::unique_ptr<CdpBridge> cdp_bridge_;
  std::unique_ptr<ConfigHandler> config_handler_;
};
`

`cpp
// 路由逻辑
void MetaChatMessageBus::HandleDispatch(const base::Value::List& args) {
    const auto& msg = args[0].GetDict();
    const std::string& channel = *msg.FindString("channel");
    const std::string& action = *msg.FindString("action");
    const base::Value* payload = msg.Find("payload");
    const std::string* request_id = msg.FindString("requestId");
    const base::Value::Dict* meta = msg.FindDict("meta");

    if (channel == "accounts") {
        accounts_handler_->Handle(action, payload, request_id, meta);
    } else if (channel == "webs") {
        web_contents_handler_->Handle(action, payload, request_id, meta);
    } else if (channel == "cdp") {
        cdp_bridge_->Handle(action, payload, request_id, meta);
    } else if (channel == "config") {
        config_handler_->Handle(action, payload, request_id, meta);
    }
}
`

### 5.6 使用示例

`javascript
// 创建账号
const result = await MessageBus.send('accounts', 'create', {
    platform: 'whatsapp',
    name: '账号A',
    seed: 'abc123',
    proxy: 'socks5://127.0.0.1:1080'
}, {
    source: 'home',
    note: '用户从首页添加新账号'
});

// 切换账号
await MessageBus.send('accounts', 'switch', { accountId: 'acc_001' }, {
    source: 'workspace',
    note: '用户点击账号A'
});

// 监听新消息
MessageBus.on('cdp', 'new-message', ({ payload, meta }) => {
    showNotification(payload);
});

// CDP 读取未读数
const { payload: count } = await MessageBus.send('cdp', 'evaluate', {
    webContentsId: 'acc_001',
    expression: 'document.querySelectorAll(".unread").length'
});

// 监听账号状态变更
MessageBus.on('accounts', 'status-changed', ({ payload }) => {
    updateAccountStatus(payload.accountId, payload.status);
});
`

---

## 六、账号配置存储

### 6.1 文件位置

%LOCALAPPDATA%/MetaChat/accounts.json（Windows）
~/Library/Application Support/MetaChat/accounts.json（Mac）

### 6.2 数据格式

`json
{
  "version": 1,
  "accounts": [
    {
      "id": "a1b2c3d4-...",
      "name": "账号A",
      "platform": "whatsapp",
      "url": "https://web.whatsapp.com",
      "persona_seed": "684729103847201",
      "proxy": {
        "type": "socks5",
        "host": "127.0.0.1",
        "port": 1080,
        "username": "",
        "password": ""
      },
      "created_at": 1719123456,
      "last_active": 1719234567,
      "notes": ""
    }
  ]
}
`

---

## 七、构建与发布

### 7.1 编译环境

| 平台 | 编译方式 | 说明 |
|------|----------|------|
| Windows | GitHub Actions (windows-2022) | 或自托管 runner |
| Mac | 本地或 CI | 后续支持 |

### 7.2 编译流程

`
1. fetch chromium@150.0.7844.0
2. git checkout ChromiumFish 对应 commit
3. git apply ChromiumFish 21 个补丁
4. rsync assets/ 覆盖
5. git apply MetaChat 补丁
6. autoninja -C out/Release chrome
7. 重命名为 metachat.exe
8. 打包为 metachat-windows.zip
`

### 7.3 启动方式

`ash
metachat.exe --metachat-mode
`

---

## 八、迭代计划

### Phase 1（MVP）

| # | 功能 | 优先级 |
|---|------|--------|
| 1 | Chromium + Fish 补丁编译通过 | P0 |
| 2 | 品牌更名（metachat.exe） | P0 |
| 3 | 隐藏原生 Chrome UI | P0 |
| 4 | per-WebContents 指纹 | P0 |
| 5 | 首页（chrome://metachat/） | P0 |
| 6 | 悬浮球 + 悬浮面板 | P0 |
| 7 | 账号管理（添加/切换） | P0 |
| 8 | 消息总线 | P0 |
| 9 | 新消息通知 | P1 |
| 10 | Cmd+K 搜索 | P1 |
| 11 | 后台 OCCLUDED 低功耗 | P1 |
| 12 | 固定/浮动模式切换 | P1 |

### Phase 2

| # | 功能 |
|---|------|
| 1 | CDP 代理桥完整实现 |
| 2 | 从账号网页读取未读数 |
| 3 | 搜索联系人/会话 |
| 4 | 鼠标悬停账号卡片展开详情 |

### Phase 3

| # | 功能 |
|---|------|
| 1 | AI 面板 |
| 2 | 设置页面 |
| 3 | 自动更新 |
| 4 | Mac 编译 |

---

## 九、扩展性保证

| 场景 | 需要改什么 |
|------|-----------|
| 加新的平台（如 TikTok） | 只在 ui/ 加配置 |
| 加 AI 面板 | 在 ui/components/ai-panel/ 下加组件 |
| 改界面样式 | 只改 ui/styles/ 的 CSS |
| 加新的弹窗/对话框 | 复用 Overlay 容器 |
| 加新的通道 | 在 message-bus 注册新 channel |
| 改消息数据格式 | 只改 CDP 桥，UI 层适配 |

C++ 引擎层一旦写好不再改动，所有后续功能在 WebUI 层实现。

---

## 十、设计决策记录

| 决策 | 选择 | 原因 |
|------|------|------|
| 不选 CEF | 直接编译 chrome.exe | CEF 补丁与 Fish 补丁冲突，且不需要嵌入 API |
| 不选 Tauri 中间层 | UI 直接在 Chromium WebUI 实现 | 减少架构层级，降低复杂度 |
| 不选 iframe 加载社交页面 | 独立 WebContents | X-Frame-Options 限制，且指纹/进程隔离更彻底 |
| 不选 JS 注入改指纹 | C++ 引擎级修改 | JS 注入可被检测，C++ 层改指纹不留痕迹 |
| 选消息总线而非多 chrome.send | 统一通信入口 | 方便扩展、调试、追踪 |
| 悬浮球 + 面板而非传统侧边栏 | 沉浸式体验 | 不遮挡网页，两种模式可选 |


---


## 十一、事件日志系统（待实现）

> 优先级：P1（Phase 1 MVP 完成后再做）
> 依赖：消息总线就绪、账号管理就绪、CDP 桥就绪

### 11.1 三层日志体系

| 层 | 名称 | 用途 | 持久化 | 机制 |
|----|------|------|--------|------|
| L1 | 调试日志 | 开发调试、运行流 | --log-file 文本文件 | Chromium VLOG，启动时控制 |
| L2 | 审计事件 | 用户操作追踪、AI 故障分析 | JSONL 文件 | 自定义 MetaChatEventStore |
| L3 | 业务数据 | 消息/联系人导出、统计分析 | JSONL 文件 | 自定义 MetaChatDataStore |

L1 已部分就绪（message_bus.cc 中有 DVLOG 调用），L2/L3 待实现。

### 11.2 事件格式

全文详见设计文档的十一节，包含：
- 事件字段定义（ts, event, traceId, spanId, user, channel, account, platform, elapsed, data, error, meta）
- 5 大类事件清单（用户操作 / CDP / 指纹 / 网络 / 系统）
- 文件结构（events/ 按天切割，data/ 按账号分割）
- 运行时控制（config 通道动态调整级别）
- AI 集成场景说明
- C++ 接口定义
- 实现计划（6 步，~210 行 C++）

### 11.9 消息总线协议改进（待改，和 L2 日志一起做）

两个改动点，协议冻结前必须改掉：

1. error 移到顶层：当前 payload.error → 顶层 {error: {code, message}}，JS 端统一检查 msg.error
2. 响应回传 traceId：当前响应无 traceId → 响应加 traceId 字段，保证追踪链完整

改动范围：message_bus.cc MakeResponse()/SendResponse()/SendEvent() 及 JS 端 onMessage

### 11.10 指纹配置（无需改源码）

Seed 已支持通过 accounts.json 配置。当前从 profile 路径 hash 派生，后续可改读 accounts.json persona_seed 字段直接传参。

详细指纹参数由 Fish 引擎从 seed 确定性派生。如有需要可加指纹模板系统（Phase 3）。


### 11.11 指纹配置覆盖系统（待实现，和协议修复、日志埋点一起做）

> 目标：用户可通过 accounts.json 的 persona_config 覆盖任意指纹参数，无需重新编译

#### 机制

在现有 `persona-seed.patch` 的基础增加一个 `--persona-config` 开关：

- browser 进程：`AppendExtraCommandLineSwitches()` 读取 accounts.json 中的 persona_config 字段，序列化 JSON 字符串传给渲染进程
- renderer 进程：`FingerprintProfile` 增加 `GetOverride(key)` 方法，解析 JSON 并返回覆盖值
- 每个 Fish fingerprint hook：优先查 `persona_config`，有覆盖就用覆盖，没有则降级到 seed 确定性派生

#### 改动手表

| 文件 | 改动 |
|------|------|
| content/public/common/content_switches.h | 声明 kPersonaConfig 开关名 |
| content/public/common/content_switches.cc | 定义 kPersonaConfig |
| third_party/blink/public/common/fingerprinting/fingerprint_profile.h | 声明 GetOverride(key) |
| third_party/blink/public/common/fingerprinting/fingerprint_profile.cc | 实现 JSON 解析 + GetOverride |
| 各个 Fish fingerprint .cc（约 8-10 处） | 每个 hook 加一行：if (auto v = profile.GetOverride("key")) return v; |
| ChromeContentBrowserClient::AppendExtraCommandLineSwitches() | 读取 accounts.json persona_config，传入 --persona-config |

#### 用户配置格式

```json
{
  "id": "wa_work",
  "persona_seed": "abc123",
  "persona_config": {
    "navigator_platform": "Win32",
    "navigator_hardware_concurrency": 8,
    "device_memory": 8,
    "screen_width": 1920,
    "screen_height": 1080,
    "screen_color_depth": 24,
    "timezone": "Asia/Shanghai",
    "locale": "zh-CN",
    "canvas_noise": 0.3,
    "webrtc_enabled": false,
    "webgl_vendor": "Google Inc. (Intel)",
    "webgl_renderer": "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0)",
    "audio_context_sample_rate": 48000
  }
}
```

不在 `persona_config` 中的字段自动降级到 seed 派生（Fish 原有行为）。新字段只需在 C++ 的 Fish hook 加一行读取，**无需改 config 解析逻辑**。

### 11.12 日志埋点（和最终编译一起做）

所有 C++ 改动点加一行日志调用，避免后续为加日志重新编译。

#### 埋点清单

| C++ 改动点 | 埋什么 | 级别 |
|------------|--------|------|
| message_bus.cc HandleDispatch | 每次 dispatch 的 channel/action/requestId | DEBG |
| message_bus.cc HandleAccounts | 账号 CRUD 操作结果 | INFO |
| message_bus.cc HandleWebs | WebContents 创建/切换/销毁 | INFO |
| message_bus.cc HandleCdp | CDP 命令/结果/超时 | DEBG |
| message_bus.cc HandleConfig | 配置变更 | INFO |
| message_bus.cc HandleNotification | 推送通知事件 | DEBG |
| persona-seed AppendExtraCommandLineSwitches | 种子注入结果 | INFO |
| persona-config GetOverride | 每次参数覆盖查询 | TRACE |
| metachat_ui.cc | WebUI 页面加载 | INFO |

#### 实现方式

```cpp
// 在 message_bus.cc 每个 handler 的入口和出口
METACHAT_VLOG(MESSAGE_BUS, 1) << "dispatch channel=" << channel << " action=" << action;
METACHAT_VLOG(ACCOUNTS, 1) << "HandleAccounts action=" << action;
METACHAT_VLOG(CDP, 2) << "HandleCdp action=" << action << " account=" << account_id;
```

日志级别通过 `config` 通道运行时控制（`config.set_log_level`）。
## 附录 A：branding-icons.patch — 品牌图标替换指南

### A.1 当前状态

`branding-icons.patch` 目前包含 ChromiumFish 的图标文件（Fish 图标），用于占位使编译通过。
后续 MetaChat 需要替换为自己的品牌图标。

### A.2 需替换的文件清单（共 43 个）

```
chrome/app/theme/chromium/
├── chromeos/
│   ├── chrome_app_icon_192.png
│   ├── chrome_app_icon_32.png
│   ├── crosh_app_icon_256.png
│   ├── webstore_app_icon_128.png
│   ├── webstore_app_icon_16.png
├── linux/
│   ├── product_logo_128.png
│   ├── product_logo_24.png
│   ├── product_logo_256.png
│   ├── product_logo_32.xpm
├── product_logo_16.png
├── product_logo_22.png
├── product_logo_24.png
├── product_logo_256.png
├── product_logo_32.png
├── product_logo_48.png
├── product_logo_64.png
├── ...（共 43 个文件，完整列表见 branding-icons.patch 中的 diff --git 行）
```

### A.3 替换步骤

```bash
# 1. 准备好所有替换的图标文件（保持文件名和路径一致）
# 2. 在 Chromium 源码目录中替换文件
cd F:/chromium_src/src
# 将新图标复制到对应路径，覆盖旧文件

# 3. 用 git diff 生成 binary patch（必须用 git diff --binary）
cd F:/chromium_src/src
git diff --binary -- chrome/app/theme/chromium/ > F:/metacaht/browser/patches/branding-icons.patch

# 4. 验证 patch 可用
cd F:/chromium_src/src
git checkout -- chrome/app/theme/chromium/
git apply F:/metacaht/browser/patches/branding-icons.patch

# 5. 提交到 metacaht 仓库
```

### A.4 重要注意事项

| 事项 | 说明 |
|------|------|
| **格式** | 必须使用 `git diff --binary` 生成 GIT binary patch 格式，不能用 `Binary files differ` 格式 |
| **禁止 PowerShell 写入 .patch** | PowerShell 默认 UTF-16 LE，会破坏二进制内容 |
| **平台覆盖** | Windows (.ico)、macOS (.icns)、Linux (.png/.xpm) |
| **图标尺寸** | 建议 16×16 ~ 256×256 全尺寸集 |
| **CI 测试** | 替换后需在 GHA 运行一次编译验证 |
