//! 本地小配置（如主程序备注名），存于应用配置目录的 aliases.json。
//! 注意：这只影响本工具内的显示，不触碰微信本体。

use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Aliases {
    #[serde(skip_serializing_if = "Option::is_none")]
    main: Option<String>,
}

fn config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败：{e}"))?;
    Ok(dir.join("aliases.json"))
}

pub fn main_alias(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let file = config_file(app)?;
    let Ok(text) = fs::read_to_string(&file) else {
        return Ok(None);
    };
    let aliases: Aliases = serde_json::from_str(&text).unwrap_or_default();
    // 兼容旧版本可能已写入的 "WeChat" 备注：读取侧一并过滤（写入侧已拒绝该值）
    Ok(aliases
        .main
        .filter(|s| !s.trim().is_empty() && !s.trim().eq_ignore_ascii_case("WeChat")))
}

pub fn set_main_alias(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    // "WeChat" 是后端「无备注」的标记值（前端按此映射 i18n 显示名），
    // 用户设置同名备注会与标记值混淆，故拒绝（大小写不敏感）
    if trimmed.eq_ignore_ascii_case("WeChat") {
        return Err("备注名不能为 WeChat，会与主程序默认显示名冲突".into());
    }
    let file = config_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{e}"))?;
    }
    let aliases = Aliases {
        main: Some(trimmed.to_string()),
    };
    let json = serde_json::to_string(&aliases).map_err(|e| e.to_string())?;
    // 原子写：先写临时文件再 rename，进程中断不会留下损坏的 JSON
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| format!("写入配置失败：{e}"))?;
    fs::rename(&tmp, &file).map_err(|e| format!("写入配置失败：{e}"))
}
