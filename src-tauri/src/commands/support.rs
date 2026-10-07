use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::System;
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTelemetryDto {
    pub os_name: String,
    pub os_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub java_version: String,
    pub app_version: String,
    pub active_instance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDiagnosticResponseDto {
    pub success: bool,
    pub provider: String,
    pub model: String,
    pub local_hints: Vec<String>,
    pub culprit_mod: Option<String>,
    pub analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugReportSubmissionDto {
    pub report_text: String,
    pub include_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugReportResultDto {
    pub success: bool,
    pub report_id: String,
    pub message: String,
}

#[tauri::command]
pub async fn get_system_telemetry_dossier() -> Result<SystemTelemetryDto, AppError> {
    tokio::task::spawn_blocking(|| {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_bytes = sys.total_memory();
        let avail_bytes = sys.available_memory();

        let total_ram_gb = ((total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)) * 10.0).round() / 10.0;
        let available_ram_gb = ((avail_bytes as f64 / (1024.0 * 1024.0 * 1024.0)) * 10.0).round() / 10.0;

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Host Processor".to_string());
        let cpu_cores = sys.cpus().len();

        let os_name = System::name().unwrap_or_else(|| "Unknown OS".to_string());
        let os_version = System::os_version().unwrap_or_default();
        let java_version = SystemUtils::get_java_version();

        let cfg = config::load_app_config();
        let active_instance = Path::new(&cfg.current_instance)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Default (.minecraft)".to_string());

        Ok(SystemTelemetryDto {
            os_name,
            os_version,
            cpu_brand,
            cpu_cores,
            total_ram_gb,
            available_ram_gb,
            java_version,
            app_version: config::APP_VERSION.to_string(),
            active_instance,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn diagnose_crash_with_neural_core(
    state: State<'_, AppState>,
    log_snippet: String,
) -> Result<AiDiagnosticResponseDto, AppError> {
    let clean_input = log_snippet.trim();
    if clean_input.is_empty() {
        return Ok(AiDiagnosticResponseDto {
            success: false,
            provider: "offline".to_string(),
            model: "none".to_string(),
            local_hints: Vec::new(),
            culprit_mod: None,
            analysis: "Log payload is empty. Provide a valid crash stacktrace.".to_string(),
        });
    }

    let local_hints = ToolManager::analyze_crash_log(clean_input);
    let culprit_mod = local_hints
        .iter()
        .find(|h| h.starts_with("Suspected culprit:"))
        .map(|s| s.replace("Suspected culprit:", "").trim().to_string());

    let sanitized_snippet = ToolManager::scrub_personal_data(clean_input);
    let cfg = config::load_app_config();

    let ai_response = state.api.ask_ai_crash_analysis(&sanitized_snippet).await;
    let has_error = ai_response.contains("Error")
        || ai_response.contains("not configured")
        || ai_response.contains("Invalid");

    Ok(AiDiagnosticResponseDto {
        success: !has_error,
        provider: cfg.ai_provider,
        model: cfg.ai_model,
        local_hints,
        culprit_mod,
        analysis: ai_response,
    })
}

#[tauri::command]
pub async fn submit_encrypted_bug_report(
    payload: BugReportSubmissionDto,
) -> Result<BugReportResultDto, AppError> {
    let clean_report = ToolManager::scrub_personal_data(payload.report_text.trim());
    if clean_report.is_empty() {
        return Ok(BugReportResultDto {
            success: false,
            report_id: String::new(),
            message: "Report text cannot be empty.".to_string(),
        });
    }

    let telemetry_str = if payload.include_telemetry {
        let dossier = get_system_telemetry_dossier().await?;
        ToolManager::scrub_personal_data(&format!(
            "OS: {} {}\nCPU: {} ({} cores)\nRAM: {} GB total ({} GB free)\nJava: {}\nApp: v{}",
            dossier.os_name, dossier.os_version, dossier.cpu_brand, dossier.cpu_cores,
            dossier.total_ram_gb, dossier.available_ram_gb, dossier.java_version, dossier.app_version
        ))
    } else {
        "Telemetry excluded by operator".to_string()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()?;

    let request_body = json!({
        "type": "bug_report",
        "report": clean_report,
        "telemetry": telemetry_str,
        "version": config::APP_VERSION
    });

    let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
    let response = client.post(&target_url).json(&request_body).send().await?;

    if response.status().is_success() {
        let resp_json: Value = response.json().await.unwrap_or_default();
        let report_id = resp_json["id"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("local_{}", uuid::Uuid::new_v4().simple()));

        Ok(BugReportResultDto {
            success: true,
            report_id,
            message: "Encrypted diagnostic report submitted to Cloudflare KV Edge.".to_string(),
        })
    } else {
        Ok(BugReportResultDto {
            success: false,
            report_id: String::new(),
            message: format!("Dispatch server rejected frame with status {}", response.status()),
        })
    }
}

#[tauri::command]
pub async fn load_latest_crash_or_log() -> Result<String, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let crash_dir = mc_dir.join("crash-reports");

        if crash_dir.exists() {
            let mut reports: Vec<_> = fs::read_dir(&crash_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
                .map(|e| e.path())
                .collect();

            reports.sort_by(|a, b| {
                let a_m = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_m = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                b_m.cmp(&a_m)
            });

            if let Some(newest_crash) = reports.first() {
                if let Ok(f) = File::open(newest_crash) {
                    let mut buffer = String::new();
                    let mut handle = f.take(12000);
                    if handle.read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
                        return Ok(ToolManager::scrub_personal_data(&buffer));
                    }
                }
            }
        }

        let log_path = mc_dir.join("logs").join("latest.log");
        if log_path.exists() {
            let file = File::open(&log_path)?;
            let reader = BufReader::new(file);
            let mut lines = Vec::new();
            for l in reader.lines().map_while(Result::ok) {
                lines.push(l);
                if lines.len() > 1000 {
                    lines.remove(0);
                }
            }
            let start = lines.len().saturating_sub(250);
            let joined = lines[start..].join("\n");
            return Ok(ToolManager::scrub_personal_data(&joined));
        }

        Ok("No crash reports or active logs detected on disk.".to_string())
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}