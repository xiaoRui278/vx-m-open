//! 未支持平台的占位实现，接入 Windows 后移除本文件。

use super::{LaunchTarget, WeChatInfo};

type PlatformResult<T> = Result<T, String>;

const UNSUPPORTED: &str = "当前平台尚未支持，Windows 支持计划在后续版本提供";

pub fn detect() -> PlatformResult<WeChatInfo> {
    Err(UNSUPPORTED.into())
}

pub fn launch_targets(_main_alias: Option<&str>) -> PlatformResult<Vec<LaunchTarget>> {
    Err(UNSUPPORTED.into())
}

pub fn launch(_path: &str) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn activate(_path: &str, _pid: Option<u32>) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn terminate(_pid: u32) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn generate_with_progress(_name: &str, _on_step: &dyn Fn(u32)) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn rename(_path: &str, _new_name: &str) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn delete(_path: &str) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}

pub fn rebuild(_path: &str) -> PlatformResult<()> {
    Err(UNSUPPORTED.into())
}
