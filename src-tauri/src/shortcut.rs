use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn setup_shortcut(app: &mut tauri::App) -> tauri::Result<()> {
    let handle = app.handle().clone();
    app.global_shortcut().on_shortcut("Alt+Space", move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            crate::tray::toggle_main_window(&handle);
        }
    })?;
    Ok(())
}
