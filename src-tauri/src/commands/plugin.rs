use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginMeta {
    pub name: String,
    pub repo: String,
    pub description: String,
    pub author: String,
    pub stars: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

/// Call GitHub Topics API to discover plugins tagged `open-tools-plugin`.
#[tauri::command]
pub async fn discover_plugins() -> Result<Vec<PluginMeta>, String> {
    let client = reqwest::Client::new();
    let url = "https://api.github.com/search/repositories?q=topic:open-tools-plugin&sort=stars&order=desc&per_page=30";
    let response = client
        .get(url)
        .header("User-Agent", "open-tools")
        .send()
        .await
        .map_err(|e| format!("GitHub API error: {e}"))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let items = json["items"]
        .as_array()
        .ok_or_else(|| "Unexpected GitHub API response".to_string())?;

    let plugins: Vec<PluginMeta> = items
        .iter()
        .filter_map(|item| {
            Some(PluginMeta {
                name: item["name"].as_str()?.to_string(),
                repo: item["html_url"].as_str()?.to_string(),
                description: item["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                author: item["owner"]["login"].as_str().unwrap_or("").to_string(),
                stars: item["stargazers_count"].as_u64().unwrap_or(0) as u32,
            })
        })
        .collect();

    Ok(plugins)
}

fn plugins_root() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join(".open-tools/plugins")
}

fn read_manifest(plugin_dir: &PathBuf) -> Result<serde_json::Value, String> {
    let manifest_path = plugin_dir.join("rubick.json");
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read rubick.json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid rubick.json: {e}"))
}

/// Clone a GitHub repo and install as a plugin.
#[tauri::command]
pub async fn install_plugin(repo_url: String) -> Result<InstalledPlugin, String> {
    let root = plugins_root();
    fs::create_dir_all(&root).map_err(|e| format!("mkdir: {e}"))?;

    let tmp = root.join(".tmp-clone");
    if tmp.exists() {
        fs::remove_dir_all(&tmp).ok();
    }

    let output = Command::new("git")
        .args(["clone", "--depth=1", &repo_url])
        .arg(&tmp)
        .output()
        .map_err(|e| format!("git clone failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr));
    }

    let manifest = read_manifest(&tmp)?;
    let plugin_id = manifest["id"]
        .as_str()
        .map(String::from)
        .or_else(|| {
            // Fallback: derive id from repo URL
            repo_url
                .split('/')
                .last()
                .map(|s| s.trim_end_matches(".git").to_string())
        })
        .ok_or_else(|| "Missing plugin id in rubick.json".to_string())?;

    let plugin_name = manifest["name"].as_str().unwrap_or(&plugin_id).to_string();
    let version = manifest["version"].as_str().unwrap_or("0.0.0").to_string();
    let description = manifest["description"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let dest = root.join(&plugin_id);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("rm existing plugin: {e}"))?;
    }
    fs::rename(&tmp, &dest).map_err(|e| format!("move plugin: {e}"))?;

    Ok(InstalledPlugin {
        id: plugin_id,
        name: plugin_name,
        version,
        description,
        enabled: true,
    })
}

/// Remove an installed plugin's directory.
#[tauri::command]
pub async fn uninstall_plugin(id: String) -> Result<(), String> {
    let dir = plugins_root().join(&id);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("rm plugin: {e}"))?;
    }
    Ok(())
}

/// List installed plugins by scanning the plugins directory.
#[tauri::command]
pub async fn list_plugins() -> Result<Vec<InstalledPlugin>, String> {
    let root = plugins_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();
    let dirs = fs::read_dir(&root).map_err(|e| format!("read_dir: {e}"))?;

    for entry in dirs.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("rubick.json");
        if !manifest_path.exists() {
            continue;
        }
        if let Ok(manifest) = read_manifest(&path) {
            let fallback = entry.file_name().to_string_lossy().to_string();
            let id = manifest["id"].as_str().unwrap_or(&fallback);
            plugins.push(InstalledPlugin {
                id: id.to_string(),
                name: manifest["name"].as_str().unwrap_or(id).to_string(),
                version: manifest["version"].as_str().unwrap_or("0.0.0").to_string(),
                description: manifest["description"].as_str().unwrap_or("").to_string(),
                enabled: true,
            });
        }
    }

    Ok(plugins)
}

/// Instruct the sidecar to load a plugin at runtime.
#[tauri::command]
pub async fn load_plugin(plugin_id: String, app: tauri::AppHandle) -> Result<String, String> {
    let root = plugins_root();
    let plugin_path = root.join(&plugin_id);
    let path_str = plugin_path.to_string_lossy().to_string();

    let payload = serde_json::json!({
        "id": format!("load-{plugin_id}"),
        "type": "load-plugin",
        "pluginId": plugin_id,
        "path": path_str
    })
    .to_string();

    let state = app.state::<crate::sidecar::SidecarState>();
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
