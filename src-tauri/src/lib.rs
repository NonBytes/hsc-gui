mod history;
mod scanner;

use history::HistoryEntry;
use scanner::{ScanOptions, ScanResult};
use std::fs;

#[tauri::command]
async fn scan_url(url: String, follow_redirects: bool, custom_headers: Vec<String>) -> ScanResult {
    let opts = ScanOptions {
        url,
        follow_redirects,
        custom_headers,
    };
    scanner::scan_url(opts).await
}

#[tauri::command]
async fn scan_file(content: String) -> ScanResult {
    scanner::scan_file(&content)
}

#[tauri::command]
async fn batch_scan(
    urls: Vec<String>,
    follow_redirects: bool,
    custom_headers: Vec<String>,
) -> Vec<ScanResult> {
    let handles: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let headers = custom_headers.clone();
            tokio::spawn(async move {
                scanner::scan_url(ScanOptions {
                    url,
                    follow_redirects,
                    custom_headers: headers,
                })
                .await
            })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(ScanResult {
                url: String::new(),
                final_url: String::new(),
                status_code: 0,
                http_version: String::new(),
                transport_security: Vec::new(),
                cors: Vec::new(),
                cookie_warnings: Vec::new(),
                present_headers: Vec::new(),
                missing_headers: Vec::new(),
                warnings: Vec::new(),
                all_headers: std::collections::HashMap::new(),
                error: Some(format!("Task failed: {}", e)),
            }),
        }
    }
    results
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn get_history() -> Vec<HistoryEntry> {
    history::load_history()
}

#[tauri::command]
fn save_to_history(url: String, result: ScanResult) -> Result<(), String> {
    let entry = HistoryEntry {
        id: format!("{}", chrono::Utc::now().timestamp_millis()),
        url,
        timestamp: chrono::Utc::now().to_rfc3339(),
        result,
    };
    history::save_entry(entry)
}

#[tauri::command]
fn delete_history_entry(id: String) -> Result<(), String> {
    history::delete_entry(&id)
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    history::clear_history()
}

#[tauri::command]
fn export_report(result: ScanResult, format: String) -> Result<String, String> {
    match format.as_str() {
        "json" => serde_json::to_string_pretty(&result).map_err(|e| e.to_string()),
        "markdown" => Ok(generate_markdown_report(&result)),
        _ => Err("Unsupported format".into()),
    }
}

fn generate_markdown_report(result: &ScanResult) -> String {
    let mut md = String::new();
    md.push_str("# Security Headers Report\n\n");
    md.push_str(&format!("**URL:** {}\n", result.url));
    md.push_str(&format!("**Final URL:** {}\n", result.final_url));
    md.push_str(&format!("**Status:** {}\n", result.status_code));
    md.push_str(&format!("**HTTP Version:** {}\n\n", result.http_version));

    md.push_str("## Transport Security\n\n");
    for t in &result.transport_security {
        md.push_str(&format!("- **{}:** {}\n", t.label, t.value));
    }

    md.push_str("\n## CORS Configuration\n\n");
    if result.cors.is_empty() {
        md.push_str("No CORS headers found (same-origin default).\n");
    } else {
        for c in &result.cors {
            md.push_str(&format!("- **{}:** {}\n", c.header, c.value));
        }
    }

    md.push_str("\n## Cookie Security\n\n");
    if result.cookie_warnings.is_empty() {
        md.push_str("No cookie issues found.\n");
    } else {
        for c in &result.cookie_warnings {
            md.push_str(&format!(
                "- **{}** — Missing: {}\n",
                c.name, c.missing_flags
            ));
        }
    }

    md.push_str("\n## Present Security Headers\n\n");
    if result.present_headers.is_empty() {
        md.push_str("None.\n");
    } else {
        for h in &result.present_headers {
            md.push_str(&format!("- **{}:** `{}`\n", h.name, h.value));
        }
    }

    md.push_str("\n## Missing Security Headers\n\n");
    if result.missing_headers.is_empty() {
        md.push_str("None — excellent!\n");
    } else {
        for h in &result.missing_headers {
            md.push_str(&format!("- **{}** — {}\n", h.name, h.description));
        }
    }

    md.push_str("\n## Warnings & Information Leaks\n\n");
    if result.warnings.is_empty() {
        md.push_str("None — good job!\n");
    } else {
        for w in &result.warnings {
            let icon = match w.severity.as_str() {
                "critical" => "CRITICAL",
                "warning" => "WARNING",
                _ => "INFO",
            };
            md.push_str(&format!("[{}] **{}:** {}\n", icon, w.header, w.message));
        }
    }

    md
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_url,
            scan_file,
            batch_scan,
            read_file,
            get_history,
            save_to_history,
            delete_history_entry,
            clear_history,
            export_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
