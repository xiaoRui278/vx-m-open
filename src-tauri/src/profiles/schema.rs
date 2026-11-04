//! Profile 配置结构：按平台的强类型描述（字段对齐 profile 平台设计稿 §3.4，
//! 但 detect 两平台字段不同，故拆分为 MacOsPlatform / WindowsPlatform 两个结构）。
//!
//! 本版本只有内置 Rust 常量，无外部 JSON 文件（loader / 远程清单是 v1.3+ 与 v2.0 的事）；
//! serde derive 为 v2.0 序列化预留，命名 camelCase 与设计稿一致。

use serde::{Deserialize, Serialize};

/// 策略枚举统一从 strategy 模块引用；此处 re-export 供下游 `schema::Xxx` 直接使用
pub use crate::profiles::strategy::*;

/// 展示名（i18n 后置到前端，这里只带语言键值）
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedText {
    pub zh: String,
    pub en: String,
}

// ---------- macOS（clone 模型） ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MacOsDetect {
    pub strategy: DetectStrategy,
    /// 候选 .app 目录名（如 WeChat.app）
    pub app_names: Vec<String>,
    /// 副本 Bundle ID 前缀（副本 = 前缀 + 纯数字编号）
    pub bundle_id_prefix: String,
    /// 搜索目录（`~` 前缀由引擎展开）
    pub search_dirs: Vec<String>,
    /// 可执行文件判定后缀（如 /Contents/MacOS/WeChat）
    pub executable_suffix: String,
    pub version_strategy: VersionStrategy,
    /// Info.plist 中的版本 key
    pub version_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CopyConfig {
    pub strategy: CopyStrategy,
    /// 首选策略不可用时的回退
    pub fallback: CopyStrategy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MacMultiInstance {
    pub strategy: MultiInstanceStrategy,
    pub copy: CopyConfig,
    /// 副本 Bundle ID 生成模式，`{n}` 为编号占位（如 com.tencent.xinWeChat{n}）
    pub bundle_id_pattern: String,
    /// 副本编号起始值（主程序占 1，副本从 2 起）
    pub bundle_id_start_at: u32,
    /// Info.plist 中写入显示名的 key 列表
    pub display_name_keys: Vec<String>,
    pub sign: SignStrategy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MacOsPlatform {
    pub detect: MacOsDetect,
    pub multi_instance: MacMultiInstance,
    pub launch: LaunchStrategy,
    pub activate: ActivateStrategy,
    pub terminate: TerminateConfig,
    pub delete: DeleteStrategy,
    pub rebuild: RebuildStrategy,
}

// ---------- Windows（mutex 模型） ----------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowsDetect {
    pub strategy: DetectStrategy,
    /// 候选进程映像名（WeChat.exe 3.x / Weixin.exe 4.x）
    pub exe_names: Vec<String>,
    /// 注册表子键（HKCU 下），按顺序尝试
    pub registry_keys: Vec<String>,
    /// 安装目录所在的注册表值名
    pub install_path_value: String,
    /// 注册表未命中时的默认安装目录
    pub fallback_dirs: Vec<String>,
    pub version_strategy: VersionStrategy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowsMultiInstance {
    pub strategy: MultiInstanceStrategy,
    /// 单实例互斥体名的宽松匹配片段（全部关闭，兼容 3.x/4.x 命名差异）
    pub mutex_marks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowsPlatform {
    pub detect: WindowsDetect,
    pub multi_instance: WindowsMultiInstance,
    pub launch: LaunchStrategy,
    pub activate: ActivateStrategy,
    pub terminate: TerminateConfig,
    pub delete: DeleteStrategy,
    pub rebuild: RebuildStrategy,
}

// ---------- 共享 ----------

/// 终止参数：策略 + 优雅退出宽限 + 宽限轮询节奏
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TerminateConfig {
    pub strategy: TerminateStrategy,
    /// 优雅退出宽限毫秒数
    pub grace_ms: u64,
    /// 宽限期内检查进程是否退出的轮询间隔
    pub poll_interval_ms: u64,
    /// 宽限期内最大轮询次数
    pub max_polls: u32,
}
