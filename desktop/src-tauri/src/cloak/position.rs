/// 窗口定位 — 查找 CloakBrowser 窗口并按指定位置/大小放置

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>, lParam: isize) -> i32;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn MoveWindow(hWnd: isize, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;

    fn GetWindowLongW(hWnd: isize, nIndex: i32) -> i32;
    fn SetWindowLongW(hWnd: isize, nIndex: i32, dwNewLong: i32) -> i32;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
}


#[cfg(target_os = "windows")]
pub fn position_window(pid: u32, x: i32, y: i32, width: i32, height: i32) -> bool {
    if let Some(hwnd) = find_window_by_pid(pid) {
        unsafe { MoveWindow(hwnd, x, y, width, height, 1); }
        log::info!("Positioned PID {} at ({},{}) {}x{}", pid, x, y, width, height);
        true
    } else {
        log::warn!("Window not found for PID {}", pid);
        false
    }
}

#[cfg(target_os = "windows")]
fn find_window_by_pid(pid: u32) -> Option<isize> {
    use std::sync::Mutex;
    static mut FOUND_HWND: isize = 0;
    static LOCK: Mutex<()> = Mutex::new(());
    unsafe extern "system" fn enum_cb(hwnd: isize, lparam: isize) -> i32 {
        let target = lparam as u32;
        let mut actual: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut actual);
        if actual == target { FOUND_HWND = hwnd; return 0; }
        1
    }
    unsafe {
        FOUND_HWND = 0;
        let _lock = LOCK.lock().unwrap();
        EnumWindows(Some(enum_cb), pid as isize);
        if FOUND_HWND != 0 { Some(FOUND_HWND) } else { None }
    }
}


const GWL_STYLE: i32 = -16;
const WS_CAPTION: i32 = 0x00C00000;
const WS_SYSMENU: i32 = 0x00080000;
const WS_MINIMIZEBOX: i32 = 0x00020000;
const WS_MAXIMIZEBOX: i32 = 0x00010000;
const WS_THICKFRAME: i32 = 0x00040000;
const WS_OVERLAPPEDWINDOW: i32 = 0x00CF0000;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;

/// 移除窗口标题栏、系统菜单、最小化/最大化/关闭按钮
/// 使 CloakBrowser --app 窗口变为纯内容面板
#[cfg(target_os = "windows")]
pub fn remove_window_chrome(pid: u32) -> bool {
    if let Some(hwnd) = find_window_by_pid(pid) {
        unsafe {
            // 读取当前样式，去掉标题栏、系统菜单、按钮、可调边框
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            let new_style = (style & !WS_OVERLAPPEDWINDOW) 
                | (0x8000_0000u32 as i32)  // WS_POPUP — 弹出窗口无边框
                | (0x1000_0000u32 as i32)  // WS_VISIBLE
                | (0x0400_0000u32 as i32)  // WS_CLIPSIBLINGS
                | (0x0200_0000u32 as i32); // WS_CLIPCHILDREN
            SetWindowLongW(hwnd, GWL_STYLE, new_style);
            // 从任务栏隐藏
            let ex_style = GetWindowLongW(hwnd, -20); // GWL_EXSTYLE
            SetWindowLongW(hwnd, -20, ex_style & !(0x0004_0000u32 as i32) | (0x0000_0080u32 as i32));
            SetWindowPos(hwnd, 0, 0, 0, 0, 0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER);
        }
        log::info!("Removed window chrome from PID {}", pid);
        true
    } else {
        log::warn!("Window not found for PID {} (remove_chrome)", pid);
        false
    }
}

#[cfg(not(target_os = "windows"))]
pub fn remove_window_chrome(_pid: u32) -> bool { false }

/// 将 CloakBrowser 窗口嵌入 Tauri 父窗口，去掉浏览器装饰
/// 配合 CDP Browser.setWindowBounds(fullscreen) 实现无头纯内容
#[cfg(target_os = "windows")]
pub fn embed_window(pid: u32, parent_hwnd: isize) -> Result<(), String> {
    let child_hwnd = find_window_by_pid(pid)
        .ok_or_else(|| format!("找不到 PID {} 的窗口", pid))?;
    unsafe {
        // 1. 设为子窗口
        extern "system" {
            fn SetParent(hWndChild: isize, hWndNewParent: isize) -> isize;
        }
        SetParent(child_hwnd, parent_hwnd);

        // 2. 去掉所有浏览器装饰
        let style = GetWindowLongW(child_hwnd, GWL_STYLE);
        let new_style = (style as u32 
            & !(0x00CF0000u32)  // WS_OVERLAPPEDWINDOW
            & !0x00C00000u32   // WS_CAPTION
            & !0x00080000u32   // WS_SYSMENU
            & !0x00040000u32   // WS_THICKFRAME
            & !0x00020000u32   // WS_MINIMIZEBOX
            & !0x00010000u32   // WS_MAXIMIZEBOX
            | 0x40000000u32    // WS_CHILD
            | 0x10000000u32)   // WS_VISIBLE
            as i32;
        SetWindowLongW(child_hwnd, GWL_STYLE, new_style);

        // 3. 刷新边框
        SetWindowPos(child_hwnd, 0, 0, 0, 0, 0,
            0x0020 | 0x0002 | 0x0001 | 0x0004);  // SWP_FRAMECHANGED | NOMOVE | NOSIZE | NOZORDER

        log::info!("Embedded PID {} into parent HWND {}", pid, parent_hwnd);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn embed_window(_pid: u32, _parent: isize) -> Result<(), String> {
    Err("只支持 Windows".into())
}


/// 隐藏窗口（移到屏幕外，进程保持运行）
#[cfg(target_os = "windows")]
pub fn hide_window(pid: u32) -> bool {
    if let Some(hwnd) = find_window_by_pid(pid) {
        unsafe { MoveWindow(hwnd, -9999, -9999, 1, 1, 1); }
        log::info!("Moved PID {} off-screen", pid);
        true
    } else {
        log::warn!("Window not found for PID {} (hide)", pid);
        false
    }
}

/// 显示窗口到指定位置（从屏幕外移回）
#[cfg(target_os = "windows")]
pub fn show_window(pid: u32, x: i32, y: i32, w: i32, h: i32) -> bool {
    if let Some(hwnd) = find_window_by_pid(pid) {
        unsafe { MoveWindow(hwnd, x, y, w, h, 1); }
        log::info!("Moved PID {} to ({},{}) {}x{}", pid, x, y, w, h);
        true
    } else {
        log::warn!("Window not found for PID {} (show)", pid);
        false
    }
}

#[cfg(not(target_os = "windows"))]
pub fn hide_window(_pid: u32) -> bool { false }

#[cfg(not(target_os = "windows"))]
pub fn show_window(_pid: u32, _x: i32, _y: i32, _w: i32, _h: i32) -> bool { false }

#[cfg(not(target_os = "windows"))]
pub fn position_window(_pid: u32, _x: i32, _y: i32, _w: i32, _h: i32) -> bool { false }
