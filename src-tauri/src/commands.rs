//! Tauri IPC 命令层：只做参数转发，业务逻辑在 platform 内实现。
//!
//! 注意：Tauri v2 的同步 command 在主线程执行，耗时操作（克隆/签名/进程
//! 扫描等）必须用 `async fn` + `spawn_blocking`，否则会冻结整个窗口 UI。

use crate::config;
use crate::platform::{provider, LaunchTarget, WeChatInfo};
use tauri::async_runtime::spawn_blocking;
use tauri::AppHandle;

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
pub async fn generate_copy(name: String) -> Result<(), String> {
    run_blocking(move || provider::generate(&name)).await
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
