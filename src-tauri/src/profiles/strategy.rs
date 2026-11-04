//! 策略枚举：多开机制的引擎内置能力词（profile 只能引用，不能携带可执行内容）。
//!
//! 枚举值与 profile 平台设计稿 §3.3 对齐；本版本仅供内置微信 profile 使用，
//! serde 命名为 camelCase 以便 v2.0 直接序列化为 JSON Schema 形态。

use serde::{Deserialize, Serialize};

/// 检测策略：如何在系统中定位应用
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DetectStrategy {
    /// macOS：扫描候选目录下的 .app
    BundleScan,
    /// Windows：先注册表 InstallPath，后默认目录
    RegistryThenPaths,
}

/// 版本来源策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VersionStrategy {
    /// macOS：读 Info.plist 指定 key
    PlistKey,
    /// Windows：读 exe 文件版本资源
    FileVersion,
    /// 预留：目录名版本优先、文件版本兜底（当前无使用方，v2.0 多应用启用）
    FolderVersionThenFileVersion,
}

/// 多开策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MultiInstanceStrategy {
    /// macOS 副本模型：克隆 App + 改 Bundle ID + 重签名
    Clone,
    /// Windows 互斥体模型：挂起启动 + 关闭单实例互斥体
    Mutex,
}

/// 复制策略（仅 macOS clone 模型）
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CopyStrategy {
    /// APFS clonefile（同卷秒级零拷贝）
    ApfsClone,
    /// 普通递归复制（APFS 不可用时的回退）
    RecursiveCopy,
}

/// 重签名策略（仅 macOS clone 模型；签名参数由引擎固定，profile 不可传）
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SignStrategy {
    /// ad-hoc 签名（codesign --sign -）
    Adhoc,
}

/// 启动策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LaunchStrategy {
    /// macOS：`open <path>` / `open -a`
    OpenApp,
    /// Windows：挂起启动 + 恢复后关闭单实例互斥体
    SuspendedThenCloseMutex,
}

/// 激活策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActivateStrategy {
    /// macOS：按路径 `open -a`
    OpenApp,
    /// Windows：按 pid 枚举窗口并前置
    FocusWindowByPid,
}

/// 终止策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TerminateStrategy {
    /// macOS：SIGTERM 优雅退出，宽限期内未退再 SIGKILL
    SigtermThenKill,
    /// Windows：WM_QUIT 优雅退出，宽限期内未退再 TerminateProcess
    TerminateProcess,
}

/// 删除策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeleteStrategy {
    /// macOS：移到废纸篓（可反悔）
    MoveToTrash,
    /// 预留：直接删除（当前无使用方，v2.0 多应用启用）
    RemoveDir,
    /// 应用不支持删除操作
    Unsupported,
}

/// 重建策略
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RebuildStrategy {
    /// macOS clone 模型：保留 Bundle ID 与显示名，克隆新版主程序替换
    CloneReplace,
    /// 应用不支持重建（如 Windows 互斥体模型无副本）
    Unsupported,
}
