use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, IsMenuItem},
    Manager, Runtime, AppHandle,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
}

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    // 创建空托盘，菜单将由前端初始化
    let _ = TrayIconBuilder::with_id("main-tray")
        .tooltip("SDKMAN GUI")
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            // 处理菜单事件
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[tauri::command]
pub fn update_tray_menu(app: AppHandle, items: Vec<TrayMenuItem>) -> Result<(), String> {
    // 获取托盘图标
    if let Some(tray) = app.tray_by_id("main-tray") {
        // 根据前端传入的配置创建菜单项
        let menu_items: Result<Vec<_>, _> = items
            .iter()
            .map(|item| MenuItem::with_id(&app, &item.id, &item.label, true, None::<&str>))
            .collect();

        let menu_items = menu_items.map_err(|e| e.to_string())?;

        // 创建 trait object 引用
        let menu_item_refs: Vec<&dyn IsMenuItem<_>> = menu_items
            .iter()
            .map(|item| item as &dyn IsMenuItem<_>)
            .collect();

        // 创建新菜单
        let menu = Menu::with_items(&app, &menu_item_refs[..])
            .map_err(|e| e.to_string())?;

        // 更新托盘菜单
        tray.set_menu(Some(menu))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
