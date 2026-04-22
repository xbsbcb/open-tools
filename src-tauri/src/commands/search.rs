use tauri::command;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct AppResult {
    pub name: String,
    pub icon: String,
    pub path: String,
    pub score: u32,
}

/// Parse a single .desktop file, returning an AppResult if Name and Exec are present
/// and the name matches the query (case-insensitive contains).
fn parse_desktop_file(path: &PathBuf, query: &str) -> Option<AppResult> {
    let content = fs::read_to_string(path).ok()?;

    let mut in_entry = false;
    let mut name = String::new();
    let mut exec = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_entry = true;
        } else if line.starts_with('[') {
            in_entry = false;
        } else if in_entry {
            if line.starts_with("Name=") && name.is_empty() {
                name = line["Name=".len()..].to_string();
            } else if line.starts_with("Exec=") && exec.is_empty() {
                exec = line["Exec=".len()..].to_string();
            }
        }
    }

    if name.is_empty() || exec.is_empty() {
        return None;
    }

    if !name.to_lowercase().contains(&query.to_lowercase()) {
        return None;
    }

    Some(AppResult {
        name,
        icon: String::new(),
        path: path.to_string_lossy().to_string(),
        score: 0,
    })
}

/// Collect matching .desktop files from a directory into results
fn scan_dir(dir: PathBuf, query: &str, results: &mut Vec<AppResult>) {
    let read = match fs::read_dir(&dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
            if let Some(app) = parse_desktop_file(&path, query) {
                results.push(app);
            }
        }
    }
}

#[command]
pub fn search_apps(query: String) -> Vec<AppResult> {
    let mut results: Vec<AppResult> = Vec::new();

    // System-wide applications
    scan_dir(PathBuf::from("/usr/share/applications"), &query, &mut results);

    // User-local applications
    if let Some(home) = std::env::var_os("HOME") {
        let local_apps = PathBuf::from(home).join(".local/share/applications");
        scan_dir(local_apps, &query, &mut results);
    }

    // Sort by name, deduplicate by path, cap at 20
    results.sort_by(|a, b| a.name.cmp(&b.name));
    results.dedup_by(|a, b| a.path == b.path);
    results.truncate(20);

    results
}
