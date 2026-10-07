use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;
use walkdir::WalkDir;

use crate::commands::AppState;
use crate::config;
use crate::doctor_manager::DoctorAnalysisReportDto;
use crate::error::AppError;
use crate::shield::quarantine::QuarantineRecordDto;
use crate::shield::ShieldScanReportDto;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionResultDto {
    pub success: bool,
    pub msg: String,
    pub clipboard: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeModeStatusDto {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldScanProgressDto {
    pub current_file: String,
    pub scanned_count: usize,
    pub total_files: usize,
    pub percent: f64,
    pub threats_found: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashInvestigationDto {
    pub has_crash: bool,
    pub filename: String,
    pub timestamp: String,
    pub culprit_mod: Option<String>,
    pub hints: Vec<String>,
    pub raw_snippet: String,
}

#[tauri::command]
pub async fn execute_system_tool(
    state: State<'_, AppState>,
    tool_id: String,
    _mc_version: Option<String>,
    _loader: Option<String>,
) -> Result<ToolExecutionResultDto, AppError> {
    if tool_id == "mclogs" {
        let cfg = config::load_app_config();
        let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");
        if !log_path.exists() {
            return Ok(ToolExecutionResultDto {
                success: false,
                msg: "Active instance log file (latest.log) not found.".to_string(),
                clipboard: None,
            });
        }
        let content = tokio::fs::read_to_string(&log_path).await?;
        if let Some(url) = state.api.upload_to_mclogs(&content).await {
            return Ok(ToolExecutionResultDto {
                success: true,
                msg: "Log uploaded to mclo.gs successfully. URL copied.".to_string(),
                clipboard: Some(url),
            });
        }
        return Ok(ToolExecutionResultDto {
            success: false,
            msg: "Failed to upload log to mclo.gs.".to_string(),
            clipboard: None,
        });
    }

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = cfg.current_instance.clone();

        match tool_id.as_str() {
            "clean_logs" => {
                let logs_dir = Path::new(&mc_dir).join("logs");
                let (count, freed_mb) = ToolManager::clean_logs(logs_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("Deleted {} old log files ({:.1} MB freed).", count, freed_mb),
                    clipboard: None,
                })
            }
            "kill_java" => {
                let killed = SystemUtils::kill_zombie_processes();
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("Terminated {} orphaned Java processes.", killed),
                    clipboard: None,
                })
            }
            "backup" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let backups_dir = Path::new(&mc_dir).join("backups_devkit");
                let size = ToolManager::create_backup(
                    saves_dir.to_str().unwrap_or(""),
                    backups_dir.to_str().unwrap_or(""),
                );
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("World backup archive created ({:.1} MB).", size),
                    clipboard: None,
                })
            }
            "clean_worlds" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let cleaned = ToolManager::clean_world_caches(saves_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("Purged minimap chunk caches across {} worlds.", cleaned),
                    clipboard: None,
                })
            }
            "flush_dns" => {
                SystemUtils::flush_dns_cache();
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: "Local DNS cache flushed successfully.".to_string(),
                    clipboard: None,
                })
            }
            "unlock_worlds" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let unlocked = ToolManager::unlock_worlds(saves_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("Removed locks from {} world saves.", unlocked),
                    clipboard: None,
                })
            }
            "generate_jvm" => {
                let (ram, args) = SystemUtils::generate_jvm_args();
                Ok(ToolExecutionResultDto {
                    success: true,
                    msg: format!("Generated optimized arguments for {} GB RAM.", ram),
                    clipboard: Some(args),
                })
            }
            "wipe_configs" => {
                let cfg_dir = Path::new(&mc_dir).join("config");
                let success = ToolManager::wipe_configs(cfg_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResultDto {
                    success,
                    msg: if success {
                        "Instance config folder purged.".to_string()
                    } else {
                        "Config folder missing or locked.".to_string()
                    },
                    clipboard: None,
                })
            }
            "reset_video" => {
                let options_txt = Path::new(&mc_dir).join("options.txt");
                let success = if options_txt.exists() {
                    fs::remove_file(options_txt).is_ok()
                } else {
                    false
                };
                Ok(ToolExecutionResultDto {
                    success,
                    msg: if success {
                        "options.txt removed. Graphics reset to default.".to_string()
                    } else {
                        "options.txt not found.".to_string()
                    },
                    clipboard: None,
                })
            }
            "ai_fps" => {
                let applied = SystemUtils::optimize_fps(&mc_dir);
                if applied >= 0 {
                    Ok(ToolExecutionResultDto {
                        success: true,
                        msg: format!("Applied {} graphics and render distance tweaks.", applied),
                        clipboard: None,
                    })
                } else {
                    Ok(ToolExecutionResultDto {
                        success: false,
                        msg: "Optimization failed. Check options.txt access rights.".to_string(),
                        clipboard: None,
                    })
                }
            }
            _ => Ok(ToolExecutionResultDto {
                success: false,
                msg: format!("Unknown tool command identifier: {}", tool_id),
                clipboard: None,
            }),
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn run_tool(
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<ToolExecutionResultDto, AppError> {
    execute_system_tool(state, tool_id, None, None).await
}

#[tauri::command]
pub async fn investigate_latest_crash() -> Result<CrashInvestigationDto, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let crash_dir = Path::new(&cfg.current_instance).join("crash-reports");

        if !crash_dir.exists() {
            return Ok(CrashInvestigationDto {
                has_crash: false,
                filename: String::new(),
                timestamp: String::new(),
                culprit_mod: None,
                hints: Vec::new(),
                raw_snippet: String::new(),
            });
        }

        let mut reports: Vec<_> = fs::read_dir(&crash_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
            .collect();

        reports.sort_by(|a, b| {
            let a_m = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_m = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_m.cmp(&a_m)
        });

        if let Some(newest) = reports.first() {
            let filename = newest.file_name().to_string_lossy().to_string();
            let timestamp = newest
                .metadata()
                .and_then(|m| m.modified())
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Local> = t.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "Recent".to_string());

            if let Ok(file) = File::open(newest.path()) {
                let mut buffer = String::new();
                let mut handle = file.take(8192);
                let _ = handle.read_to_string(&mut buffer);

                let hints = ToolManager::analyze_crash_log(&buffer);
                let culprit = hints.iter().find(|h| h.starts_with("Suspected culprit:")).cloned();

                return Ok(CrashInvestigationDto {
                    has_crash: true,
                    filename,
                    timestamp,
                    culprit_mod: culprit,
                    hints,
                    raw_snippet: buffer,
                });
            }
        }

        Ok(CrashInvestigationDto {
            has_crash: false,
            filename: String::new(),
            timestamp: String::new(),
            culprit_mod: None,
            hints: Vec::new(),
            raw_snippet: String::new(),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn analyze_mod_doctor(
    state: State<'_, AppState>,
    mc_version: String,
    loader: String,
) -> Result<DoctorAnalysisReportDto, AppError> {
    let doctor = state.doctor.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let cfg_dir = mc_dir.join("config");

        let raw_val = doctor.run_analysis(
            mods_dir.to_str().unwrap_or(""),
            cfg_dir.to_str().unwrap_or(""),
            &mc_version,
            &loader,
        );

        serde_json::from_value::<DoctorAnalysisReportDto>(raw_val)
            .map_err(|e| AppError::Serialization(e))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn apply_doctor_remediation(
    state: State<'_, AppState>,
    issues: Vec<Value>,
    mc_version: String,
    loader: String,
) -> Result<Value, AppError> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");

    Ok(state
        .doctor
        .apply_fixes(
            mods_dir.to_str().unwrap_or(""),
            &state.api,
            &issues,
            &mc_version,
            &loader,
        )
        .await)
}

#[tauri::command]
pub async fn scan_shield_security_stream(
    state: State<'_, AppState>,
    target_dir: Option<String>,
    progress_channel: Channel<ShieldScanProgressDto>,
) -> Result<ShieldScanReportDto, AppError> {
    let shield = state.shield.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let dir_str = target_dir
            .unwrap_or_else(|| Path::new(&cfg.current_instance).join("mods").to_string_lossy().to_string());
        let path = Path::new(&dir_str);

        let mut targets: Vec<PathBuf> = Vec::new();
        if path.exists() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    if ext == "jar" || ext == "zip" {
                        targets.push(p.to_path_buf());
                    }
                }
            }
        }

        let total_files = targets.len();
        let mut threats = Vec::new();
        let mut clean_count = 0;
        let mut critical_count = 0;

        for (idx, file_path) in targets.iter().enumerate() {
            let fname = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let percent = if total_files > 0 { ((idx as f64) / (total_files as f64)) * 100.0 } else { 100.0 };

            let _ = progress_channel.send(ShieldScanProgressDto {
                current_file: fname,
                scanned_count: idx + 1,
                total_files,
                percent,
                threats_found: threats.len(),
            });

            let report = shield.scan_single_file(file_path.to_str().unwrap_or(""));
            if report.is_clean {
                clean_count += 1;
            } else {
                if report.threat_level == "CRITICAL" {
                    critical_count += 1;
                }
                threats.push(report);
            }
        }

        threats.sort_by(|a, b| b.threat_score.cmp(&a.threat_score));
        let threat_count = threats.len();

        let _ = progress_channel.send(ShieldScanProgressDto {
            current_file: "Audit Completed".to_string(),
            scanned_count: total_files,
            total_files,
            percent: 100.0,
            threats_found: threat_count,
        });

        Ok(ShieldScanReportDto {
            total_scanned: total_files,
            clean_count,
            threat_count,
            critical_count,
            threats,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn quarantine_shield_file(
    state: State<'_, AppState>,
    filepath: String,
) -> Result<QuarantineRecordDto, AppError> {
    let shield = state.shield.clone();

    tokio::task::spawn_blocking(move || {
        shield
            .quarantine(&filepath)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn restore_shield_record(
    state: State<'_, AppState>,
    quarantine_id: String,
) -> Result<bool, AppError> {
    let shield = state.shield.clone();

    tokio::task::spawn_blocking(move || {
        shield
            .restore(&quarantine_id)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn shred_shield_record(
    state: State<'_, AppState>,
    quarantine_id: String,
) -> Result<bool, AppError> {
    let shield = state.shield.clone();

    tokio::task::spawn_blocking(move || {
        shield
            .shred(&quarantine_id)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn get_shield_vault_records(state: State<'_, AppState>) -> Vec<QuarantineRecordDto> {
    state.shield.get_vault_records()
}

#[tauri::command]
pub fn get_safe_mode_status() -> SafeModeStatusDto {
    let cfg = config::load_app_config();
    let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");
    SafeModeStatusDto {
        enabled: safe_dir.exists(),
    }
}

#[tauri::command]
pub fn toggle_safe_mode_status(state: State<'_, AppState>) -> Result<SafeModeStatusDto, AppError> {
    if state.monitor.is_mc_running.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(AppError::Launch("Cannot toggle Safe Mode while Minecraft process is running.".to_string()));
    }

    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");

    if safe_dir.exists() {
        if mods_dir.exists() {
            let _ = fs::remove_dir_all(&mods_dir);
        }
        let _ = fs::rename(&safe_dir, &mods_dir);
        Ok(SafeModeStatusDto { enabled: false })
    } else {
        if mods_dir.exists() {
            let _ = fs::rename(&mods_dir, &safe_dir);
        }
        Ok(SafeModeStatusDto { enabled: true })
    }
}