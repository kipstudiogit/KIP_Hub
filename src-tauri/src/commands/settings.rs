use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::commands::AppState;
use crate::config::{self, AppConfig};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsDto {
    pub lang: String,
    pub autostart: bool,
    pub theme: String,
    pub custom_color: String,
    pub appearance: String,
    pub mica: bool,
    pub rpc: bool,
    pub scale: f32,
    pub instances: Vec<String>,
    pub current_instance: String,
    pub auto_backup: bool,
    pub ptero_url: String,
    pub low_graphics: bool,
    pub ai_provider: String,
    pub ai_model: String,
    pub ollama_url: String,
    pub close_on_launch: bool,
    pub ram_allocation: i32,
    pub jvm_gc: String,
    pub jvm_preset: String,
    pub shield_auto_scan: bool,
    pub voice_noise_suppression: bool,
    pub eula_accepted: bool,
    pub telemetry_opt_in: bool,
    pub kip_username: String,
    pub offline_username: String,
    pub game_resolution: String,
    pub game_fullscreen: bool,
    pub custom_java_path: String,
    pub custom_jvm_args: String,
    pub theme_accent: String,
    pub ai_api_key: String,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub cf_api_key: String,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaValidationResultDto {
    pub valid: bool,
    pub version_str: String,
    pub major: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedJavaRuntimeDto {
    pub path: String,
    pub major: u32,
    pub version_str: String,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTestResultDto {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

impl From<AppConfig> for AppSettingsDto {
    fn from(cfg: AppConfig) -> Self {
        let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");
        let safe_mode = safe_dir.exists();

        Self {
            lang: cfg.lang,
            autostart: cfg.autostart,
            theme: cfg.theme,
            custom_color: cfg.custom_color,
            appearance: cfg.appearance,
            mica: cfg.mica,
            rpc: cfg.rpc,
            scale: cfg.scale,
            instances: cfg.instances,
            current_instance: cfg.current_instance,
            auto_backup: cfg.auto_backup,
            ptero_url: cfg.ptero_url,
            low_graphics: cfg.low_graphics,
            ai_provider: cfg.ai_provider,
            ai_model: cfg.ai_model,
            ollama_url: cfg.ollama_url,
            close_on_launch: cfg.close_on_launch,
            ram_allocation: cfg.ram_allocation,
            jvm_gc: cfg.jvm_gc,
            jvm_preset: cfg.jvm_preset,
            shield_auto_scan: cfg.shield_auto_scan,
            voice_noise_suppression: cfg.voice_noise_suppression,
            eula_accepted: cfg.eula_accepted,
            telemetry_opt_in: cfg.telemetry_opt_in,
            kip_username: cfg.kip_username,
            offline_username: cfg.offline_username,
            game_resolution: cfg.game_resolution,
            game_fullscreen: cfg.game_fullscreen,
            custom_java_path: cfg.custom_java_path,
            custom_jvm_args: cfg.custom_jvm_args,
            theme_accent: cfg.theme_accent,
            ai_api_key: config::get_secret("ai_api_key"),
            openai_api_key: config::get_secret("openai_api_key"),
            anthropic_api_key: config::get_secret("anthropic_api_key"),
            cf_api_key: config::get_secret("cf_api_key"),
            safe_mode,
        }
    }
}

fn probe_java_binary(path: &Path) -> Option<DetectedJavaRuntimeDto> {
    let probe = if path.to_string_lossy().ends_with("javaw.exe") {
        PathBuf::from(path.to_string_lossy().replace("javaw.exe", "java.exe"))
    } else {
        path.to_path_buf()
    };

    let mut cmd = Command::new(&probe);
    cmd.arg("-version");
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    let out = cmd.output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ).to_lowercase();

    let quote_parts: Vec<&str> = text.split('"').collect();
    if quote_parts.len() > 1 {
        let ver_str = quote_parts[1];
        let dot_parts: Vec<&str> = ver_str.split('.').collect();
        if let Ok(first) = dot_parts[0].parse::<u32>() {
            let major = if first == 1 && dot_parts.len() > 1 {
                dot_parts[1].parse::<u32>().unwrap_or(8)
            } else {
                first
            };

            let vendor = if text.contains("adoptium") || text.contains("temurin") {
                "Eclipse Adoptium".to_string()
            } else if text.contains("microsoft") {
                "Microsoft OpenJDK".to_string()
            } else if text.contains("corretto") {
                "Amazon Corretto".to_string()
            } else if text.contains("zulu") {
                "Azul Zulu".to_string()
            } else {
                "HotSpot Standard".to_string()
            };

            return Some(DetectedJavaRuntimeDto {
                path: path.to_string_lossy().to_string(),
                major,
                version_str: ver_str.to_string(),
                vendor,
            });
        }
    }
    None
}

#[tauri::command]
pub fn get_app_settings() -> AppSettingsDto {
    let cfg = config::load_app_config();
    AppSettingsDto::from(cfg)
}

#[tauri::command]
pub fn get_settings() -> Value {
    let cfg = config::load_app_config();
    serde_json::to_value(cfg).unwrap_or_default()
}

#[tauri::command]
pub async fn save_setting(key: String, value: Value) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut cfg = config::load_app_config();

        match key.as_str() {
            "mc_dir" | "current_instance" | "currentInstance" => {
                if let Some(s) = value.as_str() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        let _ = fs::create_dir_all(trimmed);
                        cfg.current_instance = trimmed.to_string();
                        if !cfg.instances.contains(&trimmed.to_string()) {
                            cfg.instances.push(trimmed.to_string());
                        }
                        config::update_paths(trimmed);
                    }
                }
            }
            "offline_username" | "offlineUsername" => {
                if let Some(s) = value.as_str() {
                    cfg.offline_username = s.to_string();
                }
            }
            "eula_accepted" | "eulaAccepted" => {
                if let Some(b) = value.as_bool() {
                    cfg.eula_accepted = b;
                }
            }
            "lang" => {
                if let Some(s) = value.as_str() {
                    cfg.lang = s.to_string();
                }
            }
            "autostart" => {
                if let Some(b) = value.as_bool() {
                    cfg.autostart = b;
                }
            }
            "theme_accent" | "themeAccent" => {
                if let Some(s) = value.as_str() {
                    cfg.theme_accent = s.to_string();
                }
            }
            "low_graphics" | "lowGraphics" => {
                if let Some(b) = value.as_bool() {
                    cfg.low_graphics = b;
                }
            }
            "auto_backup" | "autoBackup" => {
                if let Some(b) = value.as_bool() {
                    cfg.auto_backup = b;
                }
            }
            "rpc" => {
                if let Some(b) = value.as_bool() {
                    cfg.rpc = b;
                }
            }
            "ram_allocation" | "ramAllocation" => {
                if let Some(i) = value.as_i64() {
                    cfg.ram_allocation = i as i32;
                }
            }
            _ => {
                let mut cfg_val = serde_json::to_value(&cfg).unwrap_or_default();
                if let Some(map) = cfg_val.as_object_mut() {
                    map.insert(key, value);
                    if let Ok(updated) = serde_json::from_value::<AppConfig>(cfg_val) {
                        cfg = updated;
                    }
                }
            }
        }

        config::save_app_config(&cfg);
        Ok(true)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn update_app_settings(settings: AppSettingsDto) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut cfg = config::load_app_config();

        let target_dir = settings.current_instance.trim();
        if !target_dir.is_empty() && target_dir != cfg.current_instance {
            let p = Path::new(target_dir);
            let _ = fs::create_dir_all(p);
            cfg.current_instance = target_dir.to_string();
            if !cfg.instances.contains(&target_dir.to_string()) {
                cfg.instances.push(target_dir.to_string());
            }
            config::update_paths(target_dir);
        }

        if settings.autostart != cfg.autostart {
            cfg.autostart = settings.autostart;
            #[cfg(target_os = "windows")]
            {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = exe_path.to_string_lossy().to_string();
                    let mut cmd = Command::new("powershell");
                    cmd.arg("-NoProfile").arg("-NonInteractive").arg("-Command");
                    if settings.autostart {
                        cmd.arg(format!(
                            "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'KIP_Hub' -Value '\"{}\"'",
                            path_str
                        ));
                    } else {
                        cmd.arg("Remove-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'KIP_Hub' -ErrorAction SilentlyContinue");
                    }
                    cmd.creation_flags(0x08000000);
                    let _ = cmd.output();
                }
            }
        }

        config::set_secret("ai_api_key", &settings.ai_api_key);
        config::set_secret("openai_api_key", &settings.openai_api_key);
        config::set_secret("anthropic_api_key", &settings.anthropic_api_key);
        config::set_secret("cf_api_key", &settings.cf_api_key);

        cfg.lang = settings.lang;
        cfg.theme = settings.theme;
        cfg.custom_color = settings.custom_color;
        cfg.appearance = settings.appearance;
        cfg.mica = settings.mica;
        cfg.rpc = settings.rpc;
        cfg.scale = settings.scale;
        cfg.auto_backup = settings.auto_backup;
        cfg.ptero_url = settings.ptero_url;
        cfg.low_graphics = settings.low_graphics;
        cfg.ai_provider = settings.ai_provider;
        cfg.ai_model = settings.ai_model;
        cfg.ollama_url = settings.ollama_url;
        cfg.close_on_launch = settings.close_on_launch;
        cfg.ram_allocation = settings.ram_allocation;
        cfg.jvm_gc = settings.jvm_gc;
        cfg.jvm_preset = settings.jvm_preset;
        cfg.shield_auto_scan = settings.shield_auto_scan;
        cfg.voice_noise_suppression = settings.voice_noise_suppression;
        cfg.eula_accepted = settings.eula_accepted;
        cfg.telemetry_opt_in = settings.telemetry_opt_in;
        cfg.offline_username = settings.offline_username;
        cfg.game_resolution = settings.game_resolution;
        cfg.game_fullscreen = settings.game_fullscreen;
        cfg.custom_java_path = settings.custom_java_path;
        cfg.custom_jvm_args = settings.custom_jvm_args;
        cfg.theme_accent = settings.theme_accent;

        config::save_app_config(&cfg);
        Ok(true)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn reset_app_settings_default() -> AppSettingsDto {
    let def = AppConfig::default();
    config::save_app_config(&def);
    AppSettingsDto::from(def)
}

#[tauri::command]
pub async fn validate_java_executable(path: String) -> Result<JavaValidationResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let p = Path::new(&path);
        if !p.exists() {
            return Ok(JavaValidationResultDto {
                valid: false,
                version_str: "Unset".to_string(),
                major: 0,
                message: "No binary located at specified path.".to_string(),
            });
        }

        if let Some(info) = probe_java_binary(p) {
            Ok(JavaValidationResultDto {
                valid: true,
                version_str: info.version_str,
                major: info.major,
                message: format!("Verified Java {} ({}) runtime.", info.major, info.vendor),
            })
        } else {
            Ok(JavaValidationResultDto {
                valid: false,
                version_str: "Unknown".to_string(),
                major: 0,
                message: "Executable responded, but version string could not be parsed.".to_string(),
            })
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn detect_system_java_runtimes() -> Result<Vec<DetectedJavaRuntimeDto>, AppError> {
    tokio::task::spawn_blocking(|| {
        let mut detected = Vec::new();
        let mut candidates = Vec::new();

        if let Ok(jh) = std::env::var("JAVA_HOME") {
            let p = PathBuf::from(jh);
            #[cfg(target_os = "windows")]
            {
                candidates.push(p.join("bin").join("javaw.exe"));
                candidates.push(p.join("bin").join("java.exe"));
            }
            #[cfg(not(target_os = "windows"))]
            {
                candidates.push(p.join("bin").join("java"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let roots = [
                "C:\\Program Files\\Eclipse Adoptium",
                "C:\\Program Files\\Microsoft",
                "C:\\Program Files\\Java",
                "C:\\Program Files\\BellSoft",
                "C:\\Program Files\\Amazon Corretto",
                "C:\\Program Files\\Zulu",
            ];
            for root in roots {
                let rpath = Path::new(root);
                if rpath.is_dir() {
                    if let Ok(entries) = fs::read_dir(rpath) {
                        for e in entries.flatten() {
                            let javaw = e.path().join("bin").join("javaw.exe");
                            if javaw.exists() {
                                candidates.push(javaw);
                            }
                        }
                    }
                }
            }
        }

        for c in candidates {
            if c.exists() {
                if let Some(info) = probe_java_binary(&c) {
                    if !detected.iter().any(|d: &DetectedJavaRuntimeDto| d.path == info.path) {
                        detected.push(info);
                    }
                }
            }
        }

        Ok(detected)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn pick_instance_directory() -> Result<String, AppError> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("powershell");
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg("Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $f.SelectedPath }");
            cmd.creation_flags(0x08000000);
            if let Ok(res) = cmd.output() {
                if res.status.success() {
                    let path_str = String::from_utf8_lossy(&res.stdout).trim().to_string();
                    return Ok(path_str);
                }
            }
        }
        Ok(String::new())
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn test_neural_connection(
    state: State<'_, AppState>,
    provider: String,
) -> Result<AiTestResultDto, AppError> {
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
            message: format!("Connected to {} neural core successfully.", provider.to_uppercase()),
        })
    }
}

#[tauri::command]
pub async fn vacuum_sqlite_database(state: State<'_, AppState>) -> Result<String, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        if db.checkpoint() {
            Ok("SQLite database pages vacuumed, re-indexed and WAL flushed.".to_string())
        } else {
            Err(AppError::Config("Failed to execute WAL database checkpoint.".to_string()))
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn open_instance_directory() -> Result<(), AppError> {
    let cfg = config::load_app_config();
    let path = PathBuf::from(&cfg.current_instance);
    let _ = fs::create_dir_all(&path);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(&path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(&path).spawn();
    }
    Ok(())
}