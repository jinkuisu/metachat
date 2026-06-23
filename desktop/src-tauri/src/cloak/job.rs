/// Job Object 进程保护
///
/// Windows: 创建 Job Object + JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE，
/// 将所有 CloakBrowser 子进程绑定到此 Job。主进程退出时，
/// 操作系统自动 TerminateJobObject → 所有子进程瞬间消失。
///
/// 非 Windows: 空操作。

#[cfg(target_os = "windows")]
mod platform {
    use std::sync::Mutex;

    type HANDLE = isize;
    type BOOL = i32;
    type DWORD = u32;
    type LPCWSTR = *const u16;
    type LPVOID = *mut u8;

    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x00002000;
    const PROCESS_ALL_ACCESS: DWORD = 0x001F0FFF;

    #[repr(C)]
    struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: DWORD,
        min_working_set_size: usize,
        max_working_set_size: usize,
        active_process_limit: DWORD,
        affinity: usize,
        child_process_count: DWORD,
        child_process_limit: DWORD,
        reserved: [DWORD; 2],
    }

    #[repr(C)]
    struct IO_COUNTERS {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        basic_limit_information: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        io_info: IO_COUNTERS,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(lpJobAttributes: *mut u8, lpName: LPCWSTR) -> HANDLE;
        fn AssignProcessToJobObject(hJob: HANDLE, hProcess: HANDLE) -> BOOL;
        fn SetInformationJobObject(hJob: HANDLE, job_object_info_class: i32, lpJobObjectInfo: LPVOID, cbJobObjectInfoLength: DWORD) -> BOOL;
        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> BOOL;
    }

    static JOB: Mutex<Option<HANDLE>> = Mutex::new(None);

    /// 初始化 Job Object。应用启动时调用一次。
    pub fn init() -> Result<(), String> {
        let handle = unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null()) };
        if handle == 0 {
            return Err("CreateJobObjectW 失败，无法创建进程保护 Job".into());
        }

        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            basic_limit_information: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                per_process_user_time_limit: 0,
                per_job_user_time_limit: 0,
                limit_flags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                min_working_set_size: 0,
                max_working_set_size: 0,
                active_process_limit: 0,
                affinity: 0,
                child_process_count: 0,
                child_process_limit: 0,
                reserved: [0, 0],
            },
            io_info: IO_COUNTERS {
                read_operation_count: 0,
                write_operation_count: 0,
                other_operation_count: 0,
                read_transfer_count: 0,
                write_transfer_count: 0,
                other_transfer_count: 0,
            },
            process_memory_limit: 0,
            job_memory_limit: 0,
            peak_process_memory_used: 0,
            peak_job_memory_used: 0,
        };

        let ret = unsafe {
            SetInformationJobObject(
                handle,
                9, // JobObjectExtendedLimitInformation
                &mut info as *mut _ as LPVOID,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
            )
        };

        if ret == 0 {
            unsafe { CloseHandle(handle); }
            return Err("SetInformationJobObject (KILL_ON_JOB_CLOSE) 失败".into());
        }

        let mut guard = JOB.lock().unwrap();
        *guard = Some(handle);
        log::info!("Job Object 已创建（KILL_ON_JOB_CLOSE）");
        Ok(())
    }

    /// 将指定 PID 的进程绑定到 Job Object。
    ///
    /// 如果进程已在其他 Job 中（Chromium 自建 Job），
    /// 会返回 ACCESS_DENIED，但这不是致命错误。
    pub fn assign(pid: u32) -> Result<(), String> {
        let job = {
            let guard = JOB.lock().unwrap();
            guard.ok_or("Job Object 未初始化")?
        };

        let proc_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
        if proc_handle == 0 {
            return Err(format!("OpenProcess(PID {}) 失败", pid));
        }

        let ret = unsafe { AssignProcessToJobObject(job, proc_handle) };
        unsafe { CloseHandle(proc_handle); }

        if ret == 0 {
            log::warn!("PID {} 未绑定到 Job — 可能已在其他 Job 中", pid);
        } else {
            log::info!("PID {} 已绑定到 Job Object", pid);
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub fn init() -> Result<(), String> { Ok(()) }
    pub fn assign(_pid: u32) -> Result<(), String> { Ok(()) }
}

pub use platform::*;
