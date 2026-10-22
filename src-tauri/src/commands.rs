//! Tauri IPC 命令层：只做参数转发，业务逻辑在 platform 内实现。
//!
//! 注意：Tauri v2 的同步 command 在主线程执行，耗时操作（克隆/签名/进程
//! 扫描等）必须用 `async fn` + `spawn_blocking`，否则会冻结整个窗口 UI。

use crate::config;
use crate::platform::{provider, LaunchTarget, WeChatInfo};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Emitter};

async fn run_blocking<T>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String>
where
    T: Send + 'static,
{
    match spawn_blocking(f).await {
        Ok(res) => res,
        Err(e) => Err(format!("任务执行失败：{e}")),
    }
}

#[tauri::command]
pub async fn detect_wechat() -> Result<WeChatInfo, String> {
    run_blocking(provider::detect).await
}

#[tauri::command]
pub async fn list_launch_targets(app: AppHandle) -> Result<Vec<LaunchTarget>, String> {
    let alias = config::main_alias(&app).ok().flatten();
    run_blocking(move || provider::launch_targets(alias.as_deref())).await
}

#[tauri::command]
pub async fn launch_instance(path: String) -> Result<(), String> {
    run_blocking(move || provider::launch(&path)).await
}

#[tauri::command]
pub async fn activate_instance(path: String, pid: Option<u32>) -> Result<(), String> {
    run_blocking(move || provider::activate(&path, pid)).await
}

#[tauri::command]
pub async fn terminate_instance(pid: u32) -> Result<(), String> {
    run_blocking(move || provider::terminate(pid)).await
}

#[tauri::command]
pub async fn generate_copy(app: AppHandle, name: String) -> Result<(), String> {
    // 分步进度：克隆/改 Bundle ID/重签名 各完成一步即 emit，
    // 前端监听 copy-progress 事件逐步点亮步骤与推进进度条
    run_blocking(move || {
        provider::generate_with_progress(&name, &|step| {
            let _ = app.emit("copy-progress", serde_json::json!({ "step": step, "name": name }));
        })
    })
    .await
}

/// 顺序重建所有版本落后的副本，逐个 emit rebuild-all-progress
/// （payload: { done, total, label }），单个失败不中断整体。
#[tauri::command]
pub async fn rebuild_all(app: AppHandle) -> Result<u32, String> {
    run_blocking(move || {
        let alias = config::main_alias(&app).ok().flatten();
        let targets: Vec<LaunchTarget> = provider::launch_targets(alias.as_deref())?;
        let outdated: Vec<LaunchTarget> = targets
            .into_iter()
            .filter(|t| !t.is_main && t.outdated && !t.running)
            .collect();
        let total = outdated.len() as u32;
        if total == 0 {
            return Ok(0);
        }
        let mut failures: Vec<String> = Vec::new();
        for (i, t) in outdated.iter().enumerate() {
            if let Err(e) = provider::rebuild(&t.path) {
                failures.push(format!("「{}」：{e}", t.label));
            }
            let _ = app.emit(
                "rebuild-all-progress",
                serde_json::json!({
                    "done": i + 1,
                    "total": total,
                    "label": t.label,
                }),
            );
        }
        if failures.is_empty() {
            Ok(total)
        } else {
            Err(format!(
                "{}/{} 个副本重建失败 —— {}",
                failures.len(),
                total,
                failures.join("；")
            ))
        }
    })
    .await
}

/// 用系统默认浏览器打开外部链接（关于页 GitHub 地址 / 更新下载页）
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("仅允许打开 https 链接".into());
    }
    run_blocking(move || {
        #[cfg(target_os = "macos")]
        let mut cmd = {
            let mut c = std::process::Command::new("/usr/bin/open");
            c.arg(&url);
            c
        };
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", "start", "", &url]);
            c
        };
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let mut cmd = {
            let mut c = std::process::Command::new("xdg-open");
            c.arg(&url);
            c
        };
        let status = cmd.status().map_err(|e| format!("打开链接失败：{e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("打开链接失败".into())
        }
    })
    .await
}

#[tauri::command]
pub async fn rename_copy(path: String, new_name: String) -> Result<(), String> {
    run_blocking(move || provider::rename(&path, &new_name)).await
}

#[tauri::command]
pub async fn delete_copy(path: String) -> Result<(), String> {
    run_blocking(move || provider::delete(&path)).await
}

#[tauri::command]
pub async fn rebuild_copy(path: String) -> Result<(), String> {
    run_blocking(move || provider::rebuild(&path)).await
}

#[tauri::command]
pub async fn set_main_alias(app: AppHandle, name: String) -> Result<(), String> {
    run_blocking(move || config::set_main_alias(&app, &name)).await
}

#[tauri::command]
pub async fn list_instance_remarks(
    app: AppHandle,
) -> Result<Vec<config::InstanceRemark>, String> {
    run_blocking(move || config::list_instance_remarks(&app)).await
}

#[tauri::command]
pub async fn set_instance_remark(
    app: AppHandle,
    pid: u32,
    started_at: u64,
    remark: String,
) -> Result<(), String> {
    run_blocking(move || config::set_instance_remark(&app, pid, started_at, &remark)).await
}
