//! 本地小配置（如主程序备注名），存于应用配置目录的 aliases.json。
//! 注意：这只影响本工具内的显示，不触碰微信本体。

use std::{fs, path::PathBuf};
use tauri::Manager;

/// Windows 运行实例的临时备注（按 pid + 启动时间匹配，进程关闭即失效）
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRemark {
    pub pid: u32,
    pub started_at: u64,
    pub remark: String,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Aliases {
    #[serde(skip_serializing_if = "Option::is_none")]
    main: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    instances: Vec<InstanceRemark>,
}

fn config_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败：{e}"))?;
    Ok(dir.join("aliases.json"))
}

/// 读取整个别名配置；文件不存在或解析失败返回默认值（不影响调用方继续写入）
fn read_aliases(app: &tauri::AppHandle) -> Aliases {
    let Ok(file) = config_file(app) else {
        return Aliases::default();
    };
    let Ok(text) = fs::read_to_string(&file) else {
        return Aliases::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// 原子写：先写临时文件再 rename，进程中断不会留下损坏的 JSON
fn write_aliases(app: &tauri::AppHandle, aliases: &Aliases) -> Result<(), String> {
    let file = config_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败：{e}"))?;
    }
    let json = serde_json::to_string(aliases).map_err(|e| e.to_string())?;
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| format!("写入配置失败：{e}"))?;
    if let Err(e) = fs::rename(&tmp, &file) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("提交配置失败：{e}"));
    }
    Ok(())
}

pub fn main_alias(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let aliases = read_aliases(app);
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
    // 保留现有的实例备注，避免整体覆盖丢失
    let mut aliases = read_aliases(app);
    aliases.main = Some(trimmed.to_string());
    write_aliases(app, &aliases)
}

/// 列出所有 Windows 实例备注（可能包含已退出进程的残留项，由前端按 pid+时间匹配过滤）
pub fn list_instance_remarks(app: &tauri::AppHandle) -> Result<Vec<InstanceRemark>, String> {
    Ok(read_aliases(app).instances)
}

/// 设置单个实例备注：同一 pid+启动时间视为同一实例，upsert；空备注 = 删除该条
pub fn set_instance_remark(
    app: &tauri::AppHandle,
    pid: u32,
    started_at: u64,
    remark: &str,
) -> Result<(), String> {
    let trimmed = remark.trim();
    let mut aliases = read_aliases(app);
    aliases
        .instances
        .retain(|r| !(r.pid == pid && r.started_at == started_at));
    if !trimmed.is_empty() {
        aliases.instances.push(InstanceRemark {
            pid,
            started_at,
            remark: trimmed.to_string(),
        });
    }
    // 上限 50 条，超出丢最旧的，防止无限增长
    if aliases.instances.len() > 50 {
        let drop = aliases.instances.len() - 50;
        aliases.instances.drain(0..drop);
    }
    write_aliases(app, &aliases)
}
