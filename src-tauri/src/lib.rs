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
            commands::set_main_alias,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
