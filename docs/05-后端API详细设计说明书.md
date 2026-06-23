# MetaChat 重构项目 - 后端API详细设计说明书

## 0. 设计规范

### 0.1 命名规范

| 层 | 规范 | 示例 |
|----|------|------|
| Java 类 | PascalCase | McUserController, SubscriptionService |
| Java 方法/变量 | camelCase | getUserById, totalAmount |
| Java 常量 | UPPER_SNAKE | MAX_SESSION_COUNT |
| Rust 类型 | PascalCase | Account, MessageContent, AppError |
| Rust 函数/变量 | snake_case | get_account, is_active |
| JSON 字段 | camelCase | userId, translatedText, createdAt |
| 数据库表/列 | snake_case | mc_user, created_at |
| API 路径 | kebab-case | /api/client/translate/batch |
| URL 参数 | camelCase | ?pageSize=20&sortBy=time |

### 0.2 API 字段标注格式

文档中 API 字段标注格式说明:

`text:string(1-5000)`    → 字段名:类型(长度范围)
`sourceLang?:string`     → 可选字段, 问号后缀
`items:[{...}]`          → 对象数组
`Err: 400(missing)`      → 可能的错误码(括号内为说明)

通用响应包装:
```json
{ "code": 200, "msg": "success", "data": {}, "timestamp": 1718000000 }
```
分页请求: { page:int, pageSize:int, ...filters }
分页响应: { code, msg, data: { total:long, page:long, pageSize:long, records:[] } }

### 0.3 消息通知机制

消息从社交平台到前端展示的完整链路:

```
社交平台 -> WebSocket/HTTP
         |
    CloakBrowser CDP 拦截帧
         |
    Rust protocol/ 模块解析 -> Message 结构体
         |
    ├── 写入本地 SQLite (messages 表)
    └── emit Tauri 事件: "new_message"
         |      payload: { platform, sessionId, senderName, preview }
         |
    Vue 前端 listen("new_message")
         |
    ├── 更新 Pinia sessionStore.unreadCount
    ├── 如当前在会话视图, 追加消息到消息列表
    └── 如不在会话视图, 显示桌面通知(可选)
```

AI 对话流式响应:

```
Vue -> invoke("translation_translate"|"ai_chat") -> Rust
  Rust -> reqwest SSE -> 第三方 API
  Rust -> emit("ai_token") 逐个 token
  Vue -> 累积显示
```

远程配置变更通知:

```
服务端修改配置/限额/规则
    -> MQTT publish: metachat/user/{userId}/config/updated
    -> 客户端 MQTT 收到
    -> 调用 GET /api/client/config 拉取最新配置
    -> 更新本地 limits/moderation 缓存
```

## 1. 项目结构

### 1.1 Maven 模块

metachat-parent (pom)
  +-- metachat-common    (公共DTO、枚举、工具类)
  +-- metachat-framework (安全、缓存、全局异常)
  +-- metachat-api       (客户端API: /api/client/*)
  +-- metachat-ops       (运维端API: /api/ops/*)
  +-- metachat-admin     (用户管理端API: /api/admin/*)
  +-- metachat-ai        (AI服务, 内部调用)
  +-- metachat-backup    (备份服务)
  +-- metachat-modules-* (业务模块: user, billing, fans, session等)

### 1.2 包结构 (以 metachat-api 为例)

com.metachat.api
  +-- config/        (WebMvc, SaToken, Undertow配置)
  +-- controller/    (REST控制器)
  +-- dto/           (请求DTO)
  +-- vo/            (响应VO)
  +-- service/       (服务层)
  +-- interceptor/   (拦截器)

### 1.3 技术栈

Spring Boot 3.4.x, Java 21, MyBatis-Plus, Sa-Token 1.39
PostgreSQL 15+, Redis 7.x
Undertow (替代Tomcat, 异步性能更好)

## 2. 认证与授权

### 2.1 统一登录流程

POST /api/public/login
  Input: { username, password }
  Output: { token, userType, role, redirectUrl, user }

处理逻辑:
1. 先查 sys_user 表 (运维/销售)
2. 若未找到, 再查 mc_user 表 (客户)
3. 校验密码 (BCrypt)
4. 生成 JWT, 含 userType 标记
5. 返回路由信息, 前端根据 userType + role 跳转

### 2.2 JWT 结构

{
  "userId": 12345,
  "userType": "sys_user",   // sys_user | mc_user
  "role": "ops",           // ops | sales | customer
  "deptId": 678,
  "permissions": ["user:view", "sys:user:add"],
  "exp": 1718000000
}

### 2.3 Sa-Token 双登录类型

配置两个 Sa-Token 登录类型:
- type="admin": sys_user 登录 (运维端+销售端)
- type="client": mc_user 登录 (客户端API)

注解使用:
@SaCheckLogin(type="client")   - 客户端API需登录
@SaCheckPermission(type="admin") - 运维+销售端需权限
@SaCheckPermission("user:view")  - 需 user:view 权限

### 2.4 权限代码前缀

sys:user/*, sys:role/*, sys:menu/*, sys:dept/*, sys:param/*
user:view/add/edit/del(客户用户), billing:* (计费)
moderation:* (审核), ai:* (AI管理), fans:* (粉丝)
payment:* (支付), statistic:* (统计), message:* (备份)

## 3. 客户端 API 端点

前缀: /api/client/*, 认证: @SaCheckLogin(type="client")

### 3.1 认证
POST /api/client/auth/login
  Auth: none
  Req: { username:string, password:string(6-128) }
  Res: { token:string, user:{ id:long, username:string, phone?:string, email?:string }, limits:{ maxSessions:int } }
  Err: 401(bad credentials), 1001(user disabled)

POST /api/client/auth/register
  Auth: none
  Req: { username:string(4-32), password:string(6-128), email?:string, phone?:string, inviteCode?:string }
  Res: { token:string, user:{ id:long, username:string } }
  Err: 409(username exists), 400(invalid invite code)
  Note: 客户自助注册, 也可由销售在Admin端创建

POST /api/client/auth/logout
  Auth: client
  Res: {}

### 3.2 配置
GET /api/client/config
  Auth: client
  Res: {
    features: { ai:bool, mobileSync:bool, voiceDictation:bool },
    limits: { maxSessions:int, maxContactsSync:int, maxTranslationChars:long },
    updateChannel:string,
    dict: { platformTypes:[], translationEngines:[], planTypes:[], syncPolicies:[] }
  }
  Note: config 变更通过 MQTT 通知客户端
{
  "features": { "ai": true, "mobile_sync": false },
  "limits": {
    "max_sessions": 5,           // 最大并发会话数（账号数）
    "max_contacts_sync": 1000,   // 联系人同步上限
    "max_translation_chars": 0   // 月翻译字符限制, 0=不限制
  },
  "update_channel": "stable",
  "dict": { ... }                // 数据字典
}

### 3.3 会话 (本地存储 + 服务端限额)
会话和消息存储在本地 SQLite，通过 Tauri IPC 命令访问。
不经过 HTTP API 获取会话内容。

会话数量（并发账号数）由服务端通过 /api/client/config 中的
limits.max_sessions 控制。客户端在添加新账号时检查此限额。
超过限额时提示用户升级套餐。

服务端仅接收联系人摘要同步（用于重粉检测）。

### 3.4 联系人 (本地存储)
联系人和标签存储在本地 SQLite，通过 Tauri IPC 命令访问。
联系人摘要通过 /api/client/sync/contacts 同步到服务端。

### 3.5 翻译 (服务端中转)
POST /api/client/translate
  Auth: client
  Req: { text:string(1-5000), sourceLang?:string, targetLang:string, engine?:string }
  Res: { translatedText:string, detectedLang?:string, engine:string, durationMs:long }
  Err: 400(missing field), 3001(engine unavailable)
  Note: API key 由服务端管理

POST /api/client/translate/batch
  Auth: client
  Req: { items:[{ text:string, sourceLang?:string, targetLang:string, engine?:string }] }
  Res: [{ translatedText:string, detectedLang?:string, engine:string, durationMs:long }]

GET /api/client/translate/engines
  Auth: client
  Res: [{ engine:string, name:string, available:bool }]
  Note: 引擎列表来自 sys_dict(dict_type=translation_engine)

服务端支持的翻译通道:
- Google Translate API (服务端 key)
- DeepL API (服务端 key)
- OpenAI / DeepSeek (服务端 key)
- 自建 Nginx 代理 tg.yituoke.org → Google Translate (无需 key, 客户端可直连)

### 3.6 AI (服务端中转)
POST /api/client/ai/chat
  Auth: client
  Req: {
    messages:[{ role:string(user|assistant|system), content:string }],
    roleId?:long, promptId?:long, stream:bool(true)
  }
  Res (SSE): data:{ type:"token", content:string }
             data:{ type:"done", usage:{ promptTokens:int, completionTokens:int } }
             data:{ type:"error", message:string }
  Err: 2001(insufficient balance), 2002(ai unavailable)
  Note: 上下文通过 messages 传入, 计费同步完成

GET /api/client/ai/roles/public
  Auth: client
  Res: [{ id:long, name:string, systemPrompt:string, tone:string, isDefault:bool }]

GET /api/client/ai/prompts/public
  Auth: client
  Res: [{ id:long, category:string, name:string, prompt:string, isDefault:bool }]

GET /api/client/ai/conversation/list
  Auth: client
  Res: [{ id:long, title:string, messageCount:int, createdAt:timestamp }]

GET /api/client/ai/conversation/{id}
  Auth: client
  Res: { id:long, messages:[{ role:string, content:string, timestamp:long }] }

AI 对话流程:
1. 客户端从本地 SQLite 加载最近消息作为上下文
2. POST /api/client/ai/chat 传入 messages
3. 服务端转发大模型 → SSE 流式返回
4. 服务端同步计费(Token 计数 + 扣减配额)
5. 客户端将完整对话存入本地 SQLite
6. 客户端可选择同步到服务端(跨设备)

服务端支持的 AI 通道:
- OpenAI GPT-4 / GPT-4o (服务端 key)
- DeepSeek (服务端 key)
- Google Gemini (服务端 key)
- Anthropic Claude (服务端 key)

### 3.7 自动回复
GET /api/client/replies                     - 回复规则
POST /api/client/replies                    - 新建规则
GET /api/client/replies/groups              - 回复分组

### 3.8 计费
POST /api/client/subscription/purchase
  Auth: client
  Req: { planId:long, quantity:int, cycleDays?:int, cancelExisting?:bool }
  Res: { subscriptionId:long, startTime:timestamp, endTime:timestamp, amount:decimal }
  Err: 2001(insufficient balance), 2002(plan unavailable)

GET /api/client/subscription/current
  Auth: client
  Res: [{ id:long, planName:string, planType:int, quantity:int, startTime:timestamp, endTime:timestamp, status:int, autoRenew:bool }]

POST /api/client/subscription/{id}/cancel
  Auth: client
  Res: { refundAmount:decimal }
  Err: 409(already cancelled)

### 3.9 支付
POST /api/client/payment/create
  Auth: client
  Req: { orderType:string, planId?:long, quantity?:int, amount:decimal, channel:string }
  Res: { orderId:string, payUrl?:string, qrCode?:string, expireAt:timestamp }
  Err: 2001(insufficient), 4002(channel unavailable)
  Note: 创建支付订单, 支付成功后自动激活订阅

GET /api/client/payment/channels
  Auth: client
  Res: [{ channel:string, name:string, enabled:bool }]

### 3.10 同步
POST /api/client/sync/contacts
  Auth: client
  Req: { contacts:[{ platformContactId:string, platform:string, addedAt:timestamp }] }
  Res: { synced:int, duplicates:int }
  Note: 自动调用, 用于重粉检测

POST /api/client/sync/upload
  Auth: client
  Req: multipart: { data:file(json), encrypt?:bool }
  Res: { version:long, itemsCount:int }
  Note: 手动触发, 全量数据

GET /api/client/sync/download?version={version}
  Auth: client
  Res: { version:long, data:{ contacts:[], labels:[], settings:{} } }
  Note: 另一端拉取

### 3.11 文件
POST /api/client/files/upload
  Auth: client
  Req: multipart: { file:binary }
  Res: { fileId:string, url:string, size:long, mime:string }
  Note: 最大 50MB

GET /api/client/files/download/{id}
  Auth: client
  Res: binary (Content-Type: application/octet-stream)

### 3.12 状态
GET /api/client/status
  Auth: none (或client)
  Res: { online:bool, version:string, accountsOnline:int, uptime:long }
  Note: 健康检查, 用于检测客户端是否在线

### 3.13 统计
GET /api/client/stats/daily
  Auth: client
  Res: { date:string, messagesSent:int, messagesReceived:int, translations:int, translationChars:long, aiConversations:int, friendsAdded:int }

GET /api/client/stats/trend?days={int}
  Auth: client
  Res: [{ date:string, messagesSent:int, messagesReceived:int }]

GET /api/client/stats/by-platform
  Auth: client
  Res: [{ platform:string, messagesSent:int, messagesReceived:int, friendsCount:int }]

GET /api/client/stats/usage
  Auth: client
  Res: { totalTranslations:long, totalTranslationChars:long, totalAiConversations:long, periodStart:string, periodEnd:string }

### 3.14 内容审核 (拉取规则)
GET /api/client/moderation/rules
  Auth: client
  Res: { version:long, sensitiveWords:[{ word:string, category:string, action:string(block|warn) }], globalRules:[], customRules:[] }
  Note: 客户端拉取后在本机执行过滤, 不经过 HTTP 检查消息

### 3.15 重粉检测
GET /api/client/repeat-fans
  Auth: client
  Res: [{ id:long, contactA:{ platform, name }, contactB:{ platform, name }, matchScore:float, detectedAt:timestamp, handled:bool }]

PUT /api/client/repeat-fans/{id}/handle
  Auth: client
  Req: { action:string(ignore|merge) }
  Res: {}

## 4. 运维端 API 端点

前缀: /api/ops/*, 认证: @SaCheckPermission(type="admin")

### 4.1 系统管理
POST /api/ops/system/users/page             - 用户列表
POST /api/ops/system/users                  - 新建用户
PUT /api/ops/system/users/{id}              - 修改用户
DELETE /api/ops/system/users/{id}           - 删除用户
POST /api/ops/system/roles/page             - 角色列表
POST /api/ops/system/roles                  - 角色CRUD
GET /api/ops/system/menus/tree              - 菜单树
GET/POST /api/ops/system/params             - 系统参数

### 4.2 资金池
POST /api/ops/fund-pool/recharge            - 充值调账
POST /api/ops/fund-pool/freeze              - 冻结
POST /api/ops/fund-pool/unfreeze            - 解冻

### 4.3 代下单
POST /api/ops/subscription/purchase         - 代客户订阅
POST /api/ops/subscription/{id}/cancel       - 退订退款

### 4.4 财务报表
GET /api/ops/reports/income                 - 收入汇总
POST /api/ops/reports/dept-billing          - 部门账单
POST /api/ops/reports/trend                 - 趋势

### 4.5 支付
POST /api/ops/payment/channels              - 渠道配置
POST /api/ops/payment/orders/page           - 订单列表

### 4.6 AI管理
POST/DELETE /api/ops/ai/roles               - AI角色
POST/DELETE /api/ops/ai/prompts             - 提示词
POST /api/ops/ai/bug-tickets                - Bug工单

### 4.7 审核
POST/PUT/DELETE /api/ops/moderation/words   - 敏感词
POST/PUT/DELETE /api/ops/moderation/rules   - 规则

### 4.8 客户管理
PUT /api/ops/client-users/{id}/lock         - 锁定
PUT /api/ops/client-users/{id}/unlock       - 解锁
PUT /api/ops/client-users/{id}/reset-pwd    - 改密

### 4.9 监控
GET /api/ops/monitor/stats                  - 平台统计
POST /api/ops/monitor/logs                  - 操作日志

## 5. 用户管理端 API 端点

前缀: /api/admin/*, 认证: @SaCheckLogin (自动识别用户类型)

### 5.1 客户管理 (销售视角)
POST /api/admin/client-users/page           - 客户列表
POST /api/admin/client-users                - 新建客户
PUT/DELETE /api/admin/client-users/{id}     - 修改/删除
PUT /api/admin/client-users/{id}/lock       - 锁定
PUT /api/admin/client-users/{id}/reset-pwd  - 改密

### 5.2 部门
GET /api/admin/depts/tree                   - 部门树
POST /api/admin/depts                       - 新建部门
PUT/DELETE /api/admin/depts/{id}            - 修改/删除
GET/PUT /api/admin/depts/{id}/pricing       - 定价

### 5.3 套餐
POST /api/admin/plans                       - 自定套餐
PUT/DELETE /api/admin/plans/{id}            - 修改/删除

### 5.4 粉丝
POST /api/admin/fans/page                   - 粉丝列表
POST/PUT /api/admin/fans/tasks              - 接粉任务
POST/PUT/DELETE /api/admin/fans/links       - 活链

### 5.5 审核(客户级)
POST/PUT/DELETE /api/admin/moderation/words - 敏感词
POST /api/admin/moderation/violations       - 违规记录

### 5.6 备份
POST /api/admin/backups/page                - 备份列表
GET /api/admin/backups/search               - 消息检索

### 5.7 统计
POST /api/admin/stats/team                  - 团队统计
POST /api/admin/stats/users                 - 人员统计

### 5.8 客户自助
GET/PUT /api/admin/profile                  - 个人资料
POST/DELETE /api/admin/team-members         - 子账号
POST /api/admin/subscription/purchase       - 购买
GET /api/admin/subscription/current         - 当前订阅
PUT /api/admin/preferences/translation      - 翻译偏好
PUT /api/admin/preferences/ai-binding       - AI绑定
POST /api/admin/backup/export               - 导出备份
GET /api/admin/stats/self                   - 个人统计

## 6. 公共 API 端点

前缀: /api/public/*, 认证: 无

POST /api/public/login                      - 统一登录
GET /api/public/captcha                     - 验证码
GET /api/public/config                      - 客户端配置
GET /api/public/key                         - RSA公钥

## 7. 数据库设计

### 7.1 系统表

sys_user:
  id BIGSERIAL PK, username VARCHAR(64) UNIQUE, password VARCHAR(256)
  real_name VARCHAR(64), dept_id BIGINT, status SMALLINT
  user_type VARCHAR(16) DEFAULT "ops"  (ops|sales)

sys_role: id, name, code UNIQUE, status
sys_menu: id, parent_id, name, permission, path, type
sys_role_menu: role_id, menu_id (复合PK)
sys_user_role: user_id, role_id (复合PK)
sys_dept: id, parent_id, name
sys_param: 系统参数表
  id BIGSERIAL PK, param_name VARCHAR(128) NOT NULL UNIQUE
  param_value TEXT NOT NULL
  param_type VARCHAR(32) DEFAULT 'string'   (string|json|number|boolean)
  description VARCHAR(512)
  updated_at TIMESTAMP

sys_dict: 数据字典表
  id BIGSERIAL PK
  dict_type VARCHAR(64) NOT NULL      -- 字典类型编码, 如 platform_type, translation_engine
  dict_code VARCHAR(64) NOT NULL      -- 字典项编码, 如 whatsapp, telegram, google
  dict_label VARCHAR(256) NOT NULL    -- 显示名
  dict_label_en VARCHAR(256)          -- 英文显示名
  dict_value VARCHAR(512)             -- 附加值 (如引擎API地址)
  sort_order INTEGER DEFAULT 0
  status SMALLINT DEFAULT 1
  remark VARCHAR(256)
  created_at TIMESTAMP
  UNIQUE(dict_type, dict_code)

sys_i18n: 国际化翻译表
  id BIGSERIAL PK
  locale VARCHAR(16) NOT NULL         -- zh-CN, en, ja, ko
  module VARCHAR(64) NOT NULL         -- common, platform, ai, moderation
  key_name VARCHAR(256) NOT NULL      -- 翻译key, 如 sessionToolbar.translation.google
  translation TEXT NOT NULL
  updated_at TIMESTAMP
  UNIQUE(locale, module, key_name)

### 7.2 客户表

mc_user:
  id BIGSERIAL PK, username VARCHAR(64) UNIQUE, password VARCHAR(256)
  status SMALLINT DEFAULT 1, dept_id BIGINT

mc_friend (联系人摘要, 重粉检测):
  id BIGSERIAL PK, user_id BIGINT NOT NULL
  platform VARCHAR(32) NOT NULL
  platform_contact_id VARCHAR(128) NOT NULL
  contact_name VARCHAR(256), added_at TIMESTAMP NOT NULL
  UNIQUE(user_id, platform, platform_contact_id)
  索引: idx_mc_friend_user(user_id), idx_mc_friend_platform(platform,platform_contact_id)

mc_account: id, dept_id UNIQUE, balance DECIMAL(10,2), frozen_amount DECIMAL(10,2)

### 7.3 计费表

mc_plan: id, name, plan_type SMALLINT(1/2/3), unit_price DECIMAL(10,2), dept_id, is_active
mc_subscription: id, user_id, plan_id, quantity INT, start_time TIMESTAMP, end_time TIMESTAMP, status SMALLINT, amount DECIMAL(10,2)
mc_recharge_record: id, account_id, amount, balance_before, balance_after, operator_id

### 7.4 业务表

mc_fans, mc_fans_link, mc_fans_task, mc_fans_session
mc_reply_group, mc_reply, mc_reply_history
mc_sensitive_word, mc_moderation_rule, mc_violation_report
ai_role, ai_prompt, ai_role_draft, ai_bug_ticket, ai_contact_role_binding

## 8. 通用响应格式

{
  "code": 200,      // 200=成功, 非200=错误
  "msg": "success",
  "data": {...},    // 业务数据
  "timestamp": 1718000000
}

分页请求: { page: int, pageSize: int }
分页响应: { total: long, page: long, pageSize: long, records: T[] }

## 9. 错误码

200=成功
400=参数错误, 401=未登录, 403=无权限
404=不存在, 409=状态冲突, 500=服务器错误
1001=用户已禁用, 1002=设备超限
2001=余额不足, 2002=套餐不可用, 2003=退款失败
3001=翻译不可用
4001=CloakBrowser启动失败, 4002=CDP连接超时

## 10. 安全

- 登录密码: BCrypt加密
- 通信: HTTPS 全站
- 敏感接口: RSA加密请求体
- API加密: CryptoRequestFilter (AES-GCM)
- 防重放: RouteRateLimitInterceptor
- XSS防护: InputSecurityInterceptor

## 11. 集成

- 文件存储: 本地文件系统 + Nginx 静态资源
- 支付回调: POST /payment/callback/alipay, /payment/callback/wechat (公开)
- MQTT: 桌面端 + 服务端 均连接 EMQX
- 桌面端: 接收消息通知 + 配置变更推送
- 服务端: 发布配置变更通知
- 缓存: Redis, key前缀 mc:sys:param, mc:ai:prompt

## 12. 内部 API

前缀: /api/internal/*, 认证: IP白名单 + Token

GET /api/internal/quota/get                   - 获取配额
GET /api/internal/quota/char/ensure           - 确保字符配额

## 13. 外键策略

所有表不使用数据库级外键约束。关联关系在应用层通过代码保证。
仅建立普通 B-tree 索引以加速 JOIN 查询。
索引命名规范: idx_表名_字段名


### 7.5 数据字典示例

sys_dict:

| dict_type | dict_code | dict_label | sort_order |
|-----------|-----------|------------|------------|
| platform_type | whatsapp | WhatsApp | 1 |
| platform_type | telegram | Telegram | 2 |
| platform_type | line | LINE | 3 |
| translation_engine | google | Google Translate | 1 |
| translation_engine | deepL | DeepL | 2 |
| translation_engine | openAi | OpenAI | 3 |
| sync_policy | auto | 自动同步 | 1 |
| sync_policy | manual | 手动同步 | 2 |
| plan_type | port | 端口套餐(按并发账号数) | 1 |
| plan_type | day | 天数套餐(按使用天数) | 2 |
| plan_type | chars | 字符套餐(按翻译字符量) | 3 |
| moderation_action | block | 拦截消息 | 1 |
| moderation_action | warn | 告警不拦截 | 2 |

sys_param:

| param_name | param_value | param_type | 说明 |
|------------|-------------|------------|------|
| app.name | MetaChat | string | 应用名称 |
| app.version | 4.0.0 | string | 版本号 |
| sync.contacts.interval | 3600 | number | 联系人自动同步间隔(秒) |
| dict.update.channels | {"items":[]} | json | 更新通道配置 |
| dict.platform.services | {"items":[]} | json | 平台服务列表 |
| feature.ai.enabled | true | boolean | AI功能开关 |
| feature.mobile.sync | false | boolean | 手机端同步开关 |

### 7.6 国际化数据管理

运维端提供 sys_i18n 管理界面，支持增删改查和批量导入/导出。
客户端启动时拉取当前 locale 的翻译数据。
默认加载 locale = zh-CN 和 en 的完整数据，其他语言按需加载。

### 4.10 数据字典管理
POST/PUT/DELETE /api/ops/system/dicts           - 字典CRUD
GET /api/ops/system/dicts/{type}                - 按类型查询

### 4.11 国际化管理
POST/PUT/DELETE /api/ops/system/i18n            - 翻译CRUD
POST /api/ops/system/i18n/import                - 批量导入
GET /api/ops/system/i18n/export                 - 批量导出

## 14. MQTT 配置变更通知

当服务端修改了客户配置、套餐限额、审核规则等，通过 MQTT 推送通知给客户端。
客户端收到通知后调用 GET /api/client/config 拉取最新配置。

MQTT 主题:
metachat/user/{userId}/config/updated    - 配置已变更(建议重新拉取)

消息体: { "type": "config", "version": 123, "updated_at": "2026-06-12T10:00:00Z" }

客户端收到通知后的行为:
1. 调用 GET /api/client/config 拉取最新配置
2. 更新 limits.max_sessions 等限额
3. 更新 moderation 规则缓存
4. 如 features 有变化, 更新本地功能开关

适用场景:
- 管理员修改了客户的套餐(limits 变化) → 推送通知
- 管理员修改了客户的审核规则 → 推送通知
- 管理员启用了 AI/手机同步等功能 → 推送通知
- 客户自助购买订阅 → 服务端触发推送