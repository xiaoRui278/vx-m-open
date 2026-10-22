mod commands;
mod config;
mod platform;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::detect_wechat,
            commands::list_launch_targets,
            commands::launch_instance,
            commands::activate_instance,
            commands::terminate_instance,
            commands::generate_copy,
            commands::rename_copy,
            commands::delete_copy,
            commands::rebuild_copy,
            commands::rebuild_all,
            commands::set_main_alias,
            commands::list_instance_remarks,
            commands::set_instance_remark,
            commands::open_url,
        ])
        .setup(|app| {
            // Windows 自绘 Fluent 标题栏：去掉系统边框，拖拽/窗口控制由前端实现
            if cfg!(windows) {
                use tauri::Manager;
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_decorations(false);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
