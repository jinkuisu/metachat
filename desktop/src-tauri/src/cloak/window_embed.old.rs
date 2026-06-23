/// 子窗口嵌入管理 - 去掉浏览器外壳
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetParent(hWndChild: isize, hWndNewParent: isize) -> isize;
    fn MoveWindow(hWnd: isize, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;
    fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>, lParam: isize) -> i32;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn GetWindowLongW(hWnd: isize, nIndex: i32) -> i32;
    fn SetWindowLongW(hWnd: isize, nIndex: i32, dwNewLong: i32) -> i32;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
}

#[cfg(target_os = "windows")]
const GWL_STYLE: i32 = -16;
const CHILD: i32 = 0x40000000;
const VISIBLE: i32 = 0x10000000;
const CAPTION: i32 = 0x00C00000;
const THICKFRAME: i32 = 0x00040000;
const SYSMENU: i32 = 0x00080000;
const MINIMIZE: i32 = 0x00020000;
const MAXIMIZE: i32 = 0x00010000;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;

pub struct WindowEmbed {
    pub cloak_hwnd: Option<isize>,
    pub child_pid: u32,
    is_embedded: AtomicBool,
}

impl WindowEmbed {
    pub fn new(child_pid: u32) -> Self {
        Self { cloak_hwnd: None, child_pid, is_embedded: AtomicBool::new(false) }
    }

    /// 嵌入 + 去除标题栏
    pub fn embed(&mut self, parent_hwnd: isize) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let child = find_window_by_pid(self.child_pid)
                .ok_or_else(|| format!("Window not found for PID {}", self.child_pid))?;
            unsafe {
                SetParent(child, parent_hwnd);
                let style = GetWindowLongW(child, GWL_STYLE) as u32;
                let new_style = (style
                    & !(CAPTION as u32)
                    & !(THICKFRAME as u32)
                    & !(SYSMENU as u32)
                    & !(MINIMIZE as u32)
                    & !(MAXIMIZE as u32)
                    | CHILD as u32
                    | VISIBLE as u32) as i32;
                SetWindowLongW(child, GWL_STYLE, new_style);
                                // Move below WebView2 so Svelte controls float on top
                SetWindowPos(child, 1, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);                SetWindowPos(child, 0, 0, 0, 0, 0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER);
                MoveWindow(child, 0, -48, 1400, 948, 1); // title bar hidden above view
            }
            self.cloak_hwnd = Some(child);
            self.is_embedded.store(true, Ordering::SeqCst);
            log::info!("Embedded {} into {} (no chrome)", child, parent_hwnd);
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        Err("Not supported on this platform".into())
    }

    pub fn resize(&self, x: i32, y: i32, width: i32, height: i32) {
        if !self.is_embedded.load(Ordering::SeqCst) { return; }
        #[cfg(target_os = "windows")]
        if let Some(hwnd) = self.cloak_hwnd {
            unsafe { MoveWindow(hwnd, x, y, width, height, 1); }
        }
    }

    pub fn unembed(&mut self) {
        self.is_embedded.store(false, Ordering::SeqCst);
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

/// 存储当前嵌入窗口的 HWND，用于 resize 事件
static EMBED_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn set_embed_hwnd(hwnd: isize) { EMBED_HWND.store(hwnd, Ordering::SeqCst); }
pub fn get_embed_hwnd() -> isize { EMBED_HWND.load(Ordering::SeqCst) }
pub fn clear_embed_hwnd() { EMBED_HWND.store(0, Ordering::SeqCst); }

/// 从 resize 处理器调用：调整嵌入窗口大小
#[cfg(target_os = "windows")]
pub fn resize_embed(x: i32, y: i32, w: i32, h: i32) {
    let hwnd = get_embed_hwnd();
    if hwnd != 0 {
        unsafe {
            extern "system" {
                fn MoveWindow(hWnd: isize, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;
            }
            MoveWindow(hwnd as isize, x, y - 48, w, h + 48, 1); // hide title bar
        }
    }
}
