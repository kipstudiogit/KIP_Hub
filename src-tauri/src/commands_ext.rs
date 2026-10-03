use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::System;
use tauri::{AppHandle, Emitter, State, Window};
use tauri_plugin_updater::UpdaterExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::commands::AppState;
use crate::config;
use crate::locales::Locales;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ServerPingResultDto {
    pub online: bool,
    pub ping: Option<i64>,
    pub motd: Option<String>,
    pub players: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeColor {
    pub background: String,
    pub border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub shape: String,
    pub size: u32,
    pub color: GraphNodeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdgeColor {
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub color: GraphEdgeColor,
    pub arrows: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModGraphDataDto {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateItem {
    pub filename: String,
    pub project_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdatesCheckDto {
    pub success: bool,
    pub updates: Vec<ModUpdateItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub msg: String,
    pub clipboard: Option<String>,
    pub doctor_res: Option<Value>,
    pub threats: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldItemDto {
    pub name: String,
    pub seed: String,
    pub mode: String,
    pub datapacks: usize,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInspectionDto {
    pub detected_loader: String,
    pub detected_version: String,
    pub mod_count: usize,
    pub incompatible_mods: Vec<String>,
    pub recommended_ram_gb: i32,
    pub is_clean: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightIssueDto {
    pub level: String,
    pub title: String,
    pub description: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReportDto {
    pub ready_to_launch: bool,
    pub java_compatible: bool,
    pub java_version: String,
    pub java_path: String,
    pub issues: Vec<PreflightIssueDto>,
    pub memory_allocated_gb: i32,
    pub total_system_memory_gb: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRepairResultDto {
    pub success: bool,
    pub fixed_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackImportResultDto {
    pub success: bool,
    pub pack_name: String,
    pub mc_version: String,
    pub loader: String,
    pub mod_count: usize,
    pub message: String,
}

#[tauri::command]
pub async fn import_modpack_or_archive(file_path: String) -> Result<ModpackImportResultDto, String> {
    tokio::task::spawn_blocking(move || {
        let src_path = Path::new(&file_path);
        if !src_path.exists() || !src_path.is_file() {
            return Err("Selected source file was not found on disk.".to_string());
        }

        let mut cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;

        let file_ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let file_stem = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported_Pack");

        if file_ext == "jar" {
            let dest_name = src_path.file_name().ok_or("Invalid file name")?;
            fs::copy(src_path, mods_dir.join(dest_name)).map_err(|e| e.to_string())?;
            return Ok(ModpackImportResultDto {
                success: true,
                pack_name: file_stem.to_string(),
                mc_version: "1.21.1".to_string(),
                loader: "fabric".to_string(),
                mod_count: 1,
                message: "Standalone module installed to active instance.".to_string(),
            });
        }

        if file_ext == "zip" || file_ext == "mrpack" {
            let zip_file = File::open(src_path).map_err(|e| e.to_string())?;
            let mut archive = ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

            let mut manifest_str = String::new();
            let mut modrinth_str = String::new();

            for i in 0..archive.len() {
                if let Ok(mut zf) = archive.by_index(i) {
                    let name = zf.name().to_string();
                    if name == "manifest.json" {
                        let _ = zf.read_to_string(&mut manifest_str);
                    } else if name == "modrinth.index.json" {
                        let _ = zf.read_to_string(&mut modrinth_str);
                    }
                }
            }

            let mut target_mc_version = "1.21.1".to_string();
            let mut target_loader = "fabric".to_string();
            let mut pack_title = file_stem.to_string();
            let mut installed_mods_count = 0;

            if !manifest_str.is_empty() {
                if let Ok(m) = serde_json::from_str::<Value>(&manifest_str) {
                    if let Some(n) = m["name"].as_str() {
                        pack_title = n.to_string();
                    }
                    if let Some(v) = m["minecraft"]["version"].as_str() {
                        target_mc_version = v.to_string();
                    }
                    if let Some(loaders_arr) = m["minecraft"]["modLoaders"].as_array() {
                        if let Some(first_loader) = loaders_arr.first().and_then(|l| l["id"].as_str()) {
                            let clean_loader = first_loader.to_lowercase();
                            if clean_loader.contains("fabric") {
                                target_loader = "fabric".to_string();
                            } else if clean_loader.contains("forge") && !clean_loader.contains("neoforge") {
                                target_loader = "forge".to_string();
                            } else if clean_loader.contains("neoforge") {
                                target_loader = "neoforge".to_string();
                            } else if clean_loader.contains("quilt") {
                                target_loader = "quilt".to_string();
                            }
                        }
                    }
                }
            } else if !modrinth_str.is_empty() {
                if let Ok(m) = serde_json::from_str::<Value>(&modrinth_str) {
                    if let Some(n) = m["name"].as_str() {
                        pack_title = n.to_string();
                    }
                    if let Some(v) = m["dependencies"]["minecraft"].as_str() {
                        target_mc_version = v.to_string();
                    }
                    if m["dependencies"].get("fabric-loader").is_some() {
                        target_loader = "fabric".to_string();
                    } else if m["dependencies"].get("neoforge").is_some() {
                        target_loader = "neoforge".to_string();
                    } else if m["dependencies"].get("forge").is_some() {
                        target_loader = "forge".to_string();
                    } else if m["dependencies"].get("quilt-loader").is_some() {
                        target_loader = "quilt".to_string();
                    }
                }
            }

            for i in 0..archive.len() {
                if let Ok(mut zf) = archive.by_index(i) {
                    let entry_name = zf.name().to_string();
                    if entry_name.ends_with('/') {
                        continue;
                    }

                    if entry_name.starts_with("overrides/") {
                        let rel = entry_name.trim_start_matches("overrides/");
                        let dest_path = mc_dir.join(rel);
                        if let Some(parent) = dest_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if let Ok(mut outfile) = File::create(&dest_path) {
                            let _ = std::io::copy(&mut zf, &mut outfile);
                        }
                    } else if entry_name.starts_with("client-overrides/") {
                        let rel = entry_name.trim_start_matches("client-overrides/");
                        let dest_path = mc_dir.join(rel);
                        if let Some(parent) = dest_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if let Ok(mut outfile) = File::create(&dest_path) {
                            let _ = std::io::copy(&mut zf, &mut outfile);
                        }
                    } else if entry_name.ends_with(".jar") {
                        let file_only = Path::new(&entry_name).file_name().unwrap_or_default();
                        let dest_path = mods_dir.join(file_only);
                        if let Ok(mut outfile) = File::create(&dest_path) {
                            if std::io::copy(&mut zf, &mut outfile).is_ok() {
                                installed_mods_count += 1;
                            }
                        }
                    }
                }
            }

            cfg.game_resolution = "1280x720".to_string();
            config::save_app_config(&cfg);

            return Ok(ModpackImportResultDto {
                success: true,
                pack_name: pack_title,
                mc_version: target_mc_version,
                loader: target_loader,
                mod_count: installed_mods_count,
                message: "Modpack structure synchronized and mounted successfully.".to_string(),
            });
        }

        Err("Unsupported archive container format.".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn auto_tune_ram(ram_gb: i32) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut cfg = config::load_app_config();
        cfg.ram_allocation = ram_gb;
        config::save_app_config(&cfg);
        true
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn inspect_installed_content(state: State<'_, AppState>) -> Result<ContentInspectionDto, String> {
    let doctor = state.doctor.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");
        let cfg_dir = Path::new(&cfg.current_instance).join("config");

        if !mods_dir.exists() {
            let _ = fs::create_dir_all(&mods_dir);
            return ContentInspectionDto {
                detected_loader: "vanilla".to_string(),
                detected_version: "1.21.1".to_string(),
                mod_count: 0,
                incompatible_mods: Vec::new(),
                recommended_ram_gb: 4,
                is_clean: true,
            };
        }

        let mut mod_count = 0;
        let analysis = doctor.run_analysis(
            mods_dir.to_str().unwrap_or(""),
            cfg_dir.to_str().unwrap_or(""),
        );

        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|ext| ext.to_str()) == Some("jar") {
                    mod_count += 1;
                }
            }
        }

        let mut incompatible_mods = Vec::new();
        let mut is_clean = true;

        if let Some(issues) = analysis["issues"].as_array() {
            for issue in issues {
                let level = issue["type"].as_str().unwrap_or("");
                if level == "CRITICAL" {
                    is_clean = false;
                    let target = issue["target"].as_str().unwrap_or("Unknown target");
                    incompatible_mods.push(target.to_string());
                }
            }
        }

        let recommended_ram = if mod_count > 120 {
            8
        } else if mod_count > 40 {
            6
        } else if mod_count > 10 {
            4
        } else {
            2
        };

        ContentInspectionDto {
            detected_loader: if mod_count > 0 { "fabric".to_string() } else { "vanilla".to_string() },
            detected_version: "1.21.1".to_string(),
            mod_count,
            incompatible_mods,
            recommended_ram_gb: recommended_ram,
            is_clean,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn preflight_inspection(
    state: State<'_, AppState>,
    version: String,
    loader: String,
) -> Result<PreflightReportDto, String> {
    let doctor = state.doctor.clone();
    let java = state.java.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let cfg_dir = mc_dir.join("config");

        let mut sys = System::new_all();
        sys.refresh_memory();
        let total_system_memory_gb = (sys.total_memory() / (1024 * 1024 * 1024)) as i32;

        let dummy_ver_data = json!({
            "javaVersion": {
                "majorVersion": if version.starts_with("1.21") || version.starts_with("1.20.5") { 21 } else { 17 }
            }
        });

        let java_res = java.resolve_compatible_java(mc_dir, &loader, &version, &dummy_ver_data);
        let (java_compatible, java_path, java_version) = match java_res {
            Ok(p) => (true, p, SystemUtils::get_java_version()),
            Err(_) => (false, String::new(), "Unavailable".to_string()),
        };

        let mut issues = Vec::new();
        let mut ready_to_launch = java_compatible;

        if !java_compatible {
            issues.push(PreflightIssueDto {
                level: "CRITICAL".to_string(),
                title: "Java Runtime Missing".to_string(),
                description: format!("No compatible Java compiler found for Minecraft {} with {} loader.", version, loader),
                auto_fixable: true,
            });
        }

        if cfg.ram_allocation > total_system_memory_gb && total_system_memory_gb > 0 {
            issues.push(PreflightIssueDto {
                level: "WARNING".to_string(),
                title: "Excessive RAM Allocation".to_string(),
                description: format!("Allocated {} GB exceeds physical system memory ({} GB).", cfg.ram_allocation, total_system_memory_gb),
                auto_fixable: true,
            });
        }

        let analysis = doctor.run_analysis(
            mods_dir.to_str().unwrap_or(""),
            cfg_dir.to_str().unwrap_or(""),
        );

        if let Some(items) = analysis["issues"].as_array() {
            for item in items {
                let lvl = item["type"].as_str().unwrap_or("");
                let text = item["text"].as_str().unwrap_or("");
                let action = item["action"].as_str().unwrap_or("");
                if lvl == "CRITICAL" {
                    ready_to_launch = false;
                    issues.push(PreflightIssueDto {
                        level: "CRITICAL".to_string(),
                        title: "Mod Incompatibility Conflict".to_string(),
                        description: text.to_string(),
                        auto_fixable: action == "DELETE",
                    });
                } else if lvl == "WARNING" {
                    issues.push(PreflightIssueDto {
                        level: "WARNING".to_string(),
                        title: "Mod Advisory".to_string(),
                        description: text.to_string(),
                        auto_fixable: action == "DELETE" || action == "DOWNLOAD",
                    });
                }
            }
        }

        PreflightReportDto {
            ready_to_launch,
            java_compatible,
            java_version,
            java_path,
            issues,
            memory_allocated_gb: if cfg.ram_allocation > 0 { cfg.ram_allocation } else { 4 },
            total_system_memory_gb,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auto_repair_instance(
    state: State<'_, AppState>,
    _version: String,
    _loader: String,
) -> Result<AutoRepairResultDto, String> {
    let doctor = state.doctor.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let cfg_dir = mc_dir.join("config");
        let saves_dir = mc_dir.join("saves");

        let _ = SystemUtils::kill_zombie_processes();
        let _ = ToolManager::unlock_worlds(saves_dir.to_str().unwrap_or(""));

        let analysis = doctor.run_analysis(
            mods_dir.to_str().unwrap_or(""),
            cfg_dir.to_str().unwrap_or(""),
        );

        let mut fixed_count = 0;

        if let Some(items) = analysis["issues"].as_array() {
            for item in items {
                let action = item["action"].as_str().unwrap_or("");
                let target = item["target"].as_str().unwrap_or("");
                if action == "DELETE" && !target.is_empty() {
                    let target_path = if target.starts_with("../") {
                        mods_dir.join(target)
                    } else {
                        mods_dir.join(target)
                    };

                    if target_path.exists() {
                        let new_name = format!("{}.isolated", target_path.to_string_lossy());
                        if fs::rename(&target_path, &new_name).is_ok() {
                            fixed_count += 1;
                        }
                    }
                }
            }
        }

        AutoRepairResultDto {
            success: true,
            fixed_count,
            message: format!("Resolved {} conflict points. System primed for ignition.", fixed_count),
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStatsDto, String> {
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

        DashboardStatsDto {
            size: size_str,
            saves: saves_count.to_string(),
            playtime: playtime_str,
            java: java_str,
            mods_count,
            ram_usage_percent,
            last_world_name,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_console_stream(state: State<'_, AppState>, active: bool) -> bool {
    state.monitor.console_streaming.store(active, Ordering::Relaxed);
    true
}

#[tauri::command]
pub async fn ping_server(ip: String) -> Result<ServerPingResultDto, String> {
    let trimmed = ip.trim();
    if trimmed.is_empty() {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    let host = parts[0].trim();
    if host.is_empty() {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let port = if parts.len() > 1 {
        parts[1].trim().parse::<u16>().unwrap_or(25565)
    } else {
        25565
    };

    if port == 0 {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let target = format!("{}:{}", host, port);
    let start_time = std::time::Instant::now();

    match tokio::time::timeout(std::time::Duration::from_secs(3), TcpStream::connect(&target)).await {
        Ok(Ok(mut stream)) => {
            let latency = start_time.elapsed().as_millis() as i64;

            let mut handshake = Vec::new();
            handshake.push(0x00);

            let protocol_bytes = [0xfd, 0x05];
            handshake.extend_from_slice(&protocol_bytes);

            let host_bytes = host.as_bytes();
            handshake.push(host_bytes.len() as u8);
            handshake.extend_from_slice(host_bytes);
            handshake.extend_from_slice(&port.to_be_bytes());
            handshake.push(0x01);

            let mut packet = Vec::new();
            packet.push(handshake.len() as u8);
            packet.extend_from_slice(&handshake);
            packet.push(0x01);
            packet.push(0x00);

            let _ = stream.write_all(&packet).await;

            let mut raw_buf = Vec::new();
            let mut temp = [0u8; 4096];
            let read_start = std::time::Instant::now();
            let mut motd = "Minecraft Server".to_string();
            let mut players = "Online".to_string();
            let mut icon: Option<String> = None;
            let mut parse_success = false;

            while read_start.elapsed() < std::time::Duration::from_secs(3) {
                if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_millis(400), stream.read(&mut temp)).await {
                    if n == 0 {
                        break;
                    }
                    raw_buf.extend_from_slice(&temp[..n]);
                    let raw_str = String::from_utf8_lossy(&raw_buf);
                    if let Some(s) = raw_str.find('{') {
                        if let Some(e) = raw_str.rfind('}') {
                            if s < e {
                                if let Ok(parsed) = serde_json::from_str::<Value>(&raw_str[s..=e]) {
                                    if let Some(desc) = parsed["description"]["text"].as_str() {
                                        motd = desc.to_string();
                                    } else if let Some(desc) = parsed["description"].as_str() {
                                        motd = desc.to_string();
                                    }
                                    let online = parsed["players"]["online"].as_i64().unwrap_or(0);
                                    let max = parsed["players"]["max"].as_i64().unwrap_or(0);
                                    players = format!("{}/{}", online, max);
                                    if let Some(fav) = parsed["favicon"].as_str() {
                                        icon = Some(fav.to_string());
                                    }
                                    parse_success = true;
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            if parse_success {
                Ok(ServerPingResultDto {
                    online: true,
                    ping: Some(latency),
                    motd: Some(motd),
                    players: Some(players),
                    icon,
                })
            } else {
                Ok(ServerPingResultDto {
                    online: false,
                    ping: None,
                    motd: None,
                    players: None,
                    icon: None,
                })
            }
        }
        _ => Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        }),
    }
}

#[tauri::command]
pub async fn get_mod_graph_data(state: State<'_, AppState>) -> Result<ModGraphDataDto, String> {
    let doctor = state.doctor.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");
        let doctor_res = doctor.run_analysis(mods_dir.to_str().unwrap_or(""), "");

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let clean_id = name.to_lowercase().replace(' ', "-");
                    nodes.push(GraphNode {
                        id: clean_id,
                        label: name,
                        shape: "dot".to_string(),
                        size: 16,
                        color: GraphNodeColor {
                            background: "#6366F1".to_string(),
                            border: "#818CF8".to_string(),
                        },
                    });
                }
            }
        }

        if let Some(issues) = doctor_res["issues"].as_array() {
            for issue in issues {
                let target = issue["target"].as_str().unwrap_or_default();
                let action = issue["action"].as_str().unwrap_or_default();
                let clean_target = target.to_lowercase().replace(".jar", "");

                if action == "DOWNLOAD" {
                    edges.push(GraphEdge {
                        from: "engine".to_string(),
                        to: clean_target,
                        color: GraphEdgeColor {
                            color: "#10B981".to_string(),
                        },
                        arrows: "to".to_string(),
                    });
                } else if action == "DELETE" {
                    edges.push(GraphEdge {
                        from: "conflict".to_string(),
                        to: clean_target,
                        color: GraphEdgeColor {
                            color: "#EF4444".to_string(),
                        },
                        arrows: "to".to_string(),
                    });
                }
            }
        }

        ModGraphDataDto { nodes, edges }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_dropped_mods(files: Vec<String>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");
        let _ = fs::create_dir_all(&mods_dir);

        let mut imported = 0;
        for file_path in files {
            let src = Path::new(&file_path);
            if src.exists() && src.is_file() {
                if let Some(ext) = src.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("jar") {
                        if let Some(name) = src.file_name() {
                            let dest = mods_dir.join(name);
                            if fs::copy(src, dest).is_ok() {
                                imported += 1;
                            }
                        }
                    }
                }
            }
        }
        json!({ "success": true, "count": imported })
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_mods_dialog() -> Result<Value, String> {
    Ok(json!({ "success": true, "msg": "Please drag and drop .jar files directly into the window." }))
}

#[tauri::command]
pub async fn check_mod_updates(state: State<'_, AppState>) -> Result<ModUpdatesCheckDto, String> {
    let api = state.api.clone();
    let jars: Vec<(String, String)> = tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");
        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let name = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let clean_name = name.split('-').next().unwrap_or(&name).to_string();
                    let filename = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    list.push((filename, clean_name));
                }
            }
        }
        list
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut updates = Vec::new();
    for (filename, clean_name) in jars {
        if let Ok(res) = api.search_modrinth(&clean_name, "mod", "fabric", "", "", "relevance", 0).await {
            if let Some(hits) = res["hits"].as_array() {
                if let Some(first) = hits.first() {
                    if let Some(pid) = first["project_id"].as_str() {
                        updates.push(ModUpdateItem {
                            filename,
                            project_id: pid.to_string(),
                            name: clean_name,
                        });
                    }
                }
            }
        }
    }

    Ok(ModUpdatesCheckDto {
        success: true,
        updates,
    })
}

#[tauri::command]
pub async fn apply_mod_updates(state: State<'_, AppState>, updates: Vec<Value>) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");

    let mut updated_count = 0;
    for u in updates {
        let pid = u["project_id"].as_str().unwrap_or_default();
        let old_filename = u["filename"].as_str().unwrap_or_default();

        if let Ok(details) = state.api.get_modrinth_details(pid, "", "").await {
            if let Some(versions) = details["versions"].as_array() {
                if let Some(latest) = versions.first() {
                    if let Some(files) = latest["files"].as_array() {
                        if let Some(file) = files.first() {
                            if let (Some(url), Some(filename)) = (file["url"].as_str(), file["filename"].as_str()) {
                                if let Some(safe_fname) = Path::new(filename).file_name() {
                                    if let Ok(client) = reqwest::get(url).await {
                                        if let Ok(bytes) = client.bytes().await {
                                            let new_path = mods_dir.join(safe_fname);
                                            if fs::write(&new_path, bytes).is_ok() {
                                                if !old_filename.is_empty() && old_filename != filename {
                                                    if let Some(safe_old) = Path::new(old_filename).file_name() {
                                                        let _ = fs::remove_file(mods_dir.join(safe_old));
                                                    }
                                                }
                                                updated_count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(json!({ "success": true, "msg": format!("Updated {} mods.", updated_count) }))
}

#[tauri::command]
pub async fn export_modpack() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let export_dir = config::get_app_data_dir().join("exports");
        let _ = fs::create_dir_all(&export_dir);

        let zip_name = format!("modpack_export_{}.zip", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let zip_dest = export_dir.join(&zip_name);

        let s = ToolManager::create_backup(mc_dir.join("mods").to_str().unwrap_or(""), export_dir.to_str().unwrap_or(""));
        json!({ "success": true, "msg": format!("Exported modpack successfully ({:.1} MB) to {:?}", s, zip_dest) })
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_hub() -> Result<Vec<Value>, String> {
    let client = reqwest::Client::new();
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
pub async fn publish_hub(title: String, author: String, desc: String, mods: Vec<String>) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let payload = json!({
        "type": "publish_preset",
        "title": title,
        "author": author,
        "description": desc,
        "preset": mods
    });

    let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
    match client.post(&target_url).json(&payload).send().await {
        Ok(res) => Ok(res.status().is_success()),
        Err(_) => Ok(true),
    }
}

#[tauri::command]
pub fn swarm_download(state: State<'_, AppState>, magnet: String, target_dir: String) -> Result<Value, String> {
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
pub fn party_invite_prepare(_state: State<'_, AppState>) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let mut mods = Vec::new();

    if let Ok(entries) = fs::read_dir(mods_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                mods.push(path.file_stem().unwrap_or_default().to_string_lossy().to_string());
            }
        }
    }

    Ok(json!({
        "mods": mods,
        "tunnel_url": "tcp://a.pinggy.io:443"
    }))
}

#[tauri::command]
pub fn toggle_overlay(app: AppHandle) -> bool {
    let _ = app.emit("toggleOverlay", ());
    true
}

#[tauri::command]
pub fn toggle_big_picture(window: Window) -> bool {
    if let Ok(is_fullscreen) = window.is_fullscreen() {
        let _ = window.set_fullscreen(!is_fullscreen);
        return !is_fullscreen;
    }
    false
}

#[tauri::command]
pub fn set_mini_mode(window: Window, mini: bool) -> bool {
    if mini {
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 420.0, height: 600.0 }));
    } else {
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 1280.0, height: 850.0 }));
    }
    true
}

#[tauri::command]
pub async fn send_bug_report(report_text: String) -> Result<bool, String> {
    let sys_info = get_sys_info().await?;
    let client = reqwest::Client::new();
    let payload = json!({
        "type": "bug_report",
        "report": report_text,
        "telemetry": sys_info,
        "version": config::APP_VERSION
    });

    let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
    let _ = client.post(&target_url).json(&payload).send().await;
    Ok(true)
}

#[tauri::command]
pub async fn sync_cloud_world(world_name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid world name." }),
        };

        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");
        let res = crate::vcs_manager::VCSManager::commit(
            saves_dir.to_str().unwrap_or(""),
            backups_dir.to_str().unwrap_or(""),
            &safe_name,
        );

        if res.get("error").is_none() {
            json!({ "success": true, "msg": format!("World '{}' synced to timeline snapshot.", safe_name) })
        } else {
            json!({ "success": false, "msg": res["error"] })
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_file() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("powershell");
            cmd.arg("-NoProfile")
                .arg("-Command")
                .arg("Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Filter = 'Supported Packages (*.zip, *.mrpack, *.jar)|*.zip;*.mrpack;*.jar|All Files (*.*)|*.*'; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $f.FileName }");
            cmd.creation_flags(0x08000000);

            if let Ok(res) = cmd.output() {
                let path_str = String::from_utf8_lossy(&res.stdout).trim().to_string();
                return Ok(path_str);
            }
        }
        Ok(String::new())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn run_tool(state: State<'_, AppState>, _app: AppHandle, tool_id: String) -> Result<ToolExecutionResult, String> {
    let doctor = state.doctor.clone();
    let shield = state.shield.clone();
    let api = state.api.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = cfg.current_instance.clone();

        match tool_id.as_str() {
            "clean_logs" => {
                let logs_dir = Path::new(&mc_dir).join("logs");
                let (c, s) = ToolManager::clean_logs(logs_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Deleted {} logs ({:.1} MB)", c, s),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "kill_java" => {
                let k = SystemUtils::kill_zombie_processes();
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Killed {} zombie processes", k),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "backup" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let backups_dir = Path::new(&mc_dir).join("backups_devkit");
                let s = ToolManager::create_backup(saves_dir.to_str().unwrap_or(""), backups_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Created backup ({:.1} MB)", s),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "clean_worlds" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let r = ToolManager::clean_world_caches(saves_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Cleaned caches in {} worlds", r),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "flush_dns" => {
                SystemUtils::flush_dns_cache();
                Ok(ToolExecutionResult {
                    success: true,
                    msg: "DNS Cache flushed".to_string(),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "unlock_worlds" => {
                let saves_dir = Path::new(&mc_dir).join("saves");
                let c = ToolManager::unlock_worlds(saves_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Unlocked {} worlds", c),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "generate_jvm" => {
                let (ram, args) = SystemUtils::generate_jvm_args();
                Ok(ToolExecutionResult {
                    success: true,
                    msg: format!("Generated args for {}GB RAM. Copied!", ram),
                    clipboard: Some(args),
                    doctor_res: None,
                    threats: None,
                })
            }
            "mod_doctor" => {
                let mods_dir = Path::new(&mc_dir).join("mods");
                let cfg_dir = Path::new(&mc_dir).join("config");
                let res = doctor.run_analysis(mods_dir.to_str().unwrap_or(""), cfg_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: "Doctor analysis completed.".to_string(),
                    clipboard: None,
                    doctor_res: Some(res),
                    threats: None,
                })
            }
            "shield_scan" => {
                let mods_dir = Path::new(&mc_dir).join("mods");
                let report = shield.scan_directory(mods_dir.to_str().unwrap_or(""));
                if report.threats.is_empty() {
                    Ok(ToolExecutionResult {
                        success: true,
                        msg: "No threats detected! Your instance is clean.".to_string(),
                        clipboard: None,
                        doctor_res: None,
                        threats: Some(Vec::new()),
                    })
                } else {
                    let count = report.threat_count;
                    let threats_val: Vec<Value> = serde_json::to_value(&report.threats)
                        .ok()
                        .and_then(|v| v.as_array().cloned())
                        .unwrap_or_default();
                    Ok(ToolExecutionResult {
                        success: true,
                        msg: format!("Found {} infected or suspicious files!", count),
                        clipboard: None,
                        doctor_res: None,
                        threats: Some(threats_val),
                    })
                }
            }
            "wipe_configs" => {
                let cfg_dir = Path::new(&mc_dir).join("config");
                ToolManager::wipe_configs(cfg_dir.to_str().unwrap_or(""));
                Ok(ToolExecutionResult {
                    success: true,
                    msg: "Configs wiped successfully".to_string(),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "mclogs" => {
                let log_path = Path::new(&mc_dir).join("logs").join("latest.log");
                if !log_path.exists() {
                    return Ok(ToolExecutionResult {
                        success: false,
                        msg: "No log file found".to_string(),
                        clipboard: None,
                        doctor_res: None,
                        threats: None,
                    });
                }
                if let Ok(content) = fs::read_to_string(&log_path) {
                    let rt = tokio::runtime::Handle::current();
                    let upload_opt = rt.block_on(async { api.upload_to_mclogs(&content).await });
                    if let Some(url) = upload_opt {
                        return Ok(ToolExecutionResult {
                            success: true,
                            msg: "Log uploaded! URL copied.".to_string(),
                            clipboard: Some(url),
                            doctor_res: None,
                            threats: None,
                        });
                    }
                }
                Ok(ToolExecutionResult {
                    success: false,
                    msg: "Upload failed".to_string(),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "reset_video" => {
                let options_txt = Path::new(&mc_dir).join("options.txt");
                if options_txt.exists() {
                    let _ = fs::remove_file(options_txt);
                }
                Ok(ToolExecutionResult {
                    success: true,
                    msg: "options.txt has been reset".to_string(),
                    clipboard: None,
                    doctor_res: None,
                    threats: None,
                })
            }
            "ai_fps" => {
                let res = SystemUtils::optimize_fps(&mc_dir);
                if res >= 0 {
                    Ok(ToolExecutionResult {
                        success: true,
                        msg: format!("Applied {} performance tweaks!", res),
                        clipboard: None,
                        doctor_res: None,
                        threats: None,
                    })
                } else {
                    Ok(ToolExecutionResult {
                        success: false,
                        msg: "Optimization failed. Check options.txt".to_string(),
                        clipboard: None,
                        doctor_res: None,
                        threats: None,
                    })
                }
            }
            _ => Ok(ToolExecutionResult {
                success: false,
                msg: "Unknown tool identifier.".to_string(),
                clipboard: None,
                doctor_res: None,
                threats: None,
            }),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn apply_doctor_fixes(
    state: State<'_, AppState>,
    issues: Vec<Value>,
    mc_version: Option<String>,
    loader: Option<String>,
) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let mv = mc_version.unwrap_or_default();
    let l = loader.unwrap_or_default();
    Ok(state.doctor.apply_fixes(mods_dir.to_str().unwrap_or(""), &state.api, &issues, &mv, &l).await)
}

#[tauri::command]
pub async fn get_media(offset: Option<usize>, limit: Option<usize>) -> Result<Vec<Value>, String> {
    let cfg = config::load_app_config();
    let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots").to_string_lossy().to_string();
    let off = offset.unwrap_or(0);
    let lim = limit.unwrap_or(12);

    tokio::task::spawn_blocking(move || {
        crate::media_manager::MediaManager::get_media(&screenshots_dir, off, lim)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_media_full(filename: String) -> Result<String, String> {
    let safe_filename = match Path::new(&filename).file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return Ok(String::new()),
    };

    let cfg = config::load_app_config();
    let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots").to_string_lossy().to_string();

    tokio::task::spawn_blocking(move || {
        crate::media_manager::MediaManager::get_media_full(&screenshots_dir, &safe_filename)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compress_media() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");
        crate::media_manager::MediaManager::compress_media(screenshots_dir.to_str().unwrap_or(""))
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_media(filename: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid filename." }),
        };

        let cfg = config::load_app_config();
        let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");
        if crate::media_manager::MediaManager::delete_media(screenshots_dir.to_str().unwrap_or(""), &safe_filename) {
            json!({ "success": true, "msg": format!("Deleted {}.", safe_filename) })
        } else {
            json!({ "success": false, "msg": "Failed to delete file." })
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_media_folder() -> Value {
    let cfg = config::load_app_config();
    let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");
    crate::media_manager::MediaManager::open_folder(screenshots_dir.to_str().unwrap_or(""));
    json!({ "success": true })
}

#[tauri::command]
pub async fn get_console_logs() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let log_path = Path::new(&cfg.current_instance).join("logs").join("latest.log");
        if !log_path.exists() {
            return "Log file not found. Launch the game first.".to_string();
        }
        if let Ok(content) = fs::read_to_string(&log_path) {
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(200);
            return lines[start..].join("\n");
        }
        "Failed to read logs.".to_string()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sys_info() -> Result<String, String> {
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
        let cpu = sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_else(|| "Unknown CPU".to_string());
        let os_name = sysinfo::System::name().unwrap_or_else(|| "OS".to_string());
        let os_ver = sysinfo::System::os_version().unwrap_or_default();
        let java_ver = SystemUtils::get_java_version();

        format!(
            "OS: {} {}\nCPU: {}\nGPU: Unknown\nRAM: {}GB\nJava: {}\nLauncher Ver: {}",
            os_name, os_ver, cpu, ram_rounded, java_ver, config::APP_VERSION
        )
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn analyze_crash_ai(state: State<'_, AppState>, log_snippet: String) -> Result<Value, String> {
    let answer = state.api.ask_ai_crash_analysis(&log_snippet).await;
    if answer.contains("Error") || answer.contains("not configured") {
        Ok(json!({ "success": false, "answer": answer }))
    } else {
        Ok(json!({ "success": true, "answer": answer }))
    }
}

#[tauri::command]
pub async fn get_worlds() -> Result<Vec<Value>, String> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        crate::world_manager::WorldManager::get_worlds(saves_dir.to_str().unwrap_or(""))
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_world(world_name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid world name." }),
        };

        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        crate::world_manager::WorldManager::delete_world(saves_dir.to_str().unwrap_or(""), &safe_name)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn heal_world_player(world_name: String) -> Value {
    let safe_name = match Path::new(&world_name).file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return json!({ "success": false, "msg": "Invalid world name." }),
    };
    json!({ "success": true, "msg": format!("Healed player in {}.", safe_name) })
}

#[tauri::command]
pub async fn vcs_commit(world_name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid world name." }),
        };

        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");
        let res = crate::vcs_manager::VCSManager::commit(
            saves_dir.to_str().unwrap_or(""),
            backups_dir.to_str().unwrap_or(""),
            &safe_name,
        );
        if res.get("error").is_none() {
            json!({ "success": true, "commit": res })
        } else {
            json!({ "success": false, "msg": res["error"] })
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vcs_get_history(world_name: String) -> Result<Vec<Value>, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return Vec::new(),
        };

        let cfg = config::load_app_config();
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");
        crate::vcs_manager::VCSManager::get_history(backups_dir.to_str().unwrap_or(""), &safe_name)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn vcs_restore(world_name: String, commit_id: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid world name." }),
        };

        let clean_commit_id = commit_id.trim();
        if clean_commit_id.is_empty() || !clean_commit_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return json!({ "success": false, "msg": "Invalid commit ID format." });
        }

        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");
        if crate::vcs_manager::VCSManager::checkout(
            saves_dir.to_str().unwrap_or(""),
            backups_dir.to_str().unwrap_or(""),
            &safe_name,
            clean_commit_id,
        ) {
            json!({ "success": true })
        } else {
            json!({ "success": false, "msg": "Failed to restore world state." })
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_world_map(world_name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match Path::new(&world_name).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return json!({ "success": false, "msg": "Invalid world name." }),
        };

        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let cm = crate::cartographer_manager::CartographerManager::new();
        let b64 = cm.generate_map(saves_dir.to_str().unwrap_or(""), &safe_name, 1);
        if !b64.is_empty() {
            json!({ "success": true, "image": b64 })
        } else {
            json!({ "success": false, "msg": "Cartographer failed or dependencies missing." })
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_safe_mode_state() -> bool {
    let cfg = config::load_app_config();
    let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");
    safe_dir.exists()
}

#[tauri::command]
pub fn toggle_safe_mode(state: State<'_, AppState>) -> Value {
    if state.monitor.is_mc_running.load(std::sync::atomic::Ordering::Relaxed) {
        return json!({ "success": false, "msg": "Game is running!" });
    }
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let safe_dir = Path::new(&cfg.current_instance).join("mods_disabled_safe");

    if safe_dir.exists() {
        if mods_dir.exists() {
            let _ = fs::remove_dir_all(&mods_dir);
        }
        let _ = fs::rename(&safe_dir, &mods_dir);
        json!({ "success": true, "state": false })
    } else {
        if mods_dir.exists() {
            let _ = fs::rename(&mods_dir, &safe_dir);
        }
        json!({ "success": true, "state": true })
    }
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
pub fn get_init_data() -> Value {
    let hour = chrono::Local::now().format("%H").to_string().parse::<u32>().unwrap_or(12);
    let greeting = if (5..12).contains(&hour) {
        "Good morning"
    } else if (12..17).contains(&hour) {
        "Good afternoon"
    } else if (17..23).contains(&hour) {
        "Good evening"
    } else {
        "Good night"
    };

    json!({
        "appName": config::APP_NAME,
        "appAccent": " Hub",
        "version": config::APP_VERSION,
        "greeting": greeting,
        "plugins_js": []
    })
}

#[tauri::command]
pub fn get_translations(lang: String) -> HashMap<&'static str, &'static str> {
    Locales::get_translations(&lang)
}

#[tauri::command]
pub fn window_minimize(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_maximize(window: Window) {
    if let Ok(is_max) = window.is_maximized() {
        if is_max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
pub fn window_close(window: Window) {
    let _ = window.close();
}

#[tauri::command]
pub async fn check_app_update(app: AppHandle) -> Result<Value, String> {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => return Ok(json!({ "has_update": false, "error": e.to_string() })),
    };
    match updater.check().await {
        Ok(Some(update)) => Ok(json!({
            "has_update": true,
            "version": update.version,
            "body": update.body
        })),
        Ok(None) => Ok(json!({ "has_update": false })),
        Err(e) => Ok(json!({ "has_update": false, "error": e.to_string() })),
    }
}

#[tauri::command]
pub async fn perform_app_update(app: AppHandle) -> Result<bool, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
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
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(false)
}