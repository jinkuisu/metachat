# -*- coding: utf-8 -*-
import sys
f=open("docs/metachat-browser-设计文档.md","rb")
raw=f.read()
f.close()
text=raw.decode("utf-8-sig").replace("\r\n","\n")
print("len:", len(text))
old="| 参数 | 类型 | 默认值 | 说明 |\n|------|------|--------|------|\n| --metachat-mode | 开关 | 无 | 启用 MetaChat 模式（隐藏原生 UI） |"
new_="| 参数 | 类型 | 默认值 | 说明 |\n|------|------|--------|------|\n| --metachat-mode | 开关 | 无 | 启用 MetaChat 模式（隐藏原生 UI） |\n| --metachat-mode-allowlist | 字符串 | 空 | 逗号分隔的 IDC 命令 ID，即使在默认拦截列表中也放行 |\n| --metachat-mode-blocklist | 字符串 | 空 | 逗号分隔的 IDC 命令 ID，额外拦截 |\n| --metachat-accounts-config | 路径 | accounts.json | 账号配置文件路径 |\n\n启动示例：\n```bash\nMetaChat.exe --metachat-mode --metachat-accounts-config=C:/Users/xxx/metachat-accounts.json\n```"
if old in text: text=text.replace(old, new_); print("step3 ok")
else: print("step3 FAIL"); sys.exit(1)