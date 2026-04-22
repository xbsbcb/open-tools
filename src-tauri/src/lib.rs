mod commands;
mod db;
mod shortcut;
mod sidecar;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:open-tools.db", db::migrations())
                .build(),
        )
        .setup(|app| {
            tray::setup_tray(app)?;
            shortcut::setup_shortcut(app)?;
            sidecar::start_deno_sidecar(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search_apps,
            commands::open::open_path,
            commands::calc::eval_expr,
            sidecar::ping_sidecar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
