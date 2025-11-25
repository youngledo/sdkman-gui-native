// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod api;
mod commands;
mod utils;
mod local;
mod config;
mod tray;
mod cache;

use std::sync::Arc;
use tokio::sync::Mutex;
use api::SdkmanApiClient;
use tauri::Manager;

fn main() {
    // 初始化 SDKMAN API 客户端
    let client = Arc::new(Mutex::new(
        SdkmanApiClient::new().expect("Failed to initialize SDKMAN API client")
    ));

    tauri::Builder::default()
        .manage(client)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当尝试启动第二个实例时，激活已有窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .setup(|app| {
            // 初始化系统托盘
            tray::create_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 阻止窗口关闭，隐藏窗口到托盘
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_jdk_versions,
            commands::list_sdk_candidates,
            commands::list_sdk_versions,
            commands::get_statistics,
            commands::scan_installed_sdks,
            commands::get_current_sdk_version,
            commands::list_installed_candidates,
            commands::is_sdk_installed,
            commands::download_sdk,
            commands::download_sdk_simple,
            commands::install_sdk,
            commands::uninstall_sdk,
            commands::verify_sdk_installation,
            commands::download_and_install_sdk,
            commands::set_default_sdk_version,
            commands::unset_default_sdk_version,
            commands::load_config,
            commands::save_config,
            commands::test_proxy,
            commands::get_sdkman_path,
            tray::update_tray_menu,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // macOS: 处理 Dock 图标点击事件
            // Windows/Linux: 任务栏图标点击会自动显示窗口，无需额外处理
            #[cfg(target_os = "macos")]
            {
                if let tauri::RunEvent::Reopen { has_visible_windows, .. } = event {
                    if !has_visible_windows {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        });
}
