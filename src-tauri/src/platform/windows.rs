//! Windows 实现。
//!
//! 多开原理：以挂起状态启动微信进程，恢复执行后立即轮询其句柄表，
//! 找到单实例互斥体（名称含 `_WeChat_App_Instance_Identity_Mutex_Name`）
//! 并关闭，微信随后检查单实例锁时将认为自己第一个启动，从而多开。
//! 不修改微信本体，不产生副本文件。

use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use sysinfo::System;
use windows_sys::Win32::Foundation::{
    CloseHandle, DuplicateHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, Thread32First, Thread32Next,
    TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, PROCESSENTRY32W, THREADENTRY32,
};
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, OpenProcess, QueryFullProcessImageNameW, ResumeThread,
    TerminateProcess, WaitForSingleObject, CREATE_SUSPENDED, PROCESS_INFORMATION,
    PROCESS_DUP_HANDLE, PROCESS_NAME_WIN32, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    STARTUPINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostThreadMessageW,
    SetForegroundWindow, ShowWindow, SW_RESTORE, WM_QUIT,
};
// windows-sys 0.61：NtQueryObject 在 Wdk::Foundation，NtQuerySystemInformation 在 Wdk::System::SystemInformation
use windows_sys::Wdk::Foundation::NtQueryObject;
use windows_sys::Wdk::System::SystemInformation::NtQuerySystemInformation;

use super::{LaunchTarget, WeChatInfo};

type PlatformResult<T> = Result<T, String>;

// ---- NT 常量（windows-sys 未导出的句柄表结构与常量，自定义） ----
const SYSTEM_EXTENDED_HANDLE_INFORMATION: i32 = 0x40;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004u32 as i32;
const OBJECT_TYPE_INFORMATION: i32 = 2;
const OBJECT_NAME_INFORMATION: i32 = 1;
const DUPLICATE_CLOSE_SOURCE: u32 = 0x0000_0001;
const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;
const WAIT_OBJECT_0: u32 = 0;

const MUTANT_TYPE_NAME: &str = "Mutant";
const WECHAT_MUTEX_MARK: &str = "WeChat_App_Instance";

const EXE_NAMES: [&str; 2] = ["WeChat.exe", "Weixin.exe"];
const REG_SUBKEYS: [&str; 2] = ["Software\\Tencent\\WeChat", "Software\\Tencent\\Weixin"];
const DEFAULT_DIRS: [&str; 3] = [
    "C:\\Program Files\\Tencent\\WeChat",
    "C:\\Program Files\\Tencent\\Weixin",
    "C:\\Program Files (x86)\\Tencent\\WeChat",
];

const NO_COPY: &str = "Windows 端直接多开微信本体，无需副本，此操作不可用";

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// 启动时注册表 / 默认路径都找不到 exe 时的提示
const NOT_INSTALLED: WeChatInfo = WeChatInfo {
    installed: false,
    path: None,
    version: None,
    bundle_id: None,
};

// ---------- 检测 ----------

fn parse_version(s: &str) -> Option<[u64; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut v = [0u64; 4];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        v[i] = p.parse().ok()?;
    }
    Some(v)
}

/// 读注册表 InstallPath（微信 4.x 键名 Weixin，3.x 键名 WeChat）
fn registry_install_dir(subkey: &str) -> Option<String> {
    let subkey_w = to_wide(subkey);
    let value_w = to_wide("InstallPath");
    let mut buf = [0u16; 512];
    let mut size = (buf.len() * 2) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            value_w.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
        )
    };
    if status != 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    let s = String::from_utf16_lossy(&buf[..len]);
    let s = s.trim_end_matches(['\\', '/']);
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// 安装目录下找版本子目录（形如 4.0.3.22）内的 exe，取版本最高的
fn find_exe(dir: &str) -> Option<(PathBuf, String)> {
    let base = PathBuf::from(dir);
    let mut best: Option<([u64; 4], PathBuf, String)> = None;
    if let Ok(rd) = std::fs::read_dir(&base) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(ver) = parse_version(&name) else {
                continue;
            };
            for exe in EXE_NAMES {
                let p = base.join(&name).join(exe);
                if p.is_file() {
                    if best.as_ref().is_none_or(|(v, _, _)| ver > *v) {
                        best = Some((ver, p, name.clone()));
                    }
                    break;
                }
            }
        }
    }
    if let Some((_, p, ver)) = best {
        return Some((p, ver));
    }
    // 兜底：exe 直接放在安装目录（无版本子目录的旧结构）
    for exe in EXE_NAMES {
        let p = base.join(exe);
        if p.is_file() {
            return Some((p, String::new()));
        }
    }
    None
}

fn build_info(exe: PathBuf, version: String) -> WeChatInfo {
    WeChatInfo {
        installed: true,
        path: Some(exe.to_string_lossy().into_owned()),
        version: if version.is_empty() { None } else { Some(version) },
        bundle_id: None,
    }
}

pub fn detect() -> PlatformResult<WeChatInfo> {
    for subkey in REG_SUBKEYS {
        if let Some(dir) = registry_install_dir(subkey) {
            if let Some((exe, version)) = find_exe(&dir) {
                return Ok(build_info(exe, version));
            }
        }
    }
    for dir in DEFAULT_DIRS {
        if let Some((exe, version)) = find_exe(dir) {
            return Ok(build_info(exe, version));
        }
    }
    Ok(NOT_INSTALLED)
}

/// 列出主程序（含运行实例数）与每个运行中的实例
pub fn launch_targets(main_alias: Option<&str>) -> PlatformResult<Vec<LaunchTarget>> {
    let info = detect()?;
    let Some(exe) = info.path.clone() else {
        return Ok(Vec::new());
    };
    let exe_lower = exe
        .trim_start_matches(r"\\?\")
        .to_ascii_lowercase();
    // 按进程映像名匹配（Weixin.exe / WeChat.exe），路径全串比对易受
    // \\?\ 前缀、8.3 短路径、注册表路径陈旧等影响而漏检运行实例
    let exe_name = Path::new(&exe)
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    let sys = System::new_all();
    let mut instances: Vec<(u32, u64)> = sys
        .processes()
        .values()
        .filter(|p| {
            let name_hit = p.name().to_string_lossy().to_ascii_lowercase() == exe_name;
            let path_hit = p
                .exe()
                .is_some_and(|e: &Path| e.to_string_lossy().to_ascii_lowercase() == exe_lower);
            (name_hit || path_hit)
                // 排除 Chromium 风格子进程（--type=renderer 等）
                && !p.cmd().iter().any(|a| a.to_string_lossy().starts_with("--type="))
        })
        .map(|p| (p.pid().as_u32(), p.start_time()))
        .collect();
    instances.sort_by_key(|&(_, t)| t);

    let count = instances.len() as u32;
    let app_name = Path::new(&exe)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut targets = vec![LaunchTarget {
        path: exe.clone(),
        app_name,
        label: main_alias
            .filter(|s| !s.trim().is_empty())
            // 语言中立标记值，前端对主程序做 i18n 映射（微信 / WeChat）
            .unwrap_or("WeChat")
            .to_string(),
        version: info.version.clone(),
        is_main: true,
        running: count > 0,
        pid: None,
        started_at: instances.first().map(|&(_, t)| t),
        outdated: false,
        instance_count: count,
    }];

    for (i, &(pid, started)) in instances.iter().enumerate() {
        targets.push(LaunchTarget {
            path: exe.clone(),
            app_name: String::new(),
            label: format!("Instance {}", i + 1),
            version: info.version.clone(),
            is_main: false,
            running: true,
            pid: Some(pid),
            started_at: Some(started),
            outdated: false,
            instance_count: 1,
        });
    }
    Ok(targets)
}

// ---------- 互斥体处理 ----------

#[repr(C)]
struct SystemHandleInfoEx {
    number_of_handles: usize,
    reserved: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HandleTableEntryEx {
    object: *mut c_void,
    unique_process_id: usize,
    handle_value: usize,
    granted_access: u32,
    creator_back_trace_index: u16,
    object_type_index: u16,
    handle_attributes: u32,
    reserved: u32,
}

/// NtQueryObject 返回的 OBJECT_TYPE_INFORMATION / OBJECT_NAME_INFORMATION
/// 首字段均为 UNICODE_STRING
#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

/// 读取 UNICODE_STRING（原始指针指向缓冲区首部）
unsafe fn read_unicode_string(raw: *const u8) -> Option<String> {
    let us = &*(raw as *const UnicodeString);
    if us.length == 0 || us.buffer.is_null() {
        return None;
    }
    let slice = std::slice::from_raw_parts(us.buffer, us.length as usize / 2);
    Some(String::from_utf16_lossy(slice))
}

/// 查询句柄的类型名 / 对象名（仅对复制到本进程的句柄调用，降低卡死风险）
fn query_object_string(handle: HANDLE, class: i32) -> Option<String> {
    let mut size = 1024usize;
    loop {
        let mut buf = vec![0u8; size];
        let mut ret = 0u32;
        let status = unsafe {
            NtQueryObject(
                handle,
                class,
                buf.as_mut_ptr() as *mut c_void,
                size as u32,
                &mut ret,
            )
        };
        if status == 0 {
            return unsafe { read_unicode_string(buf.as_ptr()) };
        }
        if status == STATUS_INFO_LENGTH_MISMATCH {
            size *= 2;
            if size > 64 * 1024 {
                return None;
            }
            continue;
        }
        return None;
    }
}

/// 尝试关闭微信进程的单实例互斥体句柄；成功返回 Ok(true)
fn close_wechat_mutex(pid: u32) -> Result<bool, String> {
    let target = unsafe { OpenProcess(PROCESS_DUP_HANDLE, 0, pid) };
    if target.is_null() {
        return Err(format!("无法打开微信进程（PID {pid}）"));
    }
    let result = close_wechat_mutex_inner(target, pid);
    unsafe { CloseHandle(target) };
    result
}

fn close_wechat_mutex_inner(target: HANDLE, pid: u32) -> Result<bool, String> {
    let me = unsafe { GetCurrentProcess() };

    // 枚举全系统句柄表（缓冲区不足时翻倍重试）
    let mut size = 1usize << 20;
    let entries: Vec<HandleTableEntryEx> = loop {
        let mut buf = vec![0u8; size];
        let status = unsafe {
            NtQuerySystemInformation(
                SYSTEM_EXTENDED_HANDLE_INFORMATION,
                buf.as_mut_ptr() as *mut c_void,
                size as u32,
                std::ptr::null_mut(),
            )
        };
        if status == 0 {
            let (count, base) = unsafe {
                let header = &*(buf.as_ptr() as *const SystemHandleInfoEx);
                let count = header.number_of_handles.min(1 << 22);
                let base = buf.as_ptr().add(std::mem::size_of::<SystemHandleInfoEx>())
                    as *const HandleTableEntryEx;
                (count, base)
            };
            let items: Vec<HandleTableEntryEx> =
                (0..count).map(|i| unsafe { *base.add(i) }).collect();
            break items;
        }
        if status == STATUS_INFO_LENGTH_MISMATCH {
            size *= 2;
            if size > 1usize << 26 {
                return Err("枚举系统句柄失败：缓冲区不足".into());
            }
            continue;
        }
        return Err(format!("枚举系统句柄失败（NTSTATUS 0x{status:08X}）"));
    };

    for e in &entries {
        if e.unique_process_id != pid as usize {
            continue;
        }
        // 先复制到本进程再查询，避免直接操作远程句柄
        let mut dup: HANDLE = std::ptr::null_mut();
        if unsafe {
            DuplicateHandle(target, e.handle_value as HANDLE, me, &mut dup, 0, 0, DUPLICATE_SAME_ACCESS)
        } == 0
        {
            continue;
        }
        let type_name = query_object_string(dup, OBJECT_TYPE_INFORMATION);
        if type_name.as_deref() == Some(MUTANT_TYPE_NAME) {
            let obj_name = query_object_string(dup, OBJECT_NAME_INFORMATION);
            if obj_name.is_some_and(|n| n.contains(WECHAT_MUTEX_MARK)) {
                // 在源进程内关闭原句柄
                let closed = unsafe {
                    DuplicateHandle(
                        target,
                        e.handle_value as HANDLE,
                        target,
                        std::ptr::null_mut(),
                        0,
                        0,
                        DUPLICATE_CLOSE_SOURCE,
                    )
                };
                unsafe { CloseHandle(dup) };
                if closed != 0 {
                    return Ok(true);
                }
                continue;
            }
        }
        unsafe { CloseHandle(dup) };
    }
    Ok(false)
}

// ---------- 启动 / 激活 / 关闭 ----------

/// 以挂起方式启动微信，恢复执行后轮询关闭单实例互斥体
pub fn launch(path: &str) -> PlatformResult<()> {
    let exe_wide = to_wide(path);
    let mut cmdline = to_wide(&format!("\"{path}\""));
    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    let ok = unsafe {
        CreateProcessW(
            exe_wide.as_ptr(),
            cmdline.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            CREATE_SUSPENDED,
            std::ptr::null(),
            std::ptr::null(),
            &si,
            &mut pi,
        )
    };
    if ok == 0 {
        return Err(format!(
            "启动微信失败（Win32 错误码 {}）",
            unsafe { GetLastError() }
        ));
    }
    let pid = pi.dwProcessId;
    unsafe {
        ResumeThread(pi.hThread);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
    }

    // 微信在初始化早期创建单实例互斥体，10 秒窗口内轮询关闭。
    // 单次枚举失败（缓冲区抖动/权限抖动）只跳过本轮，不放弃整个窗口；
    // 150ms 间隔在成功率与 CPU 开销间取平衡（互斥体在初始化早期即创建）。
    let mut last_err: Option<String> = None;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match close_wechat_mutex(pid) {
            Ok(true) => return Ok(()),
            Ok(false) => std::thread::sleep(Duration::from_millis(150)),
            Err(e) => {
                last_err = Some(e);
                std::thread::sleep(Duration::from_millis(300));
            }
        }
    }
    match last_err {
        Some(e) => Err(format!("启动超时：{e}（该版本微信可能更改了内部机制）")),
        None => Err("启动超时：未能解除微信单实例限制（该版本微信可能更改了内部机制）".into()),
    }
}

unsafe extern "system" fn enum_proc(hwnd: HANDLE, lparam: isize) -> i32 {
    let list = &mut *(lparam as *mut Vec<HANDLE>);
    list.push(hwnd);
    1
}

/// 按 PID 激活窗口（多实例共用同一 exe 路径，只能按进程区分）
pub fn activate(path: &str, pid: Option<u32>) -> PlatformResult<()> {
    let _ = path;
    let Some(pid) = pid else {
        return Err("缺少进程 ID，无法激活窗口".into());
    };
    let mut wins: Vec<HANDLE> = Vec::new();
    unsafe { EnumWindows(Some(enum_proc), &mut wins as *mut _ as isize) };
    for hwnd in wins {
        let mut wpid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut wpid) };
        if wpid == pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            unsafe {
                if IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, SW_RESTORE);
                }
                SetForegroundWindow(hwnd);
            }
            return Ok(());
        }
    }
    Err(format!("未找到进程 {pid} 的可激活窗口"))
}

/// 找到进程主线程（第一个线程），用于投递 WM_QUIT 优雅退出
fn main_thread_id(pid: u32) -> Option<u32> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snap == INVALID_HANDLE_VALUE {
        return None;
    }
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let mut result = None;
    unsafe {
        if Thread32First(snap, &mut entry) != 0 {
            loop {
                if entry.th32OwnerProcessID == pid {
                    result = Some(entry.th32ThreadID);
                    break;
                }
                if Thread32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    result
}

/// 快照 pid 的全部后代进程（BFS 遍历父→子映射），返回 (pid, exe 文件名)。
/// 必须在主进程退出前调用：主进程死后子进程被系统收养，父指针失效无法再定位。
fn descendant_pids(pid: u32) -> Vec<(u32, String)> {
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let mut procs: Vec<(u32, u32, String)> = Vec::new(); // (pid, parent_pid, exe 名)
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    unsafe {
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                procs.push((entry.th32ProcessID, entry.th32ParentProcessID, name));
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    let mut result: Vec<(u32, String)> = Vec::new();
    let mut queue = vec![pid];
    let mut i = 0;
    while i < queue.len() {
        let cur = queue[i];
        i += 1;
        for &(p, parent, ref name) in &procs {
            if parent == cur && p != 0 && !queue.contains(&p) {
                queue.push(p);
                result.push((p, name.clone()));
            }
        }
    }
    result
}

/// 查询进程可执行文件名（不含路径，小写），失败返回 None
fn process_image_name(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let mut size = buf.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            buf.as_mut_ptr(),
            &mut size,
        )
    };
    if ok == 0 {
        return None;
    }
    let full = String::from_utf16_lossy(&buf[..size as usize]);
    Path::new(&full)
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
}

/// 终止单个帮手进程。收割前校验 exe 名与快照一致，防止收割窗口内 pid 复用
/// 或快照孤儿误配时误杀无关进程；进程已消失则静默跳过
fn kill_process_hard(pid: u32, exe_name: &str) {
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
    if handle.is_null() {
        return; // 已不存在
    }
    let matches = process_image_name(handle)
        .is_some_and(|name| name == exe_name.to_ascii_lowercase());
    if matches {
        // 给 200ms 余量让其在微信正常关闭流程中自行退出，未退再强杀
        if unsafe { WaitForSingleObject(handle, 200) } != WAIT_OBJECT_0 {
            unsafe {
                TerminateProcess(handle, 1);
                let _ = WaitForSingleObject(handle, 2000);
            }
        }
    }
    unsafe { CloseHandle(handle) };
}

/// 先 WM_QUIT 优雅退出主进程，2 秒未退出再强制结束；随后收割残留的帮手子进程
pub fn terminate(pid: u32) -> PlatformResult<()> {
    // 必须在主进程退出前快照子进程树，死后父指针失效
    let children = descendant_pids(pid);
    if let Some(tid) = main_thread_id(pid) {
        unsafe { PostThreadMessageW(tid, WM_QUIT, 0, 0) };
    }
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
    if !handle.is_null() {
        if unsafe { WaitForSingleObject(handle, 2000) } != WAIT_OBJECT_0 {
            unsafe {
                TerminateProcess(handle, 1);
                let _ = WaitForSingleObject(handle, 3000);
            }
        }
        unsafe { CloseHandle(handle) };
    }
    // 正常退出时微信会自己带走帮手，此处只兜底收割残留者
    for (child, name) in children {
        kill_process_hard(child, &name);
    }
    Ok(())
}

// ---------- 副本操作（Windows 不适用） ----------

pub fn generate_with_progress(_name: &str, _on_step: &dyn Fn(u32)) -> PlatformResult<()> {
    Err(NO_COPY.into())
}

pub fn rename(_path: &str, _new_name: &str) -> PlatformResult<()> {
    Err(NO_COPY.into())
}

pub fn delete(_path: &str) -> PlatformResult<()> {
    Err(NO_COPY.into())
}

pub fn rebuild(_path: &str) -> PlatformResult<()> {
    Err(NO_COPY.into())
}
