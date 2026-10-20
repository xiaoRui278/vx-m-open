// 在 release 模式下隐藏 Windows 控制台窗口，勿删
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vx_m_open_lib::run()
}
