use tauri::command;
use std::fs;
use std::process::Command;

/// Strip field codes (%u, %U, %f, %F, etc.) from an Exec= value
fn strip_field_codes(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|token| !token.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

/// For a .desktop path, extract and run the Exec= command directly.
fn launch_desktop_file(path: &str) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read desktop file: {e}"))?;

    let mut exec_line = String::new();
    let mut in_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_entry = true;
        } else if line.starts_with('[') {
            in_entry = false;
        } else if in_entry && line.starts_with("Exec=") {
            exec_line = line["Exec=".len()..].to_string();
            break;
        }
    }

    if exec_line.is_empty() {
        return Err("No Exec= entry found in desktop file".to_string());
    }

    let clean = strip_field_codes(&exec_line);
    let mut parts = clean.split_whitespace();
    let bin = parts
        .next()
        .ok_or_else(|| "Empty Exec= command".to_string())?;
    let args: Vec<&str> = parts.collect();

    Command::new(bin)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to launch '{bin}': {e}"))?;

    Ok(())
}

#[command]
pub fn open_path(path: String) -> Result<(), String> {
    if path.ends_with(".desktop") {
        launch_desktop_file(&path)
    } else {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("xdg-open failed: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_field_codes_removes_all_codes() {
        assert_eq!(strip_field_codes("/usr/bin/firefox %u"), "/usr/bin/firefox");
        assert_eq!(strip_field_codes("/usr/bin/firefox %U"), "/usr/bin/firefox");
        assert_eq!(strip_field_codes("/usr/bin/evince %f"), "/usr/bin/evince");
        assert_eq!(strip_field_codes("/usr/bin/gedit %F"), "/usr/bin/gedit");
    }

    #[test]
    fn test_strip_field_codes_multiple_codes() {
        assert_eq!(
            strip_field_codes("/usr/bin/app %u %f --new-window"),
            "/usr/bin/app --new-window"
        );
    }

    #[test]
    fn test_strip_field_codes_no_codes() {
        assert_eq!(strip_field_codes("/usr/bin/app --flag"), "/usr/bin/app --flag");
    }

    #[test]
    fn test_strip_field_codes_empty() {
        assert_eq!(strip_field_codes(""), "");
    }

    #[test]
    fn test_strip_field_codes_only_codes() {
        assert_eq!(strip_field_codes("%u %F"), "");
    }
}
