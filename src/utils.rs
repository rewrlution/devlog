use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::env;

pub fn devlog_path() -> PathBuf {
    // 1) Allow override through environment variable
    if let Ok(dir) = env::var("DEVLOG_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    // 2) Search upwards from the current working directory for a `.devlog/` folder
    if let Ok(mut cur) = env::current_dir() {
        loop {
            let candidate = cur.join(".devlog");
            if candidate.is_dir() {
                return candidate;
            }
            if !cur.pop() {
                break; // reached filesystem root
            }
        }
    }

    // 3) Fallback to relative `.devlog/` in the current directory
    Path::new(".devlog").to_path_buf()
}

pub fn list_existing_devlog_files() -> io::Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let path = devlog_path();
    if !path.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        // Only consider regular files
        if entry.file_type()?.is_file() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy().to_string();
            if is_valid_entry_filename(&file_name) {
                out.push(file_name);
            }
        }
    }
    // sort by filename (YYYYMMDD.md) descending (newest first)
    out.sort_by(|a, b| b.cmp(a));
    Ok(out)
}

pub fn is_valid_entry_filename(name: &str) -> bool {
    if name.len() != 11 || !name.ends_with(".md") {
        return false;
    }
    let date = &name[..8];
    date.chars().all(|c| c.is_ascii_digit())
}

pub fn today_str() -> String {
    let now = chrono::Local::now().naive_local().date();
    now.format("%Y%m%d").to_string()
}
