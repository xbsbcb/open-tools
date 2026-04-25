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

// ─── Base64 (STANDARD alphabet, pure std — no external crate) ─────────────────

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64_CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64_CHARS[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ─── Locale detection ─────────────────────────────────────────────────────────

fn detect_locale() -> Option<String> {
    let lang = std::env::var("LANG").ok()?;
    let locale = lang.split('.').next()?.to_string();
    if locale.is_empty() { None } else { Some(locale) }
}

// ─── Icon resolution ──────────────────────────────────────────────────────────

const HICOLOR_SIZES: &[u32] = &[256, 128, 64, 48, 32];

fn resolve_icon(icon: &str) -> String {
    if icon.is_empty() {
        return String::new();
    }

    // Absolute path — use directly if it exists
    if icon.starts_with('/') {
        let p = PathBuf::from(icon);
        if p.exists() {
            return read_icon_as_data_url(&p);
        }
        return String::new();
    }

    // hicolor theme — sizes from large to small
    for &size in HICOLOR_SIZES {
        let base = format!(
            "/usr/share/icons/hicolor/{}x{}/apps/{}",
            size, size, icon
        );

        let png = PathBuf::from(format!("{}.png", base));
        if png.exists() {
            return read_icon_as_data_url(&png);
        }

        let svg = PathBuf::from(format!("{}.svg", base));
        if svg.exists() {
            if let Ok(bytes) = fs::read(&svg) {
                return format!("data:image/svg+xml;base64,{}", base64_encode(&bytes));
            }
            return String::new();
        }
    }

    // /usr/share/pixmaps PNG fallback
    let pixmap_png = PathBuf::from(format!("/usr/share/pixmaps/{}.png", icon));
    if pixmap_png.exists() {
        return read_icon_as_data_url(&pixmap_png);
    }

    // .xpm — skipped (not renderable as data URL)
    String::new()
}

fn read_icon_as_data_url(path: &PathBuf) -> String {
    match fs::read(path) {
        Ok(bytes) => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let prefix = if ext == "svg" {
                "data:image/svg+xml;base64,"
            } else {
                "data:image/png;base64,"
            };
            format!("{}{}", prefix, base64_encode(&bytes))
        }
        Err(_) => String::new(),
    }
}

// ─── Fuzzy scoring ────────────────────────────────────────────────────────────

/// Returns a match score for `name` against `query`:
/// - 100 : name starts with query (prefix match)
/// -  80 : any word in name starts with query
/// -  50 : name contains query (substring)
/// -   0 : no match
fn score_match(name: &str, query: &str) -> u32 {
    if query.is_empty() {
        return 50;
    }
    let name_lower = name.to_lowercase();
    let query_lower = query.to_lowercase();

    if name_lower.starts_with(&query_lower) {
        return 100;
    }

    let word_match = name_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .any(|word| word.starts_with(&query_lower));
    if word_match {
        return 80;
    }

    if name_lower.contains(&query_lower) {
        return 50;
    }

    0
}

// ─── .desktop file parsing ────────────────────────────────────────────────────

fn parse_desktop_file(path: &PathBuf, query: &str) -> Option<AppResult> {
    let content = fs::read_to_string(path).ok()?;
    parse_desktop_content(&content, path, query, detect_locale())
}

fn parse_desktop_content(
    content: &str,
    path: &PathBuf,
    query: &str,
    locale: Option<String>,
) -> Option<AppResult> {
    let mut in_entry = false;
    let mut name = String::new();
    let mut bare_name = String::new();
    let mut exec = String::new();
    let mut icon_field = String::new();

    let locale_key = locale.as_deref().map(|l| format!("Name[{l}]="));
    let lang_key = locale.as_deref().and_then(|l| {
        if l.len() >= 2 { Some(format!("Name[{}]=", &l[..2])) } else { None }
    });

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_entry = true;
        } else if line.starts_with('[') {
            in_entry = false;
        } else if in_entry {
            if name.is_empty() {
                if let Some(ref key) = locale_key {
                    if line.starts_with(key) {
                        name = line[key.len()..].to_string();
                        continue;
                    }
                }
                if let Some(ref key) = lang_key {
                    if line.starts_with(key) {
                        name = line[key.len()..].to_string();
                        continue;
                    }
                }
            }
            if line.starts_with("Name=") && bare_name.is_empty() {
                bare_name = line["Name=".len()..].to_string();
            } else if line.starts_with("Exec=") && exec.is_empty() {
                exec = line["Exec=".len()..].to_string();
            } else if line.starts_with("Icon=") && icon_field.is_empty() {
                icon_field = line["Icon=".len()..].to_string();
            }
        }
    }

    if name.is_empty() {
        name = bare_name;
    }

    if name.is_empty() || exec.is_empty() {
        return None;
    }

    let score = score_match(&name, query);
    if score == 0 {
        return None;
    }

    let icon = resolve_icon(&icon_field);

    Some(AppResult {
        name,
        icon,
        path: path.to_string_lossy().to_string(),
        score,
    })
}

// ─── App directory scanner ────────────────────────────────────────────────────

fn scan_apps_dir(dir: PathBuf, query: &str, results: &mut Vec<AppResult>) {
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

// ─── File search ──────────────────────────────────────────────────────────────

/// Scan ~/Downloads, ~/Documents, ~/Desktop (one level deep).
/// Returns files whose stem contains `query` (case-insensitive), score = 30.
fn search_files(query: &str) -> Vec<AppResult> {
    let home = match std::env::var_os("HOME") {
        Some(h) => PathBuf::from(h),
        None => return Vec::new(),
    };

    let dirs = [
        home.join("Downloads"),
        home.join("Documents"),
        home.join("Desktop"),
    ];

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for dir in &dirs {
        let read = match fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue; // skip subdirectories — one level only
            }
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&file_name);
            if stem.to_lowercase().contains(&query_lower) {
                results.push(AppResult {
                    name: file_name,
                    icon: String::new(),
                    path: path.to_string_lossy().to_string(),
                    score: 30,
                });
            }
        }
    }

    results
}

// ─── Main Tauri command ───────────────────────────────────────────────────────

#[command]
pub fn search_apps(query: String) -> Vec<AppResult> {
    let mut results: Vec<AppResult> = Vec::new();

    // System-wide applications
    scan_apps_dir(
        PathBuf::from("/usr/share/applications"),
        &query,
        &mut results,
    );

    // User-local applications
    if let Some(home) = std::env::var_os("HOME") {
        let local_apps = PathBuf::from(home).join(".local/share/applications");
        scan_apps_dir(local_apps, &query, &mut results);
    }

    // Deduplicate by path before merging (keep whichever appeared first;
    // sort descending so the higher-scored duplicate survives dedup)
    results.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    results.dedup_by(|a, b| a.path == b.path);

    // Merge file results
    let mut file_results = search_files(&query);
    results.append(&mut file_results);

    // Final sort: score descending, then name ascending (stable tiebreak)
    results.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));

    results.truncate(20);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── base64_encode ────────────────────────────────────────────────────

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_one_byte() {
        assert_eq!(base64_encode(b"f"), "Zg==");
    }

    #[test]
    fn test_base64_encode_two_bytes() {
        assert_eq!(base64_encode(b"fo"), "Zm8=");
    }

    #[test]
    fn test_base64_encode_three_bytes() {
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn test_base64_encode_rfc4648_vectors() {
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"M"), "TQ==");
    }

    #[test]
    fn test_base64_encode_binary() {
        assert_eq!(base64_encode(&[0x00, 0xFF, 0xAA]), "AP+q");
    }

    // ─── score_match ──────────────────────────────────────────────────────

    #[test]
    fn test_score_match_prefix() {
        assert_eq!(score_match("Firefox", "Fire"), 100);
        assert_eq!(score_match("Google Chrome", "Goo"), 100);
    }

    #[test]
    fn test_score_match_prefix_case_insensitive() {
        assert_eq!(score_match("Firefox", "fire"), 100);
        assert_eq!(score_match("GOOGLE CHROME", "google"), 100);
    }

    #[test]
    fn test_score_match_word_prefix() {
        assert_eq!(score_match("Google Chrome", "Ch"), 80);
        assert_eq!(score_match("Visual Studio Code", "Stu"), 80);
        assert_eq!(score_match("Visual Studio Code", "Co"), 80);
    }

    #[test]
    fn test_score_match_hyphenated_word_prefix() {
        assert_eq!(score_match("my-app-name", "app"), 80);
        assert_eq!(score_match("my-app-name", "my"), 100);
    }

    #[test]
    fn test_score_match_substring() {
        assert_eq!(score_match("Firefox", "refo"), 50);
        assert_eq!(score_match("Google Chrome", "rome"), 50);
    }

    #[test]
    fn test_score_match_no_match() {
        assert_eq!(score_match("Firefox", "xyz"), 0);
    }

    #[test]
    fn test_score_match_empty_query() {
        assert_eq!(score_match("Anything", ""), 50);
    }

    #[test]
    fn test_score_match_cjk() {
        assert_eq!(score_match("谷歌浏览器", "谷歌"), 100);
        assert_eq!(score_match("谷歌浏览器", "浏览"), 50);
    }

    #[test]
    fn test_score_match_numeric_suffix() {
        // digits are alphanumeric, so not a word boundary — falls to substring
        assert_eq!(score_match("MyApp123", "123"), 50);
        assert_eq!(score_match("123MyApp", "123"), 100);
    }

    // ─── parse_desktop_file ───────────────────────────────────────────────

    use std::io::Write;

    fn write_temp_desktop(
        name: &str,
        exec: &str,
        icon: &str,
        locale_name: Option<(&str, &str)>,
    ) -> std::path::PathBuf {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.desktop");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[Desktop Entry]").unwrap();
        if let Some((locale, loc_name)) = locale_name {
            writeln!(f, "Name[{}]={}", locale, loc_name).unwrap();
        }
        writeln!(f, "Name={}", name).unwrap();
        writeln!(f, "Exec={}", exec).unwrap();
        if !icon.is_empty() {
            writeln!(f, "Icon={}", icon).unwrap();
        }
        std::mem::forget(dir);
        path
    }

    #[test]
    fn test_parse_desktop_file_basic() {
        let path = write_temp_desktop("Firefox", "/usr/bin/firefox %u", "", None);
        let result = parse_desktop_file(&path, "Fire").unwrap();
        assert_eq!(result.name, "Firefox");
        assert_eq!(result.score, 100);
    }

    #[test]
    fn test_parse_desktop_file_no_match() {
        let path = write_temp_desktop("Firefox", "/usr/bin/firefox", "", None);
        assert!(parse_desktop_file(&path, "xyz").is_none());
    }

    #[test]
    fn test_parse_desktop_file_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.desktop");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[Desktop Entry]").unwrap();
        writeln!(f, "Comment=A test").unwrap();
        let result = parse_desktop_file(&path, "any");
        assert!(result.is_none());
    }
}
