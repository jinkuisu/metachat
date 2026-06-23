这份方案是专为提交给 AI 编程工具（如 Cursor、Windsurf）或技术合伙人设计的全套完整架构方案。方案融合了 Python (PySide6) + 物理级独立内核 (chromiumfish / CloakBrowser) + CDP 协议，在保障 C++ 级别绝对防关联安全 的同时，完美实现了 15+ 窗口极低内存运行 与 零成本白嫖 Chrome 翻译插件 的核心商业诉求。

🚀 项目技术方案：基于 Qt 外壳与独立内核的硬核防关联多开聚合系统
一、 系统核心架构图 (Architecture Overview)
系统采用“单一原生主控进程 + 跨进程 CDP 调度 + 操作系统级窗口吸附”的分布式架构。

二、 核心技术模块设计与实现路径
模块 1：系统外壳与多账号容器管理 (Qt/PySide6)
UI 设计：使用 PySide6 (QML 或 QWidget) 编写现代化 UI。左侧为账号列表（支持头像、状态红点、代理 IP 显示），右侧为 QStackedLayout（层叠布局管理器）作为嵌入网页的容器。

物理节流机制（解决卡死关键）：当用户在左侧切换账号时，不活跃的容器自动触发 container->hide()。Chromium 底层在认定窗口完全隐藏（不可见）时，会自动暂停网页渲染帧率，挂起后台 V8 定时器并释放多达 80% 的 VRAM（显存）。这确保了 15 个以上的账号同时挂机时，CPU 占用接近 0%，内存开销极低。

模块 2：物理级多进程绝对隔离 (Process Isolation)
当用户点击启动某个账号时，Qt 后端通过 QProcess 异步拉起独立的浏览器内核二进制文件（以 chromiumfish.exe 为例），并强制注入绝对隔离的命令行参数：

Python
# 伪代码：为每个账号动态分流启动
import os
from PySide6.QtCore import QProcess

def launch_profile(account_id, proxy_url, seed_string, cdp_port):
    process = QProcess()
    # 核心隔离参数
    args = [
        f"--app=https://web.whatsapp.com",                    # 应用模式，剥离地址栏
        f"--user-data-dir=C:/App/Data/{account_id}",          # 1. 独立沙盒目录，彻底隔离Cookie/缓存
        f"--proxy-server={proxy_url}",                        # 2. 独立代理IP，网络物理隔离
        f"--persona-seed={seed_string}",                      # 3. 独立指纹种子，C++底层重算硬件指纹
        f"--remote-debugging-port={cdp_port}",                # 4. 分配专属CDP端口，用于外部白嫖翻译
        f"--load-extension=C:/App/Plugins/free_translator"   # 5. 静默加载本地免登录翻译插件
    ]
    process.start("chromiumfish.exe", args)
    return process.processId()
模块 3：跨进程窗口“捕获与生吞” (Window Embedding)
由于拉起外部 .exe 是异步的，主程序需要通过操作系统句柄（HWND）将外部浏览器窗口强行“吸附”进 Qt 内部。

⚠️ 跨平台警报： 该吸附方案在 Windows 上完美支持。如果未来考虑 Mac (macOS) 市场，苹果的沙盒机制严禁跨进程窗口绑架，Mac 端需自动降级为“Qt 控制台 + 浏览器独立多弹窗”架构（AdsPower 同款商业妥协路线）。

给 AI 的核心 Windows 捕获算法：

获取 QProcess 的 PID。

启动一个定时器，每隔 50 毫秒通过 Win32 API 遍历系统窗口，过滤出 WindowProcessID == PID 且窗口类名为 Chrome_WidgetWin_1 的顶级句柄（HWND）。

拿到句柄后，执行 Qt 原生吞窗：

Python
from PySide6.QtGui import QWindow
from PySide6.QtWidgets import QWidget

# 将操作系统的 HWND 转换为 Qt 识别的 QWindow，并用容器组件包裹
foreign_window = QWindow.fromWinId(hwnd)
container_widget = QWidget.createWindowContainer(foreign_window)

# 塞进右侧的层叠布局中，完美合体
stacked_layout.addWidget(container_widget)
三、 终极白嫖功能：基于 CDP 协议的插件内部通信
为了零成本、免 Key 实现 WhatsApp 的聊天消息翻译与抓取，系统不从网页内部注入容易被风控检测的 JS，而是直接站在操作系统的上帝视角，通过 CDP 协议强行给加载的免费插件发“网络电报”。

白嫖自动化实现逻辑：
端口维护字典：Qt 后端必须在内存中维护一个端口字典：{ "acc_001": 9201, "acc_002": 9202 }，确保 15 个号的调试流量绝不串流。

定位插件后台：Qt 请求 http://127.0.0.1:{cdp_port}/json，解析返回的列表，通过 type: "background_page" 定位到免登录翻译插件的幕后 WebSocket 调试地址（webSocketDebuggerUrl）。

数据强行灌入：利用 Python 异步套接字直接连接该地址，使用 CDP 的 Runtime.evaluate 方法，伪装成前台页面向插件后台发送 chrome.runtime.sendMessage 通信暗号（暗号数据结构需先让 AI 分析插件源码解密），从而白嫖翻译结果。

四、 工业级踩坑防范预案 (必须实现的防线)
DPI 缩放混乱：外部 Chromium 被强嵌后，若用户开启了 150% 屏幕缩放，网页会变模糊。

解法：必须在 Python 主程序入口最顶部加上：QApplication.setAttribute(Qt.AA_EnableHighDpiScaling)。

孤儿进程与内存泄露：若 Qt 主程序突然闪退，后台 15 个独立浏览器会继续常驻，导致系统卡死。

解法：必须引入 Windows Job Object (作业对象) API。将主进程与拉起的所有浏览器子进程绑定到同一个 Job 中。一旦主程序异常退出，操作系统会瞬间强行掐死整个 Job 链条下的所有浏览器，绝不留痕。

五、 直接喂给 AI 工具 (Cursor / Claude) 的终极提示词 (Prompt)
你可以直接把下面这段话复制给 AI 编程工具，它就能帮你直接把核心跑起来：

Plaintext
请使用 Python (PySide6) + pywin32 + 异步 websockets 帮我实现一个防关联浏览器管理外壳。

要求：
1. UI 参考当前设计。
2. 编写一个后台管理类，维护一个端口字典。能够通过 QProcess 拉起外部独立的 "chromiumfish.exe"，参数需动态传入 --user-data-dir, --proxy-server, --persona-seed, 以及唯一的 --remote-debugging-port。
3. 编写一个 HWND 捕获器：通过拉起的 PID，在 Windows 系统中循环检索类名为 "Chrome_WidgetWin_1" 的有效句柄，并在捕获成功后使用 QWidget.createWindowContainer 强行吸附进右侧对应的 QStackedLayout 页面中。
4. 编写一个 CDP 通信模块：能够连接指定端口的 http://127.0.0.1:{port}/json，过滤出 type 为 "background_page" 的插件后台 WebSocket 地址，并封装一个异步函数，通过 CDP 的 Runtime.evaluate 跨进程向该插件发送 chrome.runtime.sendMessage 通信请求，接收并返回其翻译结果。
5. 必须在 Windows 下引入 Job Object 逻辑，确保主程序闪退时，所有拉起的外部浏览器进程同步强制销毁。
该方案绕过了重型 C++ 魔改编译的灾难级研发成本，完全利用系统级句柄嵌套和 Chromium 自身的不可见节流特性，是目前开发速度最快、性能最强、白嫖生态最稳健的商业级多开解法。