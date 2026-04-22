use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn setup_shortcut(app: &mut tauri::App) -> tauri::Result<()> {
    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut("Alt+Space", move |_app, _shortcut, _event| {
            crate::tray::toggle_main_window(&handle);
        })
        .map_err(|e| tauri::Error::Anyhow(e.into()))?;
    Ok(())
}
