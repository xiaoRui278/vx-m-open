//! macOS 实现。
//!
//! 多开原理（2026-10-20 在微信 4.1.15 实测验证）：
//! 微信 4.x 的单实例限制是沙盒容器内 `app_data/lock/lock.ini` 文件锁，
//! 容器路径由 Bundle ID 决定。因此"克隆 App + 修改 Bundle ID + 重签名"
//! 产生的副本会落入独立容器 → 独立锁 → 可与主程序共存运行。
//! 本工具不修改微信本体。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

use super::{LaunchTarget, WeChatInfo};

const MAIN_APP_NAME: &str = "WeChat.app";
const MAIN_BUNDLE_ID: &str = "com.tencent.xinWeChat";
const MAIN_EXE_SUFFIX: &str = "/Contents/MacOS/WeChat";

type PlatformResult<T> = Result<T, String>;

fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }
    dirs
}

fn app_dir_candidates(name: &str) -> Vec<PathBuf> {
    search_dirs().into_iter().map(|d| d.join(name)).collect()
}

/// 读取 .app 的 (bundle_id, version)，解析失败返回 None
fn read_bundle_info(app_path: &Path) -> Option<(String, String)> {
    let plist_path = app_path.join("Contents/Info.plist");
    let value = plist::Value::from_file(&plist_path).ok()?;
    let dict = value.as_dictionary()?;
    let bundle_id = dict.get("CFBundleIdentifier")?.as_string()?.to_string();
    let version = dict
        .get("CFBundleShortVersionString")
        .and_then(|v| v.as_string())
        .unwrap_or_default()
        .to_string();
    Some((bundle_id, version))
}

pub fn detect() -> PlatformResult<WeChatInfo> {
    for dir in app_dir_candidates(MAIN_APP_NAME) {
        if dir.join("Contents/Info.plist").is_file() {
            let (bundle_id, version) =
                read_bundle_info(&dir).ok_or_else(|| format!("解析失败：{}", dir.display()))?;
            return Ok(WeChatInfo {
                installed: true,
                path: Some(dir.to_string_lossy().into_owned()),
                version: Some(version),
                bundle_id: Some(bundle_id),
            });
        }
    }
    Ok(WeChatInfo {
        installed: false,
        path: None,
        version: None,
        bundle_id: None,
    })
}

/// 正在运行的微信主进程：按 .app 目录路径索引 (pid, started_at)
fn running_map(sys: &System) -> HashMap<String, (u32, u64)> {
    let mut map = HashMap::new();
    for (pid, process) in sys.processes() {
        let Some(exe) = process.exe() else {
            continue;
        };
        let exe = exe.to_string_lossy();
        if !exe.ends_with(MAIN_EXE_SUFFIX) {
            continue;
        }
        let Some(idx) = exe.find("/Contents/MacOS/") else {
            continue;
        };
        map.insert(
            exe[..idx].to_string(),
            (pid.as_u32(), process.start_time()),
        );
    }
    map
}

fn running_state(
    running: &HashMap<String, (u32, u64)>,
    path: &str,
) -> (bool, Option<u32>, Option<u64>) {
    match running.get(path) {
        Some(&(pid, started_at)) => (true, Some(pid), Some(started_at)),
        None => (false, None, None),
    }
}

/// 列出所有可管理目标（主程序 + 副本），并合并运行状态
pub fn launch_targets(main_alias: Option<&str>) -> PlatformResult<Vec<LaunchTarget>> {
    let sys = System::new_all();
    let running = running_map(&sys);
    let mut targets = Vec::new();

    let main = detect()?;
    if let Some(path) = &main.path {
        let (is_running, pid, started_at) = running_state(&running, path);
        targets.push(LaunchTarget {
            path: path.clone(),
            app_name: MAIN_APP_NAME.into(),
            label: main_alias
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("主程序")
                .to_string(),
            version: main.version.clone(),
            is_main: true,
            running: is_running,
            pid,
            started_at,
            outdated: false,
            instance_count: u32::from(is_running),
        });
    }

    // 副本识别：只按 Bundle ID（com.tencent.xinWeChatN，N>=2），
    // 因此任意目录名（含中文自定义名）的副本都能识别
    for dir in search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.join("Contents/Info.plist").is_file() {
                continue;
            }
            let Some((bundle_id, version)) = read_bundle_info(&path) else {
                continue;
            };
            let Some(suffix) = bundle_id.strip_prefix(MAIN_BUNDLE_ID) else {
                continue;
            };
            if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let app_name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let label = app_name.trim_end_matches(".app").to_string();
            let path_str = path.to_string_lossy().into_owned();
            let (is_running, pid, started_at) = running_state(&running, &path_str);
            // 版本同步：副本版本 != 主程序版本 → 落后
            let outdated = !version.is_empty()
                && main.version.as_ref().is_some_and(|v| !v.is_empty() && v != &version);
            targets.push(LaunchTarget {
                path: path_str,
                app_name,
                label,
                version: Some(version),
                is_main: false,
                running: is_running,
                pid,
                started_at,
                outdated,
                instance_count: u32::from(is_running),
            });
        }
    }

    targets.sort_by(|a, b| b.is_main.cmp(&a.is_main).then(a.label.cmp(&b.label)));
    Ok(targets)
}

pub fn launch(path: &str) -> PlatformResult<()> {
    let status = Command::new("/usr/bin/open")
        .arg(path)
        .status()
        .map_err(|e| format!("执行 open 失败：{e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("启动失败：{path}"))
    }
}

/// 激活：macOS 每个 .app 只有一个实例，按路径 open 即可（pid 参数仅 Windows 使用）
pub fn activate(path: &str, pid: Option<u32>) -> PlatformResult<()> {
    let _ = pid;
    let status = Command::new("/usr/bin/open")
        .args(["-a", path])
        .status()
        .map_err(|e| format!("执行 open -a 失败：{e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("激活失败：{path}"))
    }
}

pub fn terminate(pid: u32) -> PlatformResult<()> {
    let target = Pid::from_u32(pid);
    // 复用同一个 System，只刷新目标进程，避免反复全量扫描
    let mut sys = System::new();

    // 先 SIGTERM 优雅退出，2 秒内未退出再 SIGKILL
    sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    match sys.process(target) {
        None => return Ok(()), // 已退出
        Some(p) => match p.kill_with(Signal::Term) {
            Some(true) => {}
            _ => return Err(format!("无法向进程 {pid} 发送退出信号")),
        },
    }
    for _ in 0..13 {
        std::thread::sleep(std::time::Duration::from_millis(150));
        sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
        if sys.process(target).is_none() {
            return Ok(());
        }
    }
    if let Some(p) = sys.process(target) {
        p.kill();
    }
    Ok(())
}

// ---------- 副本管理 ----------

/// 校验并规范化名称：去空白、去 .app 后缀、限长、禁特殊字符
fn sanitize_name(name: &str) -> PlatformResult<String> {
    let mut n = name.trim().to_string();
    if let Some(stripped) = n.strip_suffix(".app") {
        n = stripped.trim_end().to_string();
    }
    if n.is_empty() {
        return Err("名称不能为空".into());
    }
    if n.chars().count() > 32 {
        n = n.chars().take(32).collect();
    }
    if n.eq_ignore_ascii_case("WeChat") {
        return Err("名称不能为 WeChat（会与主程序冲突）".into());
    }
    if n.chars().any(|c| matches!(c, '/' | ':')) {
        return Err("名称不能包含 / 或 :".into());
    }
    Ok(n)
}

/// 扫描全部 .app 的微信系 Bundle ID，返回下一个可用编号（从 2 起）
fn next_bundle_number() -> PlatformResult<u32> {
    let mut max = 1u32;
    for dir in search_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("app") {
                continue;
            }
            if let Some((bid, _)) = read_bundle_info(&path) {
                if let Some(suffix) = bid.strip_prefix(MAIN_BUNDLE_ID) {
                    if let Ok(n) = suffix.parse::<u32>() {
                        max = max.max(n);
                    }
                }
            }
        }
    }
    Ok(max + 1)
}

/// 修改副本 Info.plist：Bundle ID / 名称
fn rewrite_plist(app: &Path, name: Option<&str>, bundle_id: Option<&str>) -> PlatformResult<()> {
    let plist_path = app.join("Contents/Info.plist");
    let mut value = plist::Value::from_file(&plist_path).map_err(|e| e.to_string())?;
    let dict = value
        .as_dictionary_mut()
        .ok_or_else(|| "Info.plist 格式异常".to_string())?;
    if let Some(bid) = bundle_id {
        dict.insert(
            "CFBundleIdentifier".into(),
            plist::Value::String(bid.to_string()),
        );
    }
    if let Some(name) = name {
        dict.insert("CFBundleName".into(), plist::Value::String(name.into()));
        dict.insert(
            "CFBundleDisplayName".into(),
            plist::Value::String(name.into()),
        );
    }
    value
        .to_file_xml(&plist_path)
        .map_err(|e| format!("写入 Info.plist 失败：{e}"))
}

fn codesign_adhoc(app: &Path) -> PlatformResult<()> {
    let out = Command::new("/usr/bin/codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(app)
        .output()
        .map_err(|e| format!("执行 codesign 失败：{e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "重签名失败：{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn cleanup_dir(app: &Path) {
    if app.exists() {
        let _ = std::fs::remove_dir_all(app);
    }
}

/// 确认目标是副本而非主程序
fn ensure_copy(app: &Path) -> PlatformResult<String> {
    if app.file_name().and_then(|n| n.to_str()) == Some(MAIN_APP_NAME) {
        return Err("不能对主程序执行此操作".into());
    }
    let (bundle_id, _) =
        read_bundle_info(app).ok_or_else(|| format!("无法读取应用信息：{}", app.display()))?;
    let Some(suffix) = bundle_id.strip_prefix(MAIN_BUNDLE_ID) else {
        return Err("该应用不是本工具识别的微信副本".into());
    };
    if suffix.is_empty() {
        return Err("不能对主程序执行此操作".into());
    }
    Ok(bundle_id)
}

fn is_running(app: &Path) -> bool {
    let path_str = app.to_string_lossy().into_owned();
    let sys = System::new_all();
    running_map(&sys).contains_key(&path_str)
}

/// 创建副本：APFS 克隆 → 改 Info.plist → 重签名
pub fn generate(name: &str) -> PlatformResult<()> {
    let main = detect()?;
    let Some(src) = main.path else {
        return Err("未检测到微信主程序，无法创建副本".into());
    };
    let src = PathBuf::from(src);
    let name = sanitize_name(name)?;
    let bundle_id = format!("{}{}", MAIN_BUNDLE_ID, next_bundle_number()?);

    // 目标目录优先选主程序所在目录，权限不足时逐个回退
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(parent) = src.parent() {
        candidates.push(parent.to_path_buf());
    }
    for d in search_dirs() {
        if !candidates.contains(&d) {
            candidates.push(d);
        }
    }

    let mut last_err = String::new();
    for dir in candidates {
        let dest = dir.join(format!("{name}.app"));
        if dest.exists() {
            return Err(format!("已存在同名应用：{}", dest.display()));
        }
        match clone_and_sign(&src, &dest, &name, &bundle_id) {
            Ok(()) => return Ok(()),
            Err(e) => {
                cleanup_dir(&dest);
                last_err = e;
            }
        }
    }
    Err(format!("创建副本失败：{last_err}"))
}

fn clone_and_sign(
    src: &Path,
    dest: &Path,
    name: &str,
    bundle_id: &str,
) -> PlatformResult<()> {
    // APFS 克隆（cp -c），非 APFS 卷回退普通复制
    let cloned = Command::new("/bin/cp")
        .args(["-Rc"])
        .arg(src)
        .arg(dest)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !cloned {
        let copied = Command::new("/bin/cp")
            .args(["-R"])
            .arg(src)
            .arg(dest)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !copied {
            return Err("复制应用文件失败（权限不足或磁盘空间不够）".into());
        }
    }
    rewrite_plist(dest, Some(name), Some(bundle_id))?;
    codesign_adhoc(dest)
}

/// 重命名副本：Bundle ID 不变（数据容器与账号数据不受影响），
/// 仅改显示名 + 目录名 + 重签名
pub fn rename(path: &str, new_name: &str) -> PlatformResult<()> {
    let app = PathBuf::from(path);
    ensure_copy(&app)?;
    let name = sanitize_name(new_name)?;
    if is_running(&app) {
        return Err("副本正在运行，请先关闭再改名".into());
    }

    rewrite_plist(&app, Some(&name), None)?;
    let parent = app.parent().ok_or_else(|| "路径异常".to_string())?;
    let dest = parent.join(format!("{name}.app"));
    if dest.exists() {
        return Err(format!("已存在同名应用：{}", dest.display()));
    }
    std::fs::rename(&app, &dest).map_err(|e| format!("目录重命名失败：{e}"))?;
    codesign_adhoc(&dest)
}

/// 删除副本：移到废纸篓（可反悔），账号数据保留在独立容器中
pub fn delete(path: &str) -> PlatformResult<()> {
    let app = PathBuf::from(path);
    ensure_copy(&app)?;
    if is_running(&app) {
        return Err("副本正在运行，请先关闭再删除".into());
    }
    trash::delete(app).map_err(|e| format!("移到废纸篓失败：{e}"))
}

/// 重建版本落后的副本：保留原 Bundle ID 与名字。
/// 数据容器由 Bundle ID 决定，因此账号数据与登录状态完全不受影响。
pub fn rebuild(path: &str) -> PlatformResult<()> {
    let app = PathBuf::from(path);
    let bundle_id = ensure_copy(&app)?;
    if is_running(&app) {
        return Err("副本正在运行，请先关闭再重建".into());
    }
    let main = detect()?;
    let Some(src) = main.path else {
        return Err("未检测到微信主程序，无法重建".into());
    };
    let src = PathBuf::from(src);

    let name = app
        .file_stem()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| "路径异常".to_string())?;
    let parent = app.parent().ok_or_else(|| "路径异常".to_string())?;
    let dest = parent.join(format!("{name}.app"));

    // 先移除旧副本，再克隆新版主程序
    trash::delete(&app).map_err(|e| format!("移除旧副本失败：{e}"))?;
    if let Err(e) = clone_and_sign(&src, &dest, &name, &bundle_id) {
        cleanup_dir(&dest);
        return Err(format!("重建失败：{e}（旧副本已在废纸篓，可手动恢复）"));
    }
    Ok(())
}
