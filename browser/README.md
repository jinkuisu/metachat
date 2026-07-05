# MetaChat Browser

MetaChat Browser 是基于 Chromium 150.0.7844.0 + ChromiumFish 21 个反指纹补丁的独立浏览器。

## 目录结构

├── patches/                # ChromiumFish 21 个反指纹补丁
├── assets/                 # ChromiumFish 资源覆盖（图标、字体）
├── metachat-patches/       # MetaChat 自定义补丁（品牌/UI/多会话）
├── apply.sh                # 补丁应用脚本
├── UPSTREAM_REVISION       # ChromiumFish 对应的 Chromium commit
└── .gclient                # gclient 配置
