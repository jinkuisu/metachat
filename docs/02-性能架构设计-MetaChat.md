# MetaChat · CloakBrowser 性能优化架构

## 问题

每个 CloakBrowser 实例 = 一个 Chromium 子进程。WhatsApp Web 等 SPA 单实例占用 200-500MB。
20 个账号 = 4-10GB RAM，不优化不可用。

## 核心策略：四级生命周期 + LRU 自动降级

### 四级状态

| 级别 | 名称 | 进程 | 内存 | CDP |
|------|------|------|------|-----|
| L0 | OFFLINE | 无 | 0 | 无 |
| L1 | MONITOR | headless Chromium | ~30MB | Network + Runtime 仅用于监听消息 |
| L2 | ACTIVE | 全渲染 Chromium | 200-500MB | 全部启用，用户可见可交互 |
| L3 | FROZEN | 已 kill | ~5KB (SQLite Cookie) | 无，恢复时重新启动 |

### 状态转换

OFFLINE → 用户启动 → MONITOR (headless CDP 监听)
MONITOR → 用户点击查看 → ACTIVE (全渲染)
ACTIVE → 5分钟无交互 → MONITOR (降级，关闭渲染保留 CDP)
MONITOR → 30分钟无交互 → FROZEN (杀进程，保存 Cookie)
FROZEN → 用户点击 → MONITOR (重新启动，恢复 Cookie)

### LRU 调度

每 10 秒检查一次：
1. 收集所有实例内存占用
2. 总内存超过预算 75% → 从 LRU 最旧开始降级
3. ACTIVE 超过 max_active 上限 → 降级为 MONITOR
4. MONITOR 超过 max_monitor 上限 → 冻结最旧

## 核心优化：CDP-Only 监控

MONITOR 模式节省 85%+ 内存的关键：

- 启动 CloakBrowser 加 --headless=new --disable-gpu --disable-software-rasterizer
- CDP 只启用 Network.enable() 和 Runtime.enable()
- 不启用 Page/DOM/CSS (不需要渲染事件)
- 通过 Fetch.enable() 拦截图片/视频请求 → 返回 204 不加载
- WebSocket 通道完整保留 → protocol 解析器处理消息

## 资源拦截

MONITOR 模式下通过 CDP Fetch 域拦截：

- 图片 → 阻止 (节省带宽和内存)
- 视频 → 阻止
- CSS/JS/字体 → 放行 (页面正常运行需要)
- WebSocket → 放行 (消息通道)
- 平台 API → 放行 (消息数据)

## Cookie 持久化

FROZEN → 恢复流程：
1. SQLite 读取加密 Cookie
2. 启动新 CloakBrowser 实例
3. CDP Network.setCookies() 恢复
4. 导航到平台 URL
5. 等待初始化完成
6. 恢复注入脚本

## 动态 max_active

基于系统内存自动适配：

- 4GB → 2-3 个
- 8GB → 4-5 个
- 16GB → 7-8 个
- 32GB+ → 最多 8 个

计算公式：可用 = 总内存 * 0.75 - 1.5GB(系统预留)，每个 ACTIVE 按 400MB 估算。

## UI 配合

后端通过 Tauri event 推送给前端：

- account_status_changed { uId, status }
- account_memory_usage { uId, memoryMb }
- pool_stats { active, monitor, frozen, totalMemoryMb }

Ember Beads 光点颜色：
- ACTIVE → 常亮
- MONITOR → 半透明呼吸
- FROZEN → 灰色圆点
- OFFLINE → 不显示

## 旧项目对比

旧 Electron 项目：computeMaxHot() 只估算不干预，evaluateThrottling() 为空函数。
新架构：四级状态机 + 自动调度 + CDP-Only 模式 + 资源拦截 + Cookie 持久化。

## 实现优先级

Phase 1 (与 UI 同步开发)：
- PoolConfig + InstancePool 数据结构
- OFFLINE/ACTIVE/FROZEN 三级
- LRU 追踪 + 手动上限控制
- 内存监控 + Tauri event 上报

Phase 2：
- MONITOR 模式 (headless + CDP-Only)
- CDP 资源拦截
- 自动降级

Phase 3：
- 动态 max_active
- Cookie 持久化 + 恢复
- Ember Beads 状态联动
- 用户配置面板
WARM 层补充说明：

新增 WARM 层（介于 ACTIVE 和 MONITOR 之间）

WARM 的特点：
- 进程处于全渲染状态（与非 headless 无异）
- 但窗口隐藏 / 不显示在屏幕上（background: transparent, 无 WebView attach）
- 用户瞬间可见（< 100ms）：只需 WebView attach + 窗口 show
- 最多保留 2 个 WARM 实例（最近切换过的账号）
- 淘汰策略：新 ACTIVE 产生时，旧 ACTIVE → WARM；WARM 超额 → 最早 WARM 降为 MONITOR

优先级分数（替代纯 LRU）

每个账号在 MONITOR/WARM 中按分数排序：

score = unread_count * 3 + recency_weight

其中 recency_weight：
- 最近 1 分钟内有过消息 → 50 分
- 最近 5 分钟内有过消息 → 20 分
- 最近 30 分钟内有过消息 → 5 分
- 超过 30 分钟 → 0 分

调度规则：
- score >= 30 → 自动从 MONITOR 升为 WARM（预渲染等你点）
- 内存不足时先降级 score 最低的，而非纯 LRU
- 用户通过 Ember Beads 点击 MONITOR 账号时：显示 loading + 进度，预计 2-3s
- 点击 WARM 账号时：瞬间切换

Ember Beads 视觉对应：
- ACTIVE → 常亮，全尺寸
- WARM → 半亮，细微呼吸
- MONITOR + score >= 30 → 微光，脉冲较慢
- MONITOR + score < 30 → 暗点，无动画
- FROZEN → 灰色
- OFFLINE → 不显示
## 自适应优先级算法（v2）

### 核心原则

资源充足时不做计算，资源紧张时按场景自适应。

### Phase 1：容量判断

```
如果 total_active + total_warm <= max_active_capacity：
    资源充足，不降级任何账号
    停止计算

如果 total_active + total_warm > max_active_capacity：
    需要降级，进入 Phase 2
```

max_active_capacity 是动态值，基于系统内存：
- 4GB → 3
- 8GB → 5
- 16GB → 8
- 32GB+ → 10

### Phase 2：场景判断

计算两个指标：

```
消息密度 = total_unread / max_active_capacity

压力系数 = (total_active + total_warm) / max_active_capacity
```

根据这两个指标选择评分模式：

```
IF 压力系数 > 0.8 (严重超载):
  # 紧急模式：只看未读数，谁消息多保谁
  score = unread_ratio × 1.0 + 0

ELIF 消息密度 < 2 (消息很少):
  # 安静模式：未读数不重要，按 LRU 降级
  score = 0 + recency_ratio × 1.0

ELIF 消息密度 >= 2 && 压力系数 <= 0.8:
  # 正常模式：未读 × 3 + 时间权重
  score = unread_ratio × 0.6 + recency_ratio × 0.4

ELIF 消息密度 >= 5 (消息爆炸):
  # 繁忙模式：未读数主导
  score = unread_ratio × 0.8 + recency_ratio × 0.2
```

### Phase 3：Pin 优先

无论什么模式，用户手动 Pinned 的账号：

```
if account.is_pinned:
    score += 0.3
    # 且永不 FROZEN（最多降到 MONITOR）
```

### 示例场景

场景 A：8GB 内存，3 个账号，总共 5 条未读
```
max_active_capacity = 5
total_active + warm = 3 <= 5 → Phase 1 退出
结果：全部保持 WARM/ACTIVE，不降级
```

场景 B：8GB 内存，8 个账号，总共 120 条未读
```
max_active_capacity = 5
total_active + warm = 8 > 5 → 进入 Phase 2
消息密度 = 120 / 5 = 24 >= 5 → 繁忙模式
score = unread_ratio × 0.8 + recency_ratio × 0.2
结果：未读数最高的 5 个账号保持，3 个降级
```

场景 C：16GB 内存，12 个账号，总共 10 条未读
```
max_active_capacity = 8
total_active + warm = 12 > 8 → 进入 Phase 2
消息密度 = 10 / 8 = 1.25 < 2 → 安静模式
score = recency_ratio × 1.0
结果：最近活跃的 8 个保持，4 个最久未用的降级
```

### 降级阈值（谁被降级）

按 score 从低到高排序。得分最低的 N 个降级：

```
需要降级的数量 = (total_active + total_warm) - max_active_capacity
```

### WARM 到 MONITOR

WARM 降级后进入 MONITOR（headless CDP 继续监听消息）。
MONITOR 中如果收到新消息 → score 重新计算 → 可能自动提升回 WARM。

### Freeze 策略

MONITOR 的账号再满足以下任一条件才 FREEZE：
- 超过 30 分钟无任何消息活动
- 系统内存使用超过 90% 预算
- 用户手动强制关闭
