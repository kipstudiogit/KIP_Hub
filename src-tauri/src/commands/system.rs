use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::System;
use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;
use crate::locales::Locales;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBreakdownDto {
    pub total_size_mb: f64,
    pub mods_size_mb: f64,
    pub saves_size_mb: f64,
    pub screenshots_size_mb: f64,
    pub logs_size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentWorldOverviewDto {
    pub name: String,
    pub last_played: String,
    pub mode: String,
    pub hardcore: bool,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardOverviewDto {
    pub size: String,
    pub saves: String,
    pub playtime: String,
    pub java: String,
    pub mods_count: usize,
    pub ram_usage_percent: u8,
    pub total_ram_gb: f64,
    pub used_ram_gb: f64,
    pub cpu_usage_percent: f32,
    pub cpu_brand: String,
    pub last_world: Option<RecentWorldOverviewDto>,
    pub latest_screenshot_thumbnail: Option<String>,
    pub storage: StorageBreakdownDto,
    pub is_game_running: bool,
    pub running_game_pid: Option<u32>,
    pub current_instance_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStatsDto {
    pub size: String,
    pub saves: String,
    pub playtime: String,
    pub java: String,
    pub mods_count: usize,
    pub ram_usage_percent: u8,
    pub last_world_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnalysisResponseDto {
    pub success: bool,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitDataDto {
    pub app_name: String,
    pub app_accent: String,
    pub version: String,
    pub greeting: String,
    pub plugins_js: Vec<String>,
    pub detected_java: String,
    pub active_instance: String,
    pub system_memory_gb: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateCheckResultDto {
    pub has_update: bool,
    pub version: Option<String>,
    pub body: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_dashboard_overview(state: State<'_, AppState>) -> Result<DashboardOverviewDto, AppError> {
    let db = state.db.clone();
    let is_mc_running = state.monitor.is_mc_running.load(Ordering::Relaxed);
    let active_pid = state.monitor.active_game_pid.load(Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir_str = cfg.current_instance.clone();
        let mc_dir = Path::new(&mc_dir_str);

        let mods_path = mc_dir.join("mods");
        let saves_path = mc_dir.join("saves");
        let screenshots_path = mc_dir.join("screenshots");
        let logs_path = mc_dir.join("logs");

        let total_size_mb = SystemUtils::get_dir_size_mb(&mc_dir_str);
        let mods_size_mb = SystemUtils::get_dir_size_mb(mods_path.to_str().unwrap_or(""));
        let saves_size_mb = SystemUtils::get_dir_size_mb(saves_path.to_str().unwrap_or(""));
        let screenshots_size_mb = SystemUtils::get_dir_size_mb(screenshots_path.to_str().unwrap_or(""));
        let logs_size_mb = SystemUtils::get_dir_size_mb(logs_path.to_str().unwrap_or(""));

        let mut mods_count = 0;
        if let Ok(entries) = fs::read_dir(&mods_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".jar") {
                    mods_count += 1;
                }
            }
        }

        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu_all();

        let total_bytes = sys.total_memory();
        let used_bytes = sys.used_memory();
        let total_ram_gb = (total_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let used_ram_gb = (used_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
        let ram_usage_percent = if total_bytes > 0 {
            ((used_bytes as f64 / total_bytes as f64) * 100.0).min(100.0) as u8
        } else {
            0
        };

        let cpu_usage_percent = sys.global_cpu_usage();
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Host CPU".to_string());

        let pt = db.get_play_time(&mc_dir_str);
        let playtime_str = format!("{}h {}m", pt / 3600, (pt % 3600) / 60);
        let java_str = SystemUtils::get_java_version();
        let saves_count = SystemUtils::get_saves_count(saves_path.to_str().unwrap_or(""));

        let mut last_world: Option<RecentWorldOverviewDto> = None;
        let mut newest_time = std::time::SystemTime::UNIX_EPOCH;

        if let Ok(entries) = fs::read_dir(&saves_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        if let Ok(mod_time) = meta.modified() {
                            if mod_time > newest_time {
                                newest_time = mod_time;
                                let world_path = entry.path();
                                let dirname = entry.file_name().to_string_lossy().to_string();
                                let info = crate::world_manager::WorldManager::get_world_info(
                                    world_path.to_str().unwrap_or(""),
                                );

                                let mut icon_b64 = String::new();
                                let icon_path = world_path.join("icon.png");
                                if icon_path.exists() {
                                    if let Ok(mut f) = File::open(&icon_path) {
                                        let mut buf = Vec::new();
                                        if f.read_to_end(&mut buf).is_ok() {
                                            icon_b64 = format!("data:image/png;base64,{}", BASE64.encode(&buf));
                                        }
                                    }
                                }

                                let datetime: chrono::DateTime<chrono::Local> = mod_time.into();
                                last_world = Some(RecentWorldOverviewDto {
                                    name: dirname,
                                    last_played: datetime.format("%Y-%m-%d %H:%M").to_string(),
                                    mode: info["mode"].as_str().unwrap_or("Survival").to_string(),
                                    hardcore: info["hardcore"].as_bool().unwrap_or(false),
                                    icon: icon_b64,
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut latest_screenshot_thumbnail: Option<String> = None;
        if screenshots_path.exists() {
            let mut screenshot_files: Vec<_> = fs::read_dir(&screenshots_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    n.ends_with(".png") || n.ends_with(".jpg") || n.ends_with(".jpeg")
                })
                .collect();

            screenshot_files.sort_by(|a, b| {
                let a_m = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_m = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                b_m.cmp(&a_m)
            });

            if let Some(first_shot) = screenshot_files.first() {
                if let Ok(data) = fs::read(first_shot.path()) {
                    if let Ok(img) = image::load_from_memory(&data) {
                        let thumb = img.thumbnail(320, 180);
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if thumb.write_to(&mut buf, image::ImageFormat::Jpeg).is_ok() {
                            latest_screenshot_thumbnail = Some(format!(
                                "data:image/jpeg;base64,{}",
                                BASE64.encode(buf.into_inner())
                            ));
                        }
                    }
                }
            }
        }

        let instance_name = Path::new(&mc_dir_str)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Default (.minecraft)".to_string());

        let size_formatted = if total_size_mb > 1024.0 {
            format!("{:.1} GB", total_size_mb / 1024.0)
        } else {
            format!("{:.0} MB", total_size_mb)
        };

        Ok(DashboardOverviewDto {
            size: size_formatted,
            saves: saves_count.to_string(),
            playtime: playtime_str,
            java: java_str,
            mods_count,
            ram_usage_percent,
            total_ram_gb: (total_ram_gb * 10.0).round() / 10.0,
            used_ram_gb: (used_ram_gb * 10.0).round() / 10.0,
            cpu_usage_percent: (cpu_usage_percent * 10.0).round() / 10.0,
            cpu_brand,
            last_world,
            latest_screenshot_thumbnail,
            storage: StorageBreakdownDto {
                total_size_mb: (total_size_mb * 10.0).round() / 10.0,
                mods_size_mb: (mods_size_mb * 10.0).round() / 10.0,
                saves_size_mb: (saves_size_mb * 10.0).round() / 10.0,
                screenshots_size_mb: (screenshots_size_mb * 10.0).round() / 10.0,
                logs_size_mb: (logs_size_mb * 10.0).round() / 10.0,
            },
            is_game_running: is_mc_running,
            running_game_pid: if active_pid > 0 { Some(active_pid) } else { None },
            current_instance_name: instance_name,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStatsDto, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = cfg.current_instance.clone();

        let size_mb = SystemUtils::get_dir_size_mb(&mc_dir);
        let size_str = if size_mb > 1024.0 {
            format!("{:.1} GB", size_mb / 1024.0)
        } else {
            format!("{:.0} MB", size_mb)
        };

        let saves_path = Path::new(&mc_dir).join("saves");
        let saves_count = SystemUtils::get_saves_count(saves_path.to_str().unwrap_or(""));

        let mut last_world_name: Option<String> = None;
        let mut newest_time = std::time::SystemTime::UNIX_EPOCH;

        if let Ok(entries) = fs::read_dir(&saves_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        if let Ok(mod_time) = meta.modified() {
                            if mod_time > newest_time {
                                newest_time = mod_time;
                                last_world_name = Some(entry.file_name().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        let mods_path = Path::new(&mc_dir).join("mods");
        let mut mods_count = 0;
        if let Ok(entries) = fs::read_dir(mods_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".jar") {
                    mods_count += 1;
                }
            }
        }

        let mut sys = System::new_all();
        sys.refresh_memory();
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let ram_usage_percent = if total_mem > 0 {
            ((used_mem as f64 / total_mem as f64) * 100.0).min(100.0) as u8
        } else {
            0
        };

        let pt = db.get_play_time(&mc_dir);
        let playtime_str = format!("{}h {}m", pt / 3600, (pt % 3600) / 60);
        let java_str = SystemUtils::get_java_version();

        Ok(DashboardStatsDto {
            size: size_str,
            saves: saves_count.to_string(),
            playtime: playtime_str,
            java: java_str,
            mods_count,
            ram_usage_percent,
            last_world_name,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn toggle_console_stream(state: State<'_, AppState>, active: bool) -> bool {
    state.monitor.console_streaming.store(active, Ordering::Relaxed);
    true
}

#[tauri::command]
pub async fn fetch_hub() -> Result<Vec<Value>, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()?;
    let target_url = format!("{}/api/presets", config::CLOUDFLARE_URL.trim_end_matches('/'));
    match client.get(&target_url).send().await {
        Ok(res) if res.status().is_success() => {
            let data: Value = res.json().await.unwrap_or(json!([]));
            Ok(data.as_array().cloned().unwrap_or_default())
        }
        _ => Ok(vec![
            json!({
                "id": "essential-fps",
                "title": "K.I.P. FPS Boost Pack",
                "author": "KIP Studio",
                "description": "Essential performance optimization mods stack.",
                "preset": ["sodium", "lithium", "ferrite-core", "entityculling"]
            }),
            json!({
                "id": "vanilla-plus",
                "title": "Vanilla Enhanced Experience",
                "author": "Community",
                "description": "Quality of life, visual enhancers, and fluid animations.",
                "preset": ["sodium", "iris", "modmenu", "appleskin", "ambient-sounds"]
            }),
        ]),
    }
}

#[tauri::command]
pub async fn publish_hub(
    title: String,
    author: String,
    desc: String,
    mods: Vec<String>,
) -> Result<bool, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let payload = json!({
        "type": "publish_preset",
        "title": ToolManager::scrub_personal_data(&title),
        "author": ToolManager::scrub_personal_data(&author),
        "description": ToolManager::scrub_personal_data(&desc),
        "preset": mods
    });

    let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
    match client.post(&target_url).json(&payload).send().await {
        Ok(res) => Ok(res.status().is_success()),
        Err(_) => Ok(true),
    }
}

#[tauri::command]
pub fn swarm_download(
    state: State<'_, AppState>,
    magnet: String,
    target_dir: String,
) -> Result<Value, AppError> {
    let cfg = config::load_app_config();
    let dest_dir = if target_dir == "MODS_DIR" {
        Path::new(&cfg.current_instance).join("mods")
    } else {
        let safe_sub = Path::new(&target_dir)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Path::new(&cfg.current_instance).join(safe_sub)
    };

    match state.swarm.download_magnet(&magnet, dest_dir.to_str().unwrap_or(""), |_, _| {}) {
        Ok(_) => Ok(json!({ "success": true })),
        Err(e) => Ok(json!({ "success": false, "msg": e })),
    }
}

#[tauri::command]
pub async fn send_bug_report(report_text: String) -> Result<bool, AppError> {
    let sys_info = get_sys_info().await?;
    let clean_report = ToolManager::scrub_personal_data(&report_text);
    let clean_sys_info = ToolManager::scrub_personal_data(&sys_info);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let payload = json!({
        "type": "bug_report",
        "report": clean_report,
        "telemetry": clean_sys_info,
        "version": config::APP_VERSION
    });

    let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
    let _ = client.post(&target_url).json(&payload).send().await;
    Ok(true)
}

#[tauri::command]
pub async fn get_console_logs() -> Result<String, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");
        if !log_path.exists() {
            return Ok("Log file not found. Launch the game first.".to_string());
        }
        let file = fs::File::open(&log_path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            lines.push(line);
            if lines.len() > 1000 {
                lines.remove(0);
            }
        }
        let start = lines.len().saturating_sub(200);
        Ok(lines[start..].join("\n"))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn get_sys_info() -> Result<String, AppError> {
    tokio::task::spawn_blocking(|| {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let total_bytes = sys.total_memory();
        let ram = if total_bytes > 0 {
            (total_bytes as f64) / (1024.0 * 1024.0 * 1024.0)
        } else {
            0.0
        };
        let ram_rounded = (ram * 10.0).round() / 10.0;
        let cpu = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());
        let os_name = sysinfo::System::name().unwrap_or_else(|| "OS".to_string());
        let os_ver = sysinfo::System::os_version().unwrap_or_default();
        let java_ver = SystemUtils::get_java_version();

        Ok(format!(
            "OS: {} {}\nCPU: {}\nGPU: Unknown\nRAM: {}GB\nJava: {}\nLauncher Ver: {}",
            os_name, os_ver, cpu, ram_rounded, java_ver, config::APP_VERSION
        ))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn analyze_crash_ai(
    state: State<'_, AppState>,
    log_snippet: String,
) -> Result<AiAnalysisResponseDto, AppError> {
    let clean_snippet = ToolManager::scrub_personal_data(&log_snippet);
    let answer = state.api.ask_ai_crash_analysis(&clean_snippet).await;
    let has_error = answer.contains("Error")
        || answer.contains("not configured")
        || answer.contains("Invalid");
    Ok(AiAnalysisResponseDto {
        success: !has_error,
        answer,
    })
}

#[tauri::command]
pub fn save_note(text: String) -> bool {
    let notes_path = config::get_app_data_dir().join("overlay_notes.txt");
    fs::write(notes_path, text).is_ok()
}

#[tauri::command]
pub fn get_note() -> String {
    let notes_path = config::get_app_data_dir().join("overlay_notes.txt");
    fs::read_to_string(notes_path).unwrap_or_default()
}

#[tauri::command]
pub fn get_init_data() -> InitDataDto {
    let cfg = config::load_app_config();
    let hour = chrono::Local::now()
        .format("%H")
        .to_string()
        .parse::<u32>()
        .unwrap_or(12);
    let greeting = if (5..12).contains(&hour) {
        "Good morning"
    } else if (12..17).contains(&hour) {
        "Good afternoon"
    } else if (17..23).contains(&hour) {
        "Good evening"
    } else {
        "Good night"
    };

    let mut sys = System::new_all();
    sys.refresh_memory();
    let ram_gb = (sys.total_memory() / (1024 * 1024 * 1024)) as i32;
    let java_ver = SystemUtils::get_java_version();

    InitDataDto {
        app_name: config::APP_NAME.to_string(),
        app_accent: " Hub".to_string(),
        version: config::APP_VERSION.to_string(),
        greeting: greeting.to_string(),
        plugins_js: Vec::new(),
        detected_java: java_ver,
        active_instance: cfg.current_instance,
        system_memory_gb: ram_gb,
    }
}

#[tauri::command]
pub fn get_translations(lang: String) -> HashMap<&'static str, &'static str> {
    Locales::get_translations(&lang)
}

#[tauri::command]
pub async fn check_app_update(app: AppHandle) -> Result<AppUpdateCheckResultDto, AppError> {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            return Ok(AppUpdateCheckResultDto {
                has_update: false,
                version: None,
                body: None,
                error: Some(e.to_string()),
            })
        }
    };

    match updater.check().await {
        Ok(Some(update)) => Ok(AppUpdateCheckResultDto {
            has_update: true,
            version: Some(update.version),
            body: update.body,
            error: None,
        }),
        Ok(None) => Ok(AppUpdateCheckResultDto {
            has_update: false,
            version: None,
            body: None,
            error: None,
        }),
        Err(e) => Ok(AppUpdateCheckResultDto {
            has_update: false,
            version: None,
            body: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn perform_app_update(app: AppHandle) -> Result<bool, AppError> {
    let updater = app
        .updater()
        .map_err(|e| AppError::Config(e.to_string()))?;

    if let Some(update) = updater.check().await.map_err(|e| AppError::Config(e.to_string()))? {
        let mut downloaded = 0;
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    let _ = downloaded;
                    let _ = content_length;
                },
                || (),
            )
            .await
            .map_err(|e| AppError::Config(e.to_string()))?;
        app.restart();
    }
    Ok(false)
}

#[tauri::command]
pub async fn pick_file() -> Result<String, AppError> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("powershell");
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg("Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Filter = 'Supported Packages (*.zip, *.mrpack, *.jar)|*.zip;*.mrpack;*.jar|All Files (*.*)|*.*'; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $f.FileName }");
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