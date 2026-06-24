use crate::scanner::ScanResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub timestamp: String,
    pub result: ScanResult,
}

fn history_path() -> PathBuf {
    let mut path = data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("hsc-gui");
    fs::create_dir_all(&path).ok();
    path.push("history.json");
    path
}

fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
    }
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_entry(entry: HistoryEntry) -> Result<(), String> {
    let mut history = load_history();
    history.insert(0, entry);
    if history.len() > 500 {
        history.truncate(500);
    }
    let path = history_path();
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn delete_entry(id: &str) -> Result<(), String> {
    let mut history = load_history();
    history.retain(|e| e.id != id);
    let path = history_path();
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn clear_history() -> Result<(), String> {
    let path = history_path();
    fs::write(&path, "[]").map_err(|e| e.to_string())
}
