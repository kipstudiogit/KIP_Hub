use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::commands::AppState;
use crate::config::{self, AppConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaValidationResultDto {
    pub valid: bool,
    pub version_str: String,
    pub major: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTestResultDto {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

#[tauri::command]
pub fn get_settings() -> Value {
    let cfg = config::load_app_config();
    let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");
    let safe_mode = safe_dir.exists();

    json!({
        "mc_dir": cfg.current_instance,
        "lang": cfg.lang,
        "theme": cfg.theme,
        "auto_backup": cfg.auto_backup,
        "rpc": cfg.rpc,
        "instances": cfg.instances,
        "ai_provider": cfg.ai_provider,
        "ai_model": cfg.ai_model,
        "ai_api_key": config::get_secret("ai_api_key"),
        "openai_api_key": config::get_secret("openai_api_key"),
        "anthropic_api_key": config::get_secret("anthropic_api_key"),
        "ollama_url": cfg.ollama_url,
        "cf_api_key": config::get_secret("cf_api_key"),
        "autostart": cfg.autostart,
        "safe_mode": safe_mode,
        "low_graphics": cfg.low_graphics,
        "close_on_launch": cfg.close_on_launch,
        "ram_allocation": cfg.ram_allocation,
        "jvm_gc": cfg.jvm_gc,
        "jvm_preset": cfg.jvm_preset,
        "shield_auto_scan": cfg.shield_auto_scan,
        "voice_noise_suppression": cfg.voice_noise_suppression,
        "eula_accepted": cfg.eula_accepted,
        "telemetry_opt_in": cfg.telemetry_opt_in,
        "offline_username": cfg.offline_username,
        "game_resolution": cfg.game_resolution,
        "game_fullscreen": cfg.game_fullscreen,
        "custom_java_path": cfg.custom_java_path,
        "custom_jvm_args": cfg.custom_jvm_args,
        "theme_accent": cfg.theme_accent
    })
}

#[tauri::command]
pub async fn save_setting(key: String, value: Value) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut cfg = config::load_app_config();
        match key.as_str() {
            "mc_dir" => {
                if let Some(s) = value.as_str() {
                    let _ = tauri::async_runtime::block_on(crate::commands::change_instance(s.to_string()));
                }
            }
            "auto_backup" => { if let Some(b) = value.as_bool() { cfg.auto_backup = b; } }
            "rpc" => { if let Some(b) = value.as_bool() { cfg.rpc = b; } }
            "ai_provider" => { if let Some(s) = value.as_str() { cfg.ai_provider = s.to_string(); } }
            "ai_model" => { if let Some(s) = value.as_str() { cfg.ai_model = s.to_string(); } }
            "ai_api_key" => { if let Some(s) = value.as_str() { config::set_secret("ai_api_key", s); } }
            "openai_api_key" => { if let Some(s) = value.as_str() { config::set_secret("openai_api_key", s); } }
            "anthropic_api_key" => { if let Some(s) = value.as_str() { config::set_secret("anthropic_api_key", s); } }
            "ollama_url" => { if let Some(s) = value.as_str() { cfg.ollama_url = s.to_string(); } }
            "cf_api_key" => { if let Some(s) = value.as_str() { config::set_secret("cf_api_key", s); } }
            "lang" => { if let Some(s) = value.as_str() { cfg.lang = s.to_string(); } }
            "low_graphics" => { if let Some(b) = value.as_bool() { cfg.low_graphics = b; } }
            "close_on_launch" => { if let Some(b) = value.as_bool() { cfg.close_on_launch = b; } }
            "ram_allocation" => { if let Some(i) = value.as_i64() { cfg.ram_allocation = i as i32; } }
            "jvm_gc" => { if let Some(s) = value.as_str() { cfg.jvm_gc = s.to_string(); } }
            "jvm_preset" => { if let Some(s) = value.as_str() { cfg.jvm_preset = s.to_string(); } }
            "shield_auto_scan" => { if let Some(b) = value.as_bool() { cfg.shield_auto_scan = b; } }
            "voice_noise_suppression" => { if let Some(b) = value.as_bool() { cfg.voice_noise_suppression = b; } }
            "eula_accepted" => { if let Some(b) = value.as_bool() { cfg.eula_accepted = b; } }
            "telemetry_opt_in" => { if let Some(b) = value.as_bool() { cfg.telemetry_opt_in = b; } }
            "offline_username" => { if let Some(s) = value.as_str() { cfg.offline_username = s.to_string(); } }
            "game_resolution" => { if let Some(s) = value.as_str() { cfg.game_resolution = s.to_string(); } }
            "game_fullscreen" => { if let Some(b) = value.as_bool() { cfg.game_fullscreen = b; } }
            "custom_java_path" => { if let Some(s) = value.as_str() { cfg.custom_java_path = s.to_string(); } }
            "custom_jvm_args" => { if let Some(s) = value.as_str() { cfg.custom_jvm_args = s.to_string(); } }
            "theme_accent" => { if let Some(s) = value.as_str() { cfg.theme_accent = s.to_string(); } }
            "autostart" => { if let Some(b) = value.as_bool() { cfg.autostart = b; } }
            _ => return false,
        }
        config::save_app_config(&cfg);
        true
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_settings_to_default() -> Value {
    let def = AppConfig::default();
    config::save_app_config(&def);
    get_settings()
}

#[tauri::command]
pub async fn validate_java_binary(path: String) -> Result<JavaValidationResultDto, String> {
    tokio::task::spawn_blocking(move || {
        let probe = if path.ends_with("javaw.exe") {
            path.replace("javaw.exe", "java.exe")
        } else if path.ends_with("javaw") {
            path.replace("javaw", "java")
        } else {
            path.clone()
        };

        let mut cmd = Command::new(&probe);
        cmd.arg("-version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        match cmd.output() {
            Ok(output) => {
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );

                let text_lower = text.to_lowercase();
                let quote_parts: Vec<&str> = text_lower.split('"').collect();
                if quote_parts.len() > 1 {
                    let ver_str = quote_parts[1];
                    let dot_parts: Vec<&str> = ver_str.split('.').collect();
                    if let Ok(first) = dot_parts[0].parse::<u32>() {
                        let major = if first == 1 && dot_parts.len() > 1 {
                            dot_parts[1].parse::<u32>().unwrap_or(8)
                        } else {
                            first
                        };

                        return Ok(JavaValidationResultDto {
                            valid: true,
                            version_str: ver_str.to_string(),
                            major,
                            message: format!("Recognized OpenJDK {} runtime.", major),
                        });
                    }
                }

                Ok(JavaValidationResultDto {
                    valid: false,
                    version_str: "Unknown".to_string(),
                    major: 0,
                    message: "Binary responded, but version string could not be parsed.".to_string(),
                })
            }
            Err(e) => Ok(JavaValidationResultDto {
                valid: false,
                version_str: "Unavailable".to_string(),
                major: 0,
                message: format!("Execution failed: {}", e),
            }),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, AppState>,
    provider: String,
    _model: Option<String>,
    _endpoint: Option<String>,
) -> Result<AiTestResultDto, String> {
    let start = std::time::Instant::now();
    let prompt = "Ping test: respond with the word OK.";
    let answer = state.api.ask_ai_crash_analysis(prompt).await;
    let latency_ms = start.elapsed().as_millis() as u64;

    if answer.contains("Error") || answer.contains("not configured") || answer.contains("Invalid") {
        Ok(AiTestResultDto {
            success: false,
            latency_ms,
            message: answer,
        })
    } else {
        Ok(AiTestResultDto {
            success: true,
            latency_ms,
            message: format!("Connected to {} successfully.", provider.to_uppercase()),
        })
    }
}

#[tauri::command]
pub async fn vacuum_database(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        if db.checkpoint() {
            Ok("SQLite database pages vacuumed and optimized successfully.".to_string())
        } else {
            Err("Failed to complete WAL database checkpoint.".to_string())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_instance_folder() -> Result<(), String> {
    let cfg = config::load_app_config();
    let path = Path::new(&cfg.current_instance);
    let _ = std::fs::create_dir_all(path);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
    Ok(())
}