//! macOS 策略实现（clone 模型）。
//!
//! 多开原理（2026-10-20 在微信 4.1.15 实测验证）：
//! 微信 4.x 的单实例限制是沙盒容器内 `app_data/lock/lock.ini` 文件锁，
//! 容器路径由 Bundle ID 决定。因此"克隆 App + 修改 Bundle ID + 重签名"
//! 产生的副本会落入独立容器 → 独立锁 → 可与主程序共存运行。
//! 本工具不修改微信本体。
//!
//! 应用专属参数（App 名、Bundle ID 前缀、搜索目录等）全部来自
//! `crate::profiles` 的 macOS 声明；本文件只含策略机制，无应用专属常量。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

use super::engine::EnginePlatform;
use super::{LaunchTarget, PlatformResult, WeChatInfo};
use crate::profiles::schema::{
    CopyConfig, CopyStrategy, MacOsDetect, MacOsPlatform, MacMultiInstance, SignStrategy,
    TerminateConfig,
};

/// 展开候选目录：`~` 前缀替换为用户主目录
fn search_dirs(dirs: &[String]) -> Vec<PathBuf> {
    let home = std::env::var("HOME").ok();
    dirs.iter()
        .map(|d| match (d.strip_prefix("~/"), &home) {
            (Some(rest), Some(h)) => PathBuf::from(h).join(rest),
            _ => PathBuf::from(d),
        })
        .collect()
}

/// 主程序 .app 候选完整路径（搜索目录 × App 名）
fn app_dir_candidates(detect: &MacOsDetect) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in search_dirs(&detect.search_dirs) {
        for name in &detect.app_names {
            out.push(dir.join(name));
        }
    }
    out
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

pub fn detect(d: &MacOsDetect) -> PlatformResult<WeChatInfo> {
    for dir in app_dir_candidates(d) {
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

/// 正在运行的应用主进程：按 .app 目录路径索引 (pid, started_at)
fn running_map(sys: &System, d: &MacOsDetect) -> HashMap<String, (u32, u64)> {
    let mut map = HashMap::new();
    for (pid, process) in sys.processes() {
        let Some(exe) = process.exe() else {
            continue;
        };
        let exe = exe.to_string_lossy();
        if !exe.ends_with(&d.executable_suffix) {
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
pub fn launch_targets(
    p: &MacOsPlatform,
    main_alias: Option<&str>,
    default_label: &str,
) -> PlatformResult<Vec<LaunchTarget>> {
    let d = &p.detect;
    // 只刷新所需进程信息（exe 需显式指定，start_time 自动填充），避免 new_all 全量枚举的额外开销
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let running = running_map(&sys, d);
    let mut targets = Vec::new();

    let main = detect(d)?;
    if let Some(path) = &main.path {
        let (is_running, pid, started_at) = running_state(&running, path);
        targets.push(LaunchTarget {
            path: path.clone(),
            app_name: d.app_names[0].clone(),
            label: main_alias
                .filter(|s| !s.trim().is_empty())
                // 语言中立标记值，前端对主程序做 i18n 映射
                .unwrap_or(default_label)
                .to_string(),
            version: main.version.clone(),
            is_main: true,
            running: is_running,
            pid,
            started_at,
            outdated: false,
            instance_count: u32::from(is_running),
            profile_id: String::new(),
        });
    }

    // 副本识别：只按 Bundle ID（前缀 + 纯数字编号），
    // 因此任意目录名（含中文自定义名）的副本都能识别
    for dir in search_dirs(&d.search_dirs) {
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
            let Some(suffix) = bundle_id.strip_prefix(&d.bundle_id_prefix) else {
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
                profile_id: String::new(),
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

pub fn terminate(pid: u32, tc: &TerminateConfig) -> PlatformResult<()> {
    let target = Pid::from_u32(pid);
    // 复用同一个 System，只刷新目标进程，避免反复全量扫描
    let mut sys = System::new();

    // 先 SIGTERM 优雅退出，宽限期内未退出再 SIGKILL（节奏来自 profile 终止参数）
    sys.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    match sys.process(target) {
        None => return Ok(()), // 已退出
        Some(p) => match p.kill_with(Signal::Term) {
            Some(true) => {}
            _ => return Err(format!("无法向进程 {pid} 发送退出信号")),
        },
    }
    for _ in 0..tc.max_polls {
        std::thread::sleep(std::time::Duration::from_millis(tc.poll_interval_ms));
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

/// 主程序 .app 目录名的 stem 集合（用于副本命名冲突校验）
fn main_app_stems(d: &MacOsDetect) -> Vec<String> {
    d.app_names
        .iter()
        .filter_map(|n| {
            Path::new(n)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .collect()
}

/// 校验并规范化名称：去空白、去 .app 后缀、限长、禁特殊字符、禁与主程序同名
fn sanitize_name(name: &str, main_stems: &[String]) -> PlatformResult<String> {
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
    if main_stems.iter().any(|s| s.eq_ignore_ascii_case(&n)) {
        return Err("名称不能与主程序同名（会与主程序冲突）".into());
    }
    if n.chars().any(|c| matches!(c, '/' | ':')) {
        return Err("名称不能包含 / 或 :".into());
    }
    Ok(n)
}

/// 扫描全部 .app 的应用系 Bundle ID，返回下一个可用编号
/// （主程序占 1 号，副本编号从 bundleIdStartAt 起，validate 保证 ≥ 2）
fn next_bundle_number(d: &MacOsDetect, mi: &MacMultiInstance) -> PlatformResult<u32> {
    let mut max = mi.bundle_id_start_at - 1;
    for dir in search_dirs(&d.search_dirs) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("app") {
                continue;
            }
            if let Some((bid, _)) = read_bundle_info(&path) {
                if let Some(suffix) = bid.strip_prefix(&d.bundle_id_prefix) {
                    if let Ok(n) = suffix.parse::<u32>() {
                        max = max.max(n);
                    }
                }
            }
        }
    }
    Ok(max + 1)
}

/// 修改副本 Info.plist：Bundle ID / 名称（名称写入 profile 声明的展示名 key）
fn rewrite_plist(
    app: &Path,
    name: Option<&str>,
    bundle_id: Option<&str>,
    mi: &MacMultiInstance,
) -> PlatformResult<()> {
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
        for key in &mi.display_name_keys {
            dict.insert(key.clone(), plist::Value::String(name.into()));
        }
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

/// 确认目标是副本而非主程序：
/// 目录名不得为主程序 .app 名，且 Bundle ID 为「前缀 + 非空数字编号」
fn ensure_copy(app: &Path, d: &MacOsDetect) -> PlatformResult<String> {
    if let Some(name) = app.file_name().and_then(|n| n.to_str()) {
        if d.app_names.iter().any(|n| n == name) {
            return Err("不能对主程序执行此操作".into());
        }
    }
    let (bundle_id, _) =
        read_bundle_info(app).ok_or_else(|| format!("无法读取应用信息：{}", app.display()))?;
    let Some(suffix) = bundle_id.strip_prefix(&d.bundle_id_prefix) else {
        return Err("该应用不是本工具识别的微信副本".into());
    };
    if suffix.is_empty() {
        return Err("不能对主程序执行此操作".into());
    }
    Ok(bundle_id)
}

fn is_running(app: &Path, d: &MacOsDetect) -> bool {
    let path_str = app.to_string_lossy().into_owned();
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    running_map(&sys, d).contains_key(&path_str)
}

/// 按复制策略复制应用：APFS 克隆（cp -Rc）首选，失败按回退策略普通复制
fn copy_app(src: &Path, dest: &Path, copy: &CopyConfig) -> PlatformResult<()> {
    let apfs_cloned = copy.strategy == CopyStrategy::ApfsClone
        && Command::new("/bin/cp")
            .args(["-Rc"])
            .arg(src)
            .arg(dest)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    if apfs_cloned {
        return Ok(());
    }
    if copy.strategy == CopyStrategy::ApfsClone {
        if copy.fallback != CopyStrategy::RecursiveCopy {
            return Err("复制应用文件失败（权限不足或磁盘空间不够）".into());
        }
        // 非 APFS 卷，走回退策略
    } else if copy.strategy != CopyStrategy::RecursiveCopy {
        return Err("复制应用文件失败（权限不足或磁盘空间不够）".into());
    }
    let copied = Command::new("/bin/cp")
        .args(["-R"])
        .arg(src)
        .arg(dest)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if copied {
        Ok(())
    } else {
        Err("复制应用文件失败（权限不足或磁盘空间不够）".into())
    }
}

/// 按签名策略签名（当前仅 ad-hoc，签名参数由引擎固定）
fn sign_app(app: &Path, sign: SignStrategy) -> PlatformResult<()> {
    match sign {
        SignStrategy::Adhoc => {
            if std::env::var("VXMO_WALKTHROUGH_FAIL_SIGN").is_ok() {
                return Err("模拟签名失败（走查注入）".into());
            }
            codesign_adhoc(app)
        }
    }
}

/// 创建副本：复制 → 改 Info.plist → 重签名。
/// on_step 在每个大步骤完成时回调（1=复制 2=改 Bundle ID 3=重签名），
/// 供上层 emit 进度事件给前端逐步点亮。
pub fn generate_with_progress(
    name: &str,
    on_step: &dyn Fn(u32),
    p: &MacOsPlatform,
) -> PlatformResult<()> {
    let main = detect(&p.detect)?;
    let Some(src) = main.path else {
        return Err("未检测到微信主程序，无法创建副本".into());
    };
    let src = PathBuf::from(src);
    let name = sanitize_name(name, &main_app_stems(&p.detect))?;
    let bundle_id = p
        .multi_instance
        .bundle_id_pattern
        .replace("{n}", &next_bundle_number(&p.detect, &p.multi_instance)?.to_string());

    // 目标目录优先选主程序所在目录，权限不足时逐个回退
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(parent) = src.parent() {
        candidates.push(parent.to_path_buf());
    }
    for d in search_dirs(&p.detect.search_dirs) {
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
        match generate_one(&src, &dest, &name, &bundle_id, on_step, &p.multi_instance) {
            Ok(()) => return Ok(()),
            Err(e) => {
                cleanup_dir(&dest);
                last_err = e;
            }
        }
    }
    Err(format!("创建副本失败：{last_err}"))
}

fn generate_one(
    src: &Path,
    dest: &Path,
    name: &str,
    bundle_id: &str,
    on_step: &dyn Fn(u32),
    mi: &MacMultiInstance,
) -> PlatformResult<()> {
    copy_app(src, dest, &mi.copy)?;
    on_step(1);
    rewrite_plist(dest, Some(name), Some(bundle_id), mi)?;
    on_step(2);
    sign_app(dest, mi.sign)?;
    on_step(3);
    Ok(())
}

/// 重命名副本：Bundle ID 不变（数据容器与账号数据不受影响），
/// 仅改显示名 + 目录名 + 重签名
pub fn rename(path: &str, new_name: &str, p: &MacOsPlatform) -> PlatformResult<()> {
    let app = PathBuf::from(path);
    ensure_copy(&app, &p.detect)?;
    let name = sanitize_name(new_name, &main_app_stems(&p.detect))?;
    if is_running(&app, &p.detect) {
        return Err("副本正在运行，请先关闭再改名".into());
    }

    rewrite_plist(&app, Some(&name), None, &p.multi_instance)?;
    let parent = app.parent().ok_or_else(|| "路径异常".to_string())?;
    let dest = parent.join(format!("{name}.app"));
    if dest.exists() {
        return Err(format!("已存在同名应用：{}", dest.display()));
    }
    std::fs::rename(&app, &dest).map_err(|e| format!("目录重命名失败：{e}"))?;
    sign_app(&dest, p.multi_instance.sign)
}

/// 删除副本：按 profile 删除策略执行（macOS 优先废纸篓），账号数据保留在独立容器中
pub fn delete(path: &str, p: &MacOsPlatform) -> PlatformResult<()> {
    use crate::profiles::schema::DeleteStrategy;
    let app = PathBuf::from(path);
    ensure_copy(&app, &p.detect)?;
    if is_running(&app, &p.detect) {
        return Err("副本正在运行，请先关闭再删除".into());
    }
    match p.delete {
        DeleteStrategy::MoveToTrash => trash::delete(app).map_err(|e| format!("移到废纸篓失败：{e}")),
        DeleteStrategy::RemoveDir => std::fs::remove_dir_all(app).map_err(|e| format!("删除失败：{e}")),
        DeleteStrategy::Unsupported => Err("此应用不支持删除操作".into()),
    }
}

/// 重建版本落后的副本：保留原 Bundle ID 与名字。
/// 数据容器由 Bundle ID 决定，因此账号数据与登录状态完全不受影响。
///
/// 事务化流程：先在临时目录完成「复制 → 改 Info.plist → 重签名」全部
/// 失败率高的步骤（期间旧副本原地不动，任何失败都完整可用），再通过
/// 两次同卷 rename 原子换位，最后旧副本才进废纸篓。
/// 极小窗口说明：第 3-4 步之间进程被杀会留下 `{name}.app.vxmo-old`（新
/// 副本未就位），下次重建同名副本时会自动清理；不为此引入恢复扫描。
pub fn rebuild(path: &str, p: &MacOsPlatform) -> PlatformResult<()> {
    let app = PathBuf::from(path);
    let bundle_id = ensure_copy(&app, &p.detect)?;
    if is_running(&app, &p.detect) {
        return Err("副本正在运行，请先关闭再重建".into());
    }
    let main = detect(&p.detect)?;
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
    let tmp = parent.join(format!("{name}.app.vxmo-tmp"));
    let old = parent.join(format!("{name}.app.vxmo-old"));

    // 清理上次中断残留的临时目录（正常路径下不存在）
    cleanup_dir(&tmp);
    cleanup_dir(&old);

    // 1+2. 复制到临时目录并改造（Bundle ID 与显示名不变 + 重签名），
    //      失败时旧副本未受任何影响
    if let Err(e) = generate_one(&src, &tmp, &name, &bundle_id, &|_| {}, &p.multi_instance) {
        cleanup_dir(&tmp);
        return Err(format!("重建失败：{e}（旧副本未受影响）"));
    }

    // 3+4. 同卷两次 rename 原子换位；换位失败把旧副本移回原位
    match swap_dirs(&app, &tmp, &old, &dest) {
        SwapOutcome::Done => {}
        SwapOutcome::Failed(e) => {
            cleanup_dir(&tmp);
            return Err(format!("重建失败：目录替换未完成（{e}）（旧副本未受影响）"));
        }
        SwapOutcome::RolledBack(e) => {
            cleanup_dir(&tmp);
            return Err(format!(
                "重建失败：目录替换未完成（{e}）（旧副本已恢复原位，未受影响）"
            ));
        }
        SwapOutcome::RollbackFailed(e, re) => {
            cleanup_dir(&tmp);
            return Err(format!(
                "重建失败：目录替换未完成（{e}）（旧副本回滚失败：{re}，保留在 {}，可手动移回原位）",
                old.display()
            ));
        }
    }

    // 5. 换位完成，旧副本最后进废纸篓；清理失败不影响新副本可用
    if let Err(e) = trash::delete(&old) {
        if let Err(re) = std::fs::remove_dir_all(&old) {
            return Err(format!(
                "重建成功，但旧副本清理失败（废纸篓：{e}；直删：{re}），残留于 {}",
                old.display()
            ));
        }
    }
    Ok(())
}

/// 同卷目录换位结果
enum SwapOutcome {
    /// tmp 已就位 dest，旧副本停在 old
    Done,
    /// 第一步就失败，现场未动
    Failed(std::io::Error),
    /// 第二步失败，旧副本已移回原位
    RolledBack(std::io::Error),
    /// 第二步失败且回滚也失败，旧副本滞留 old
    RollbackFailed(std::io::Error, std::io::Error),
}

/// 原子换位：`app → old`、`tmp → dest`（两次同卷 rename），失败时回滚旧副本
fn swap_dirs(app: &Path, tmp: &Path, old: &Path, dest: &Path) -> SwapOutcome {
    if let Err(e) = std::fs::rename(app, old) {
        return SwapOutcome::Failed(e);
    }
    if let Err(e) = std::fs::rename(tmp, dest) {
        return match std::fs::rename(old, app) {
            Ok(()) => SwapOutcome::RolledBack(e),
            Err(re) => SwapOutcome::RollbackFailed(e, re),
        };
    }
    SwapOutcome::Done
}

impl EnginePlatform for MacOsPlatform {
    fn detect_app(&self) -> PlatformResult<WeChatInfo> {
        detect(&self.detect)
    }

    fn launch_targets(
        &self,
        main_alias: Option<&str>,
        default_label: &str,
    ) -> PlatformResult<Vec<LaunchTarget>> {
        launch_targets(self, main_alias, default_label)
    }

    fn launch(&self, path: &str) -> PlatformResult<()> {
        launch(path)
    }

    fn activate(&self, path: &str, pid: Option<u32>) -> PlatformResult<()> {
        activate(path, pid)
    }

    fn terminate(&self, pid: u32) -> PlatformResult<()> {
        terminate(pid, &self.terminate)
    }

    fn generate_with_progress(&self, name: &str, on_step: &dyn Fn(u32)) -> PlatformResult<()> {
        generate_with_progress(name, on_step, self)
    }

    fn rename(&self, path: &str, new_name: &str) -> PlatformResult<()> {
        rename(path, new_name, self)
    }

    fn delete(&self, path: &str) -> PlatformResult<()> {
        delete(path, self)
    }

    fn rebuild(&self, path: &str) -> PlatformResult<()> {
        rebuild(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 原子换位：正常路径 —— tmp 就位 dest，旧副本停在 old
    #[test]
    fn swap_dirs_success() {
        let base = std::env::temp_dir().join(format!("vxmo-swap-{}", std::process::id()));
        std::fs::create_dir_all(base.join("copy.app")).unwrap();
        std::fs::create_dir_all(base.join("copy.app.vxmo-tmp")).unwrap();
        let (app, tmp) = (base.join("copy.app"), base.join("copy.app.vxmo-tmp"));
        let (old, dest) = (base.join("copy.app.vxmo-old"), base.join("copy.app"));
        match swap_dirs(&app, &tmp, &old, &dest) {
            SwapOutcome::Done => {}
            _ => panic!("expected Done"),
        }
        assert!(dest.exists(), "新副本应就位 dest（与原 app 路径相同）");
        assert!(!base.join("copy.app.vxmo-tmp").exists());
        assert!(old.exists());
        std::fs::remove_dir_all(&base).unwrap();
    }

    /// 原子换位：第二步失败（dest 被占）—— 旧副本回滚到原位
    #[test]
    fn swap_dirs_rollback() {
        let base = std::env::temp_dir().join(format!("vxmo-swap-rb-{}", std::process::id()));
        std::fs::create_dir_all(base.join("copy.app")).unwrap();
        std::fs::create_dir_all(base.join("copy.app.vxmo-tmp")).unwrap();
        // 同卷 rename 目录到已存在的非空目录必然失败 → 模拟第二步失败
        std::fs::create_dir_all(base.join("occupied.vxmo-old-dest")).unwrap();
        std::fs::write(base.join("occupied.vxmo-old-dest/x"), "x").unwrap();
        let (app, tmp) = (base.join("copy.app"), base.join("copy.app.vxmo-tmp"));
        let (old, dest) = (
            base.join("copy.app.vxmo-old"),
            base.join("occupied.vxmo-old-dest"),
        );
        match swap_dirs(&app, &tmp, &old, &dest) {
            SwapOutcome::RolledBack(_) => {}
            _ => panic!("expected RolledBack"),
        }
        assert!(app.exists(), "旧副本应已回滚原位");
        std::fs::remove_dir_all(&base).unwrap();
    }
}
