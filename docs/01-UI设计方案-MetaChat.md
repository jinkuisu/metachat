 # MetaChat · UI 设计方案
 
 ## 设计概念
 
 Canvas — 无固定 UI 的沉浸画布。没有侧栏、顶栏、底部固定导航。
 所有 UI 元素是浮在氛围画布上的玻璃片，用完即隐。
 浏览平台页面时 MetaChat 的壳几乎消失。
 
 ---
 
 ## 一、色调
 
 - Canvas 渐变：#FDFBF7 → #F5F2EC → #EDE9E2（165deg, 30s 循环）
 - 玻璃面板：rgba(255,255,255,0.85) + backdrop-filter blur(20px)
 - 主色：#0D9488 (teal-600)  |  hover：#14B8A6
 - 正文：#1C1917 (stone-900) | 辅助：#78716C (stone-500)
 - 阴影：0 4px 12px + 0 12px 40px rgba(0,0,0,0.06)（两层）
 - 字体：Plus Jakarta Sans (300-700)
 - 平台色：WA #25D366, TG #0088cc, FB #1877F2, IG #E4405F, X #1DA1F2
 
 ---
 
 ## 二、三屏状态
 
 A) Canvas 主页：暖渐变背景 + 问候玻璃卡 + 底部浮动平台药丸 + 🤖⚡🔤 工具岛
 B) 平台展开：hover 平台药丸后账号卡片漂浮出现，click → zoom-in 进入浏览器
 C) 浏览器沉浸：CloakBrowser 全屏嵌入平台页面。← 半透明返回按钮(左上) + 工具岛(右上)
   翻译浮球/快捷回复按钮通过 CDP 注入到平台页面内
 
 ---
 
 ## 三、工具面板
 
 浏览模式右上角 🤖⚡🔤 → click 滑入 380px 玻璃面板（AI/Reply/Translate tab）
 关闭：外部点击或 ESC
 
 ---
 
 ## 四、通知系统（两种模式可切换）
 
 模式1 — Ember Beads：左边缘 6px 光点，有未读时发光脉动。hover 展开预览，click 切换账号。
 模式2 — Status Bar：底部 32px 玻璃条，显示所有账号实时状态。
 
 ---
 
 ## 五、其他视图
 
 设置/回复/提示词：画布背景不变，居中弹出玻璃卡片（max-width:800px），ESC 关闭。
 
 ---
 
 ## 六、快捷操作
 
 - ⌘K：全局命令面板
 - Tab（浏览模式）：居中弹出账号概览，↑↓ Enter 切换
 - zoom-in/out：300ms cubic-bezier(0.16,1,0.3,1) 过渡
 
 ---
 
 ## 七、流程
 
 Canvas → hover 药丸 → 账号卡片 → click → zoom-in + 浏览器沉浸
 ← 返回 / ESC → zoom-out → Canvas
 
 浏览中：Ember Beads/Bar 管理多账号。🤖⚡🔤 工具。Tab 切换。⌘K 命令。
