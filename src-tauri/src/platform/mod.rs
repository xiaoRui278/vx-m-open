//! 平台抽象层。
//!
//! macOS 与 Windows 的多开原理不同（macOS 靠副本换 Bundle ID，Windows 靠
//! 处理单实例互斥体），commands 层只面向同一组函数签名，接入 Windows
//! 时新增 `windows` 模块并替换 cfg 分发即可。

use serde::Serialize;

#[cfg(target_os = "macos")]
pub use macos as provider;

#[cfg(target_os = "windows")]
pub use windows as provider;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use unsupported as provider;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod unsupported;

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
    /// 展示名：主程序用备注名（无则"主程序"），副本用目录名
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
}
