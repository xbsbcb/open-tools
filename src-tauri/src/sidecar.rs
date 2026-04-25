use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

pub struct SidecarState {
    pub child: Mutex<Option<CommandChild>>,
}

pub fn start_deno_sidecar(app: AppHandle) {
    let sidecar = match app.shell().sidecar("deno") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sidecar init error: {e}");
            return;
        }
    };

    let (mut rx, child) = match sidecar.spawn() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("sidecar spawn error: {e}");
            return;
        }
    };

    // Store child for later stdin writes
    let state = app.state::<SidecarState>();
    *state.child.lock().unwrap() = Some(child);

    let _ = app.emit("sidecar-ready", "");

    tauri::async_runtime::spawn(async move {
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
pub async fn send_to_sidecar(
    state: tauri::State<'_, SidecarState>,
    payload: String,
) -> Result<String, String> {
    let mut guard = state.child.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some(child) => {
            child
                .write((payload + "\n").as_bytes())
                .map_err(|e| format!("write error: {e}"))?;
            Ok("ack".to_string())
        }
        None => Err("sidecar not running".to_string()),
    }
}

#[tauri::command]
pub async fn ping_sidecar(
    state: tauri::State<'_, SidecarState>,
    message: String,
) -> Result<String, String> {
    let payload = serde_json::json!({
        "id": "ping",
        "type": "ping",
        "data": message
    })
    .to_string();

    let mut guard = state.child.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some(child) => {
            child
                .write((payload + "\n").as_bytes())
                .map_err(|e| format!("write error: {e}"))?;
            Ok("ack".to_string())
        }
        None => Err("sidecar not running".to_string()),
    }
}
