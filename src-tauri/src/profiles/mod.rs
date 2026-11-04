//! 应用 Profile：把「某应用如何被多开」的参数从平台实现里抽出来，
//! 形成『引擎（策略实现）+ profile（参数声明）』的结构。
//!
//! 本版本为 M6 引擎抽象的内部结构：只有内置微信 profile（Rust 常量），
//! 不引入外部 profile 文件；JSON 加载、远程清单、多应用注册表是 v1.3+/v2.0 的事。

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub mod schema;
pub mod strategy;

use schema::{LocalizedText, MacOsPlatform, WindowsPlatform};

/// 一个可多开应用的完整参数声明
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppProfile {
    /// Schema 版本（当前恒 1，v2.0 开放外部 profile 时启用不兼容校验）
    pub schema_version: u32,
    /// Profile 唯一标识（如 "wechat"），LaunchTarget.profileId 使用
    pub id: String,
    pub name: LocalizedText,
    /// 平台参数声明；某平台无实现时为 None
    pub macos: Option<MacOsPlatform>,
    pub windows: Option<WindowsPlatform>,
}

/// 内置微信 profile 的参数逐项溯源：
/// - macOS 侧来自 `platform/macos.rs` 原 `MAIN_APP_NAME` / `MAIN_BUNDLE_ID` /
///   `MAIN_EXE_SUFFIX` 常量与搜索目录、Bundle 编号规则（start 2 起）、
///   SIGTERM 宽限轮询（2000ms / 150ms × 13）
/// - Windows 侧来自 `platform/windows.rs` 原 `EXE_NAMES` / `REG_SUBKEYS` /
///   `DEFAULT_DIRS` / `WECHAT_MUTEX_MARKS` 常量与 WM_QUIT 宽限（2000ms）
static WECHAT: LazyLock<AppProfile> = LazyLock::new(|| {
    let profile = AppProfile {
        schema_version: 1,
        id: "wechat".into(),
        name: LocalizedText {
            zh: "微信".into(),
            en: "WeChat".into(),
        },
        macos: Some(MacOsPlatform {
            detect: schema::MacOsDetect {
                strategy: strategy::DetectStrategy::BundleScan,
                app_names: vec!["WeChat.app".into()],
                bundle_id_prefix: "com.tencent.xinWeChat".into(),
                search_dirs: vec!["/Applications".into(), "~/Applications".into()],
                executable_suffix: "/Contents/MacOS/WeChat".into(),
                version_strategy: strategy::VersionStrategy::PlistKey,
                version_key: "CFBundleShortVersionString".into(),
            },
            multi_instance: schema::MacMultiInstance {
                strategy: strategy::MultiInstanceStrategy::Clone,
                copy: schema::CopyConfig {
                    strategy: strategy::CopyStrategy::ApfsClone,
                    fallback: strategy::CopyStrategy::RecursiveCopy,
                },
                bundle_id_pattern: "com.tencent.xinWeChat{n}".into(),
                bundle_id_start_at: 2,
                display_name_keys: vec!["CFBundleName".into(), "CFBundleDisplayName".into()],
                sign: strategy::SignStrategy::Adhoc,
            },
            launch: strategy::LaunchStrategy::OpenApp,
            activate: strategy::ActivateStrategy::OpenApp,
            terminate: schema::TerminateConfig {
                strategy: strategy::TerminateStrategy::SigtermThenKill,
                grace_ms: 2000,
                poll_interval_ms: 150,
                max_polls: 13,
            },
            delete: strategy::DeleteStrategy::MoveToTrash,
            rebuild: strategy::RebuildStrategy::CloneReplace,
        }),
        windows: Some(WindowsPlatform {
            detect: schema::WindowsDetect {
                strategy: strategy::DetectStrategy::RegistryThenPaths,
                exe_names: vec!["WeChat.exe".into(), "Weixin.exe".into()],
                registry_keys: vec![
                    "Software\\Tencent\\WeChat".into(),
                    "Software\\Tencent\\Weixin".into(),
                ],
                install_path_value: "InstallPath".into(),
                fallback_dirs: vec![
                    "C:\\Program Files\\Tencent\\WeChat".into(),
                    "C:\\Program Files\\Tencent\\Weixin".into(),
                    "C:\\Program Files (x86)\\Tencent\\WeChat".into(),
                ],
                version_strategy: strategy::VersionStrategy::FileVersion,
            },
            multi_instance: schema::WindowsMultiInstance {
                strategy: strategy::MultiInstanceStrategy::Mutex,
                mutex_marks: vec![
                    "WeChat_App_Instance".into(),
                    "Weixin_App_Instance".into(),
                    // 片段 3 已覆盖片段 1/2（3.x 实际名同时含三者）；单独列出是为了
                    // 明确表达对 4.x 改名的兼容意图，勿当作冗余删除
                    "App_Instance_Identity_Mutex".into(),
                ],
            },
            launch: strategy::LaunchStrategy::SuspendedThenCloseMutex,
            activate: strategy::ActivateStrategy::FocusWindowByPid,
            // WM_QUIT 宽限 2000ms 由 WaitForSingleObject 单次等待实现，
            // 无轮询节奏（poll 字段仅 SigtermThenKill 使用，置 0）
            terminate: schema::TerminateConfig {
                strategy: strategy::TerminateStrategy::TerminateProcess,
                grace_ms: 2000,
                poll_interval_ms: 0,
                max_polls: 0,
            },
            delete: strategy::DeleteStrategy::Unsupported,
            rebuild: strategy::RebuildStrategy::Unsupported,
        }),
    };
    validate(&profile).expect("内置微信 profile 参数非法（编程错误）");
    profile
});

/// 内置 profile 注册表。本版本只有微信；v2.0 演进为多 profile 查表。
pub fn builtin() -> &'static AppProfile {
    &WECHAT
}

/// 内部常量自检：必填字段非空、模式含编号占位。仅覆盖 profile 声明本身的
/// 完整性，不做外部输入校验（那是 v2.0 开放外部 profile 时的 schema 校验）。
fn validate(p: &AppProfile) -> Result<(), String> {
    if p.schema_version == 0 {
        return Err("schemaVersion 必须大于 0".into());
    }
    if p.id.trim().is_empty() {
        return Err("id 不能为空".into());
    }
    if p.name.zh.trim().is_empty() || p.name.en.trim().is_empty() {
        return Err("name 中英文均不能为空".into());
    }
    if let Some(m) = &p.macos {
        let d = &m.detect;
        if d.app_names.is_empty() || d.search_dirs.is_empty() {
            return Err("macOS detect：appNames / searchDirs 不能为空".into());
        }
        if d.bundle_id_prefix.trim().is_empty()
            || d.executable_suffix.trim().is_empty()
            || d.version_key.trim().is_empty()
        {
            return Err("macOS detect：bundleIdPrefix / executableSuffix / versionKey 不能为空".into());
        }
        let mi = &m.multi_instance;
        if !mi.bundle_id_pattern.contains("{n}") {
            return Err("macOS multiInstance：bundleIdPattern 必须含 {n} 占位".into());
        }
        if mi.bundle_id_start_at < 2 {
            return Err("macOS multiInstance：bundleIdStartAt 必须 ≥ 2（主程序占 1）".into());
        }
        if mi.display_name_keys.is_empty() {
            return Err("macOS multiInstance：displayNameKeys 不能为空".into());
        }
        if m.terminate.strategy == strategy::TerminateStrategy::SigtermThenKill
            && (m.terminate.grace_ms == 0 || m.terminate.poll_interval_ms == 0 || m.terminate.max_polls == 0)
        {
            return Err("macOS terminate：SigtermThenKill 需要正数的 grace/poll 参数".into());
        }
    }
    if let Some(w) = &p.windows {
        let d = &w.detect;
        if d.exe_names.is_empty() || d.registry_keys.is_empty() || d.fallback_dirs.is_empty() {
            return Err("Windows detect：exeNames / registryKeys / fallbackDirs 不能为空".into());
        }
        if d.install_path_value.trim().is_empty() {
            return Err("Windows detect：installPathValue 不能为空".into());
        }
        if w.multi_instance.mutex_marks.is_empty() {
            return Err("Windows multiInstance：mutexMarks 不能为空".into());
        }
        if w.terminate.strategy == strategy::TerminateStrategy::TerminateProcess
            && w.terminate.grace_ms == 0
        {
            return Err("Windows terminate：TerminateProcess 需要正数的 graceMs".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内置微信 profile 必须通过自检且关键参数与溯源值一致
    #[test]
    fn builtin_wechat_valid() {
        let p = builtin();
        assert_eq!(p.id, "wechat");
        assert_eq!(p.schema_version, 1);
        let m = p.macos.as_ref().unwrap();
        assert_eq!(m.detect.bundle_id_prefix, "com.tencent.xinWeChat");
        assert_eq!(m.multi_instance.bundle_id_start_at, 2);
        assert_eq!(m.terminate.grace_ms, 2000);
        assert_eq!(m.terminate.max_polls, 13);
        let w = p.windows.as_ref().unwrap();
        assert_eq!(w.multi_instance.mutex_marks.len(), 3);
        assert_eq!(w.detect.fallback_dirs.len(), 3);
        validate(p).unwrap();
    }
}
