use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub fn start_deno_sidecar(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let sidecar = match app.shell().sidecar("deno") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sidecar init error: {e}");
                return;
            }
        };

        let (mut rx, _child) = match sidecar.spawn() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("sidecar spawn error: {e}");
                return;
            }
        };

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    if let Ok(text) = String::from_utf8(line) {
                        let _ = app.emit("sidecar-output", text);
                    }
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("sidecar: {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Terminated(_) => break,
                _ => {}
            }
        }
    });
}

#[tauri::command]
pub async fn ping_sidecar(message: String) -> Result<String, String> {
    Ok(format!("pong: {message}"))
}
