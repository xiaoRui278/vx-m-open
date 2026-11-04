//! 引擎入口：持有内置 profile，按当前平台分派给策略实现。
//!
//! 对上（commands 层）暴露与历史 `provider::*` 相同签名的函数集，
//! 调用面在 M6 引擎抽象中保持不变；profile 中未声明当前平台时报错。

use super::{LaunchTarget, PlatformResult, WeChatInfo};
use crate::profiles::{self, AppProfile};

/// 平台策略接口：由 macos.rs / windows.rs 针对自己的 profile 结构实现
pub trait EnginePlatform {
    fn detect_app(&self) -> PlatformResult<WeChatInfo>;
    /// `default_label` 为主程序无备注时的展示名（profile 默认名，前端 i18n 映射）
    fn launch_targets(
        &self,
        main_alias: Option<&str>,
        default_label: &str,
    ) -> PlatformResult<Vec<LaunchTarget>>;
    fn launch(&self, path: &str) -> PlatformResult<()>;
    fn activate(&self, path: &str, pid: Option<u32>) -> PlatformResult<()>;
    fn terminate(&self, pid: u32) -> PlatformResult<()>;
    fn generate_with_progress(&self, name: &str, on_step: &dyn Fn(u32)) -> PlatformResult<()>;
    fn rename(&self, path: &str, new_name: &str) -> PlatformResult<()>;
    fn delete(&self, path: &str) -> PlatformResult<()>;
    fn rebuild(&self, path: &str) -> PlatformResult<()>;
}

/// 取当前平台的 profile 声明；profile 未声明该平台时统一报错
/// （同时兜底非 macOS/Windows 平台，取代原 unsupported 占位模块）
#[cfg(target_os = "macos")]
fn require(p: &AppProfile) -> Result<&dyn EnginePlatform, String> {
    p.macos
        .as_ref()
        .map(|m| m as &dyn EnginePlatform)
        .ok_or_else(|| "profile 未声明 macOS 平台参数".into())
}

#[cfg(target_os = "windows")]
fn require(p: &AppProfile) -> Result<&dyn EnginePlatform, String> {
    p.windows
        .as_ref()
        .map(|w| w as &dyn EnginePlatform)
        .ok_or_else(|| "profile 未声明 Windows 平台参数".into())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn require(_p: &AppProfile) -> Result<&'static dyn EnginePlatform, String> {
    Err("当前平台尚未支持".into())
}

pub fn detect() -> PlatformResult<WeChatInfo> {
    require(profiles::builtin())?.detect_app()
}

pub fn launch_targets(main_alias: Option<&str>) -> PlatformResult<Vec<LaunchTarget>> {
    let p = profiles::builtin();
    let mut targets = require(p)?.launch_targets(main_alias, &p.name.en)?;
    for t in &mut targets {
        t.profile_id = p.id.clone();
    }
    Ok(targets)
}

pub fn launch(path: &str) -> PlatformResult<()> {
    require(profiles::builtin())?.launch(path)
}

pub fn activate(path: &str, pid: Option<u32>) -> PlatformResult<()> {
    require(profiles::builtin())?.activate(path, pid)
}

pub fn terminate(pid: u32) -> PlatformResult<()> {
    require(profiles::builtin())?.terminate(pid)
}

pub fn generate_with_progress(name: &str, on_step: &dyn Fn(u32)) -> PlatformResult<()> {
    require(profiles::builtin())?.generate_with_progress(name, on_step)
}

pub fn rename(path: &str, new_name: &str) -> PlatformResult<()> {
    require(profiles::builtin())?.rename(path, new_name)
}

pub fn delete(path: &str) -> PlatformResult<()> {
    require(profiles::builtin())?.delete(path)
}

pub fn rebuild(path: &str) -> PlatformResult<()> {
    require(profiles::builtin())?.rebuild(path)
}
