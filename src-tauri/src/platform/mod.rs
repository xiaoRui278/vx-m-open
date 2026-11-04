//! 平台抽象层。
//!
//! 结构为『引擎（engine）+ profile（参数）+ 平台策略实现（macos/windows）』：
//! `engine.rs` 持有内置 profile，按当前平台分派给策略实现；平台实现只含
//! 机制逻辑，所有应用专属参数（App 名、Bundle ID、互斥体片段、注册表键等）
//! 来自 `crate::profiles` 的声明。

use serde::Serialize;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod engine;

/// 平台实现统一错误别名
pub type PlatformResult<T> = Result<T, String>;

/// 引擎入口即 provider：commands 层面向 `provider::*` 调用，不感知 profile 细节
pub use engine as provider;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WeChatInfo {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub bundle_id: Option<String>,
}

/// 一个可管理的目标：主程序或副本（含运行状态）
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTarget {
    /// .app 完整路径
    pub path: String,
    /// .app 目录名（如 WeChat.app）
    pub app_name: String,
    /// 展示名：主程序用备注名（无则用 profile 默认名，前端 i18n 映射），副本用目录名
    pub label: String,
    pub version: Option<String>,
    pub is_main: bool,
    pub running: bool,
    pub pid: Option<u32>,
    /// 进程启动时间（Unix 秒）
    pub started_at: Option<u64>,
    /// 副本版本落后于主程序（微信升级后）
    pub outdated: bool,
    /// 运行实例数（Windows 主程序多开时 > 1；macOS 恒为 0 或 1）
    pub instance_count: u32,
    /// 所属应用 profile（微信恒 "wechat"；UI 暂不展示，v1.3+ 供分组/路由）
    pub profile_id: String,
}
