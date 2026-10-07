use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLineDto {
    pub id: u64,
    pub raw: String,
    pub timestamp: Option<String>,
    pub level: String,
    pub thread: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleExportResultDto {
    pub success: bool,
    pub file_path: Option<String>,
    pub mclogs_url: Option<String>,
    pub message: String,
}

fn parse_log_line(id: u64, line: &str) -> ConsoleLineDto {
    let trimmed = line.trim();
    let mut timestamp = None;
    let mut thread = None;
    let mut level = "INFO".to_string();
    let mut message = trimmed.to_string();

    if trimmed.starts_with('[') {
        if let Some(close_first) = trimmed.find(']') {
            let first_token = &trimmed[1..close_first];
            if first_token.contains(':') && first_token.len() <= 12 {
                timestamp = Some(first_token.to_string());
                let rest = trimmed[close_first + 1..].trim();

                if rest.starts_with('[') {
                    if let Some(close_second) = rest.find(']') {
                        let second_token = &rest[1..close_second];
                        if let Some((t_name, l_name)) = second_token.split_once('/') {
                            thread = Some(t_name.trim().to_string());
                            level = l_name.trim().to_uppercase();
                        } else {
                            thread = Some(second_token.to_string());
                        }

                        if let Some(colon_pos) = rest[close_second + 1..].find(':') {
                            message = rest[close_second + 1 + colon_pos + 1..].trim().to_string();
                        } else {
                            message = rest[close_second + 1..].trim().to_string();
                        }
                    }
                }
            }
        }
    }

    if level == "INFO" {
        let upper_msg = message.to_uppercase();
        if upper_msg.contains("ERROR") || upper_msg.contains("EXCEPTION") || upper_msg.contains("CRITICAL") {
            level = "ERROR".to_string();
        } else if upper_msg.contains("WARN") {
            level = "WARN".to_string();
        }
    }

    ConsoleLineDto {
        id,
        raw: trimmed.to_string(),
        timestamp,
        level,
        thread,
        message,
    }
}

#[tauri::command]
pub async fn get_recent_console_entries(
    limit: Option<usize>,
) -> Result<Vec<ConsoleLineDto>, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path)?;
        let reader = BufReader::new(file);
        let max_entries = limit.unwrap_or(500);

        let mut raw_lines = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            raw_lines.push(line);
            if raw_lines.len() > max_entries * 2 {
                raw_lines.drain(0..max_entries);
            }
        }

        let start_idx = raw_lines.len().saturating_sub(max_entries);
        let mut parsed_entries = Vec::new();

        for (idx, line) in raw_lines[start_idx..].iter().enumerate() {
            parsed_entries.push(parse_log_line((start_idx + idx) as u64, line));
        }

        Ok(parsed_entries)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn upload_active_console_log(
    state: State<'_, AppState>,
) -> Result<ConsoleExportResultDto, AppError> {
    let cfg = config::load_app_config();
    let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");

    if !log_path.exists() {
        return Ok(ConsoleExportResultDto {
            success: false,
            file_path: None,
            mclogs_url: None,
            message: "latest.log not found in active profile.".to_string(),
        });
    }

    let raw_log = tokio::fs::read_to_string(&log_path).await?;
    let sanitized_log = ToolManager::scrub_personal_data(&raw_log);

    if let Some(url) = state.api.upload_to_mclogs(&sanitized_log).await {
        Ok(ConsoleExportResultDto {
            success: true,
            file_path: None,
            mclogs_url: Some(url),
            message: "Log uploaded to mclo.gs successfully.".to_string(),
        })
    } else {
        Ok(ConsoleExportResultDto {
            success: false,
            file_path: None,
            mclogs_url: None,
            message: "Failed to upload log payload to mclo.gs endpoint.".to_string(),
        })
    }
}

#[tauri::command]
pub async fn export_console_log_to_file() -> Result<ConsoleExportResultDto, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");

        if !log_path.exists() {
            return Ok(ConsoleExportResultDto {
                success: false,
                file_path: None,
                mclogs_url: None,
                message: "latest.log does not exist on disk.".to_string(),
            });
        }

        let raw_log = fs::read_to_string(&log_path)?;
        let sanitized = ToolManager::scrub_personal_data(&raw_log);

        let export_dir = config::get_app_data_dir().join("exports").join("logs");
        fs::create_dir_all(&export_dir)?;

        let filename = format!("latest_log_dump_{}.log", Local::now().format("%Y%m%d_%H%M%S"));
        let target_path: PathBuf = export_dir.join(&filename);

        fs::write(&target_path, sanitized)?;

        Ok(ConsoleExportResultDto {
            success: true,
            file_path: Some(target_path.to_string_lossy().to_string()),
            mclogs_url: None,
            message: format!("Log saved securely to {:?}", target_path),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn toggle_console_streaming(state: State<'_, AppState>, active: bool) -> bool {
    state.monitor.console_streaming.store(active, Ordering::Relaxed);
    true
}