//! 实机走查（真机环境依赖，不入 CI）：
//! `cargo test --lib walkthrough -- --ignored --nocapture --test-threads=1`
//!
//! 覆盖：检测 → 列目标 → 创建副本 → 改名 → 事务化重建 → 启动/激活/关闭 → 删除。
//! 走查用副本名固定前缀「走查测试号」，开始与结束时均做残留清理。
#![cfg(target_os = "macos")]

use crate::platform::provider;
use std::path::Path;
use std::time::{Duration, Instant};

const NAME: &str = "走查测试号A";
const NAME_RENAMED: &str = "走查测试号A2";

fn read_bundle_id(app: &Path) -> String {
    let value =
        plist::Value::from_file(app.join("Contents/Info.plist")).expect("读 Info.plist");
    value
        .as_dictionary()
        .and_then(|d| d.get("CFBundleIdentifier"))
        .and_then(|v| v.as_string())
        .expect("读 CFBundleIdentifier")
        .to_string()
}

/// 删除走查残留副本（正在运行会被拒绝，忽略即可）
fn cleanup(names: &[&str]) {
    for name in names {
        for base in ["/Applications", "~/Applications"] {
            let base = if let Some(rest) = base.strip_prefix("~/") {
                match std::env::var("HOME") {
                    Ok(h) => Path::new(&h).join(rest),
                    Err(_) => continue,
                }
            } else {
                Path::new(base).to_path_buf()
            };
            let app = base.join(format!("{name}.app"));
            if app.exists() {
                let _ = provider::delete(&app.to_string_lossy());
            }
        }
    }
}

fn find_copy(label: &str) -> Option<crate::platform::LaunchTarget> {
    provider::launch_targets(None)
        .unwrap()
        .into_iter()
        .find(|t| t.label == label && !t.is_main)
}

/// 轮询等待条件成立（带超时秒数）
fn wait_for(mut cond: impl FnMut() -> bool, timeout_secs: u64, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        if cond() {
            return;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    panic!("超时：{what}");
}

#[test]
#[ignore]
fn walkthrough_full_flow() {
    cleanup(&[NAME, NAME_RENAMED]);

    // 1. 检测主程序
    let info = provider::detect().expect("detect 不应报错");
    assert!(info.installed, "本机未检测到微信主程序，走查无法进行");
    println!(
        "[1] detect ✓ 主程序 {} @ {}",
        info.version.as_deref().unwrap_or("?"),
        info.path.as_deref().unwrap_or("?")
    );

    // 2. 列目标：主程序在列
    let targets = provider::launch_targets(None).expect("launch_targets 不应报错");
    assert!(targets.iter().any(|t| t.is_main), "主程序应在目标列表");
    println!("[2] launch_targets ✓ 共 {} 个目标", targets.len());

    // 3. 创建副本（三步进度回调）
    let steps = std::cell::RefCell::new(Vec::new());
    provider::generate_with_progress(NAME, &|s| steps.borrow_mut().push(s))
        .expect("创建副本不应报错");
    assert_eq!(steps.into_inner(), vec![1, 2, 3], "进度回调应依次为 1/2/3");
    let copy = find_copy(NAME).expect("创建后副本应出现在目标列表");
    assert!(!copy.running && !copy.outdated);
    assert_eq!(copy.profile_id, "wechat", "引擎层应给目标打 profileId 标");
    let bundle_before = read_bundle_id(Path::new(&copy.path));
    println!("[3] 创建副本 ✓ {}（{}）", copy.path, bundle_before);

    // 4. 改名
    provider::rename(&copy.path, NAME_RENAMED).expect("改名不应报错");
    let renamed = find_copy(NAME_RENAMED).expect("改名后应按新名出现");
    assert!(find_copy(NAME).is_none(), "旧名应消失");
    assert_eq!(
        read_bundle_id(Path::new(&renamed.path)),
        bundle_before,
        "改名不改 Bundle ID（账号容器不变）"
    );
    println!("[4] 改名 ✓ → {}（Bundle ID 保持 {}）", renamed.path, bundle_before);

    // 5. 事务化重建：版本应同步到主程序，Bundle ID 不变
    provider::rebuild(&renamed.path).expect("重建不应报错");
    let rebuilt = find_copy(NAME_RENAMED).expect("重建后副本应仍在列表");
    assert_eq!(
        read_bundle_id(Path::new(&rebuilt.path)),
        bundle_before,
        "重建保留 Bundle ID"
    );
    assert_eq!(rebuilt.version, info.version, "重建后版本应与主程序一致");
    assert!(!rebuilt.outdated);
    // 残留临时目录应被清理
    for suffix in [".vxmo-tmp", ".vxmo-old"] {
        let leftover = renamed
            .path
            .trim_end_matches(".app")
            .to_string()
            + ".app"
            + suffix;
        assert!(!Path::new(&leftover).exists(), "不应残留 {leftover}");
    }
    println!("[5] 事务化重建 ✓ 版本 {} Bundle ID 保持", rebuilt.version.unwrap_or_default());

    // 6. 启动 → 激活 → 关闭（屏幕上会短暂出现微信窗口）
    provider::launch(&rebuilt.path).expect("启动不应报错");
    wait_for(
        || find_copy(NAME_RENAMED).is_some_and(|t| t.running && t.pid.is_some()),
        20,
        "启动后应检测到运行态与 pid",
    );
    let running = find_copy(NAME_RENAMED).unwrap();
    let pid = running.pid.unwrap();
    println!("[6a] 启动 ✓ pid={pid} started_at={}", running.started_at.unwrap_or(0));

    provider::activate(&rebuilt.path, Some(pid)).expect("激活不应报错");
    println!("[6b] 激活 ✓");

    provider::terminate(pid).expect("关闭不应报错");
    wait_for(
        || find_copy(NAME_RENAMED).is_some_and(|t| !t.running),
        15,
        "关闭后应回到停止态",
    );
    println!("[6c] 关闭 ✓");

    // 7. 删除（进废纸篓）
    provider::delete(&rebuilt.path).expect("删除不应报错");
    assert!(find_copy(NAME_RENAMED).is_none(), "删除后应从列表消失");
    println!("[7] 删除 ✓（已进废纸篓）");

    cleanup(&[NAME, NAME_RENAMED]);
    println!("—— 走查全部通过 ——");
}

/// 事务化重建失败路径（sign_app 支持 VXMO_WALKTHROUGH_FAIL_SIGN 注入，平时 #[ignore]）：
/// 复制/改 plist 成功、签名必失败 → 重建应报错且旧副本完好
#[test]
#[ignore]
fn walkthrough_rebuild_sign_failure() {
    cleanup(&[NAME]);
    provider::generate_with_progress(NAME, &|_| {}).expect("创建副本不应报错");
    let copy = find_copy(NAME).expect("副本应在列表");
    let bundle_before = read_bundle_id(Path::new(&copy.path));

    // 注入：签名必失败（仅影响本测试进程）
    std::env::set_var("VXMO_WALKTHROUGH_FAIL_SIGN", "1");
    let err = provider::rebuild(&copy.path).expect_err("注入签名失败后重建应报错");
    std::env::remove_var("VXMO_WALKTHROUGH_FAIL_SIGN");
    println!("[注入] 重建报错：{err}");
    assert!(err.contains("旧副本未受影响"), "错误应说明旧副本完好");

    // 旧副本完好：仍在列表、Bundle ID 不变
    let after = find_copy(NAME).expect("旧副本应仍在列表");
    assert_eq!(read_bundle_id(Path::new(&after.path)), bundle_before);
    // 清理
    provider::delete(&after.path).expect("清理删除不应报错");
    assert!(find_copy(NAME).is_none());
    println!("[注入] 旧副本完好性验证 ✓ 已清理");
}
