pub mod booster;
pub mod builder;
pub mod console;
pub mod content;
pub mod launch;
pub mod media;
pub mod network;
pub mod nexus;
pub mod overlay;
pub mod settings;
pub mod store;
pub mod support;
pub mod system;
pub mod tools;
pub mod window;
pub mod worlds;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::api_manager::ApiManager;
use crate::auth_manager::AuthManager;
use crate::builder_manager::BuilderManager;
use crate::cartographer_manager::CartographerManager;
use crate::chunk_engine::ChunkAcceleratorManager;
use crate::config;
use crate::database::DatabaseManager;
use crate::doctor_manager::DoctorManager;
use crate::error::AppError;
use crate::hardware_booster::HardwareBoosterManager;
use crate::instance_manager::InstanceManager;
use crate::java_manager::JavaManager;
use crate::media_manager::MediaManager;
use crate::memory_matrix::MemoryMatrixManager;
use crate::mod_manager::{DetailedModInfo, ModManager};
use crate::monitor_service::MonitorService;
use crate::network_optimizer::NetworkOptimizer;
use crate::shield::ShieldManager;
use crate::store_manager::StoreManager;
use crate::swarm_manager::SwarmManager;
use crate::tunnel_manager::TunnelManager;
use crate::vcs_manager::VCSManager;
use crate::world_manager::WorldManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersionEntryDto {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub auth: Arc<AuthManager>,
    pub api: Arc<ApiManager>,
    pub java: Arc<JavaManager>,
    pub instance: Arc<InstanceManager>,
    pub doctor: Arc<DoctorManager>,
    pub shield: Arc<ShieldManager>,
    pub swarm: Arc<SwarmManager>,
    pub tunnel: Arc<TunnelManager>,
    pub world: Arc<WorldManager>,
    pub media: Arc<MediaManager>,
    pub vcs: Arc<VCSManager>,
    pub cartographer: Arc<CartographerManager>,
    pub builder: Arc<BuilderManager>,
    pub monitor: Arc<MonitorService>,
    pub store: Arc<StoreManager>,
    pub memory_matrix: Arc<MemoryMatrixManager>,
    pub chunk_engine: Arc<ChunkAcceleratorManager>,
    pub net_optimizer: Arc<NetworkOptimizer>,
    pub hardware_booster: Arc<HardwareBoosterManager>,
    pub stop_signal: Arc<AtomicBool>,
    pub cached_kip_profile: Arc<Mutex<Option<Value>>>,
    pub cached_ms_profile: Arc<Mutex<Option<Value>>>,
}

#[tauri::command]
pub async fn get_all_minecraft_versions() -> Result<Vec<McVersionEntryDto>, AppError> {
    let client = reqwest::Client::new();
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    if let Ok(res) = client.get(url).send().await {
        if let Ok(data) = res.json::<Value>().await {
            if let Some(versions) = data["versions"].as_array() {
                let mut list: Vec<McVersionEntryDto> = versions
                    .iter()
                    .filter_map(|v| {
                        let id = v["id"].as_str()?.to_string();
                        let version_type = v["type"].as_str().unwrap_or("release").to_string();
                        let release_time = v["releaseTime"].as_str().unwrap_or("").to_string();
                        Some(McVersionEntryDto {
                            id,
                            version_type,
                            release_time,
                        })
                    })
                    .collect();

                let modern_additions = [
                    McVersionEntryDto { id: "26.3".to_string(), version_type: "release".to_string(), release_time: "2026-03-24T12:00:00Z".to_string() },
                    McVersionEntryDto { id: "26.2".to_string(), version_type: "release".to_string(), release_time: "2026-02-18T10:00:00Z".to_string() },
                    McVersionEntryDto { id: "26.1".to_string(), version_type: "release".to_string(), release_time: "2026-01-14T09:00:00Z".to_string() },
                ];

                for item in modern_additions.into_iter().rev() {
                    if !list.iter().any(|v| v.id == item.id) {
                        list.insert(0, item);
                    }
                }

                if !list.is_empty() {
                    return Ok(list);
                }
            }
        }
    }

    Ok(vec![
        McVersionEntryDto { id: "26.3".to_string(), version_type: "release".to_string(), release_time: "2026-03-24T12:00:00Z".to_string() },
        McVersionEntryDto { id: "26.2".to_string(), version_type: "release".to_string(), release_time: "2026-02-18T10:00:00Z".to_string() },
        McVersionEntryDto { id: "26.1".to_string(), version_type: "release".to_string(), release_time: "2026-01-14T09:00:00Z".to_string() },
        McVersionEntryDto { id: "1.21.4".to_string(), version_type: "release".to_string(), release_time: "2024-12-03T10:00:00Z".to_string() },
        McVersionEntryDto { id: "1.21.3".to_string(), version_type: "release".to_string(), release_time: "2024-10-23T11:00:00Z".to_string() },
        McVersionEntryDto { id: "1.21.2".to_string(), version_type: "release".to_string(), release_time: "2024-10-22T08:00:00Z".to_string() },
        McVersionEntryDto { id: "1.21.1".to_string(), version_type: "release".to_string(), release_time: "2024-08-08T12:00:00Z".to_string() },
        McVersionEntryDto { id: "1.21".to_string(), version_type: "release".to_string(), release_time: "2024-06-13T10:30:00Z".to_string() },
        McVersionEntryDto { id: "1.20.6".to_string(), version_type: "release".to_string(), release_time: "2024-04-29T09:15:00Z".to_string() },
        McVersionEntryDto { id: "1.20.4".to_string(), version_type: "release".to_string(), release_time: "2023-12-07T11:00:00Z".to_string() },
        McVersionEntryDto { id: "1.20.2".to_string(), version_type: "release".to_string(), release_time: "2023-09-21T10:00:00Z".to_string() },
        McVersionEntryDto { id: "1.20.1".to_string(), version_type: "release".to_string(), release_time: "2023-06-12T08:30:00Z".to_string() },
        McVersionEntryDto { id: "1.19.4".to_string(), version_type: "release".to_string(), release_time: "2023-03-14T10:00:00Z".to_string() },
        McVersionEntryDto { id: "1.19.2".to_string(), version_type: "release".to_string(), release_time: "2022-08-05T11:00:00Z".to_string() },
        McVersionEntryDto { id: "1.18.2".to_string(), version_type: "release".to_string(), release_time: "2022-02-28T12:00:00Z".to_string() },
        McVersionEntryDto { id: "1.16.5".to_string(), version_type: "release".to_string(), release_time: "2021-01-15T09:00:00Z".to_string() },
        McVersionEntryDto { id: "1.12.2".to_string(), version_type: "release".to_string(), release_time: "2017-09-18T08:00:00Z".to_string() },
        McVersionEntryDto { id: "1.7.10".to_string(), version_type: "release".to_string(), release_time: "2014-06-26T12:00:00Z".to_string() },
    ])
}

#[tauri::command]
pub async fn kip_login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<Value, AppError> {
    let clean_user = username.trim().to_string();
    if clean_user.is_empty() || password.is_empty() {
        return Ok(json!({ "success": false, "msg": "Username and password cannot be empty." }));
    }

    let res = state
        .api
        .kip_auth_login(&clean_user, &password)
        .await
        .map_err(|e| AppError::Config(e))?;

    if res["success"].as_bool().unwrap_or(false) {
        if let Some(token) = res["token"].as_str() {
            config::set_secret("kip_token", token);
        }
        let mut cfg = config::load_app_config();
        cfg.kip_username = res["username"].as_str().unwrap_or(&clean_user).to_string();
        config::save_app_config(&cfg);
        *state.cached_kip_profile.lock() = Some(res.clone());
    }
    Ok(res)
}

#[tauri::command]
pub async fn kip_register(
    state: State<'_, AppState>,
    username: String,
    email: Option<String>,
    password: String,
) -> Result<Value, AppError> {
    let clean_user = username.trim().to_string();
    if clean_user.len() < 3 || clean_user.len() > 24 {
        return Ok(json!({ "success": false, "msg": "Username must be between 3 and 24 characters." }));
    }
    if password.len() < 6 {
        return Ok(json!({ "success": false, "msg": "Password must be at least 6 characters long." }));
    }

    let email_str = email.unwrap_or_default();
    let res = state
        .api
        .kip_auth_register(&clean_user, &email_str, &password)
        .await
        .map_err(|e| AppError::Config(e))?;

    if res["success"].as_bool().unwrap_or(false) {
        if let Some(token) = res["token"].as_str() {
            config::set_secret("kip_token", token);
        }
        let mut cfg = config::load_app_config();
        cfg.kip_username = res["username"].as_str().unwrap_or(&clean_user).to_string();
        config::save_app_config(&cfg);
        *state.cached_kip_profile.lock() = Some(res.clone());
    }
    Ok(res)
}

#[tauri::command]
pub fn get_kip_profile(state: State<'_, AppState>) -> Value {
    if let Some(profile) = state.cached_kip_profile.lock().clone() {
        return profile;
    }
    let token = config::get_secret("kip_token");
    if !token.is_empty() {
        let cfg = config::load_app_config();
        let name = if !cfg.kip_username.is_empty() {
            cfg.kip_username
        } else {
            "Authenticated User".to_string()
        };
        let profile = json!({ "success": true, "username": name, "token": token });
        *state.cached_kip_profile.lock() = Some(profile.clone());
        return profile;
    }
    json!({ "success": false })
}

#[tauri::command]
pub fn kip_logout(state: State<'_, AppState>) -> bool {
    config::set_secret("kip_token", "");
    let mut cfg = config::load_app_config();
    cfg.kip_username = String::new();
    config::save_app_config(&cfg);
    *state.cached_kip_profile.lock() = None;
    true
}

#[tauri::command]
pub async fn ms_auth_start(state: State<'_, AppState>) -> Result<Value, AppError> {
    let res = state.auth.get_device_code().await.map_err(|e| AppError::Config(e))?;
    if let Some(uri) = res["verification_uri"].as_str() {
        let _ = open::that(uri);
    }
    Ok(res)
}

#[tauri::command]
pub async fn ms_auth_poll(state: State<'_, AppState>, device_code: String) -> Result<bool, AppError> {
    match state.auth.poll_token(&device_code).await.map_err(|e| AppError::Config(e))? {
        Some(res) => {
            if let Some(acc) = res["access_token"].as_str() {
                config::set_secret("ms_access_token", acc);
                if let Some(ref_tok) = res["refresh_token"].as_str() {
                    config::set_secret("ms_refresh_token", ref_tok);
                }
                *state.cached_ms_profile.lock() = None;
                return Ok(true);
            }
            Ok(false)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub fn ms_logout(state: State<'_, AppState>) -> bool {
    config::set_secret("ms_access_token", "");
    config::set_secret("ms_refresh_token", "");
    *state.cached_ms_profile.lock() = None;
    true
}

#[tauri::command]
pub async fn get_ms_profile(state: State<'_, AppState>) -> Result<Value, AppError> {
    if let Some(profile) = state.cached_ms_profile.lock().clone() {
        return Ok(profile);
    }

    let token = config::get_secret("ms_access_token");
    let refresh = config::get_secret("ms_refresh_token");

    if token.is_empty() && refresh.is_empty() {
        return Ok(json!({ "success": false, "msg": "No token" }));
    }

    let mut current_token = token;
    let mut acc_res = if !current_token.is_empty() {
        state.auth.authenticate_minecraft(&current_token).await
    } else {
        Err("Empty token".to_string())
    };

    if acc_res.is_err() && !refresh.is_empty() {
        if let Ok(new_tokens) = state.auth.refresh_token(&refresh).await {
            if let Some(new_access) = new_tokens["access_token"].as_str() {
                config::set_secret("ms_access_token", new_access);
                if let Some(new_ref) = new_tokens["refresh_token"].as_str() {
                    config::set_secret("ms_refresh_token", new_ref);
                }
                current_token = new_access.to_string();
                acc_res = state.auth.authenticate_minecraft(&current_token).await;
            }
        }
    }

    match acc_res {
        Ok(acc) => {
            let res = json!({
                "success": true,
                "name": acc["name"],
                "uuid": acc["uuid"]
            });
            *state.cached_ms_profile.lock() = Some(res.clone());
            Ok(res)
        }
        Err(e) => Ok(json!({ "success": false, "msg": e })),
    }
}

#[tauri::command]
pub async fn get_mc_versions() -> Result<Vec<String>, AppError> {
    get_all_minecraft_versions().await.map(|list| list.into_iter().map(|v| v.id).collect())
}

#[tauri::command]
pub async fn get_loader_versions(loader: String, mc_version: String) -> Result<Vec<String>, AppError> {
    let client = reqwest::Client::new();
    let norm = loader.to_lowercase();

    match norm.as_str() {
        "fabric" => {
            let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(arr) = data.as_array() {
                        return Ok(arr
                            .iter()
                            .filter_map(|v| v["loader"]["version"].as_str().map(|s| s.to_string()))
                            .collect());
                    }
                }
            }
        }
        "quilt" => {
            let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(arr) = data.as_array() {
                        return Ok(arr
                            .iter()
                            .filter_map(|v| v["loader"]["version"].as_str().map(|s| s.to_string()))
                            .collect());
                    }
                }
            }
        }
        "neoforge" => {
            let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
            if let Ok(res) = client.get(url).send().await {
                if let Ok(body) = res.text().await {
                    let mut versions: Vec<String> = Vec::new();
                    for line in body.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                            let v = trimmed.trim_start_matches("<version>").trim_end_matches("</version>");
                            versions.push(v.to_string());
                        }
                    }
                    versions.reverse();
                    return Ok(versions);
                }
            }
        }
        "forge" => {
            let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
            if let Ok(res) = client.get(url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    let mut list = Vec::new();
                    if let Some(rec) = data["promos"][format!("{}-recommended", mc_version)].as_str() {
                        list.push(rec.to_string());
                    }
                    if let Some(lat) = data["promos"][format!("{}-latest", mc_version)].as_str() {
                        if !list.contains(&lat.to_string()) {
                            list.push(lat.to_string());
                        }
                    }
                    return Ok(list);
                }
            }
        }
        _ => {}
    }
    Ok(Vec::new())
}

#[tauri::command]
pub async fn change_instance(new_dir: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let trimmed = new_dir.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }

        let p = Path::new(trimmed);
        if !p.exists() {
            if fs::create_dir_all(p).is_err() {
                return Ok(false);
            }
        }

        let mut cfg = config::load_app_config();
        cfg.current_instance = trimmed.to_string();
        if !cfg.instances.contains(&trimmed.to_string()) {
            cfg.instances.push(trimmed.to_string());
        }
        config::save_app_config(&cfg);
        config::update_paths(trimmed);
        Ok(true)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn get_local_mods(content_type: Option<String>) -> Result<Vec<DetailedModInfo>, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        Ok(ModManager::get_content_list(&cfg.current_instance, &c_type))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn toggle_mod(filename: String, content_type: Option<String>) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return Ok(false),
        };

        let cfg = config::load_app_config();
        let subfolder = match content_type.as_deref() {
            Some("resourcepacks") => "resourcepacks",
            Some("shaderpacks") => "shaderpacks",
            _ => "mods",
        };
        let filepath = Path::new(&cfg.current_instance).join(subfolder).join(safe_filename);
        Ok(ModManager::toggle_file(filepath.to_str().unwrap_or("")))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn delete_mod(filename: String, content_type: Option<String>) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return Ok(false),
        };

        let cfg = config::load_app_config();
        let subfolder = match content_type.as_deref() {
            Some("resourcepacks") => "resourcepacks",
            Some("shaderpacks") => "shaderpacks",
            _ => "mods",
        };
        let filepath = Path::new(&cfg.current_instance).join(subfolder).join(safe_filename);
        Ok(ModManager::delete_file(filepath.to_str().unwrap_or("")))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn batch_toggle_mods(
    filenames: Vec<String>,
    enable: bool,
    content_type: Option<String>,
) -> Result<usize, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        Ok(ModManager::batch_toggle(&cfg.current_instance, &filenames, enable, &c_type))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn batch_delete_mods(
    filenames: Vec<String>,
    content_type: Option<String>,
) -> Result<usize, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        Ok(ModManager::batch_delete(&cfg.current_instance, &filenames, &c_type))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn open_content_folder(content_type: String) -> Result<(), AppError> {
    let cfg = config::load_app_config();
    let sub = match content_type.as_str() {
        "resourcepacks" => "resourcepacks",
        "shaderpacks" => "shaderpacks",
        _ => "mods",
    };
    let target = Path::new(&cfg.current_instance).join(sub);
    let _ = fs::create_dir_all(&target);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(target).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(target).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(target).spawn();
    }
    Ok(())
}

#[tauri::command]
pub async fn generate_auto_build(
    state: State<'_, AppState>,
    prompt: String,
    mc_version: String,
    loader: String,
) -> Result<Value, AppError> {
    let mods = state
        .builder
        .generate_mod_list(&prompt, &mc_version, &loader)
        .await
        .map_err(|e| AppError::Config(e))?;
    let foundation: Vec<String> = state
        .builder
        .get_foundation_mods(&loader)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    Ok(json!({
        "success": true,
        "mods": mods,
        "foundation": foundation
    }))
}

#[tauri::command]
pub async fn resolve_keybinds(state: State<'_, AppState>) -> Result<Value, AppError> {
    let builder = state.builder.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let changes = builder.resolve_keybinds(&cfg.current_instance);
        Ok(json!({
            "success": true,
            "changes": changes
        }))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn swarm_seed_start(
    state: State<'_, AppState>,
    target_folder: String,
) -> Result<Value, AppError> {
    let cfg = config::load_app_config();
    let safe_folder = Path::new(&target_folder)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let target_dir = Path::new(&cfg.current_instance).join(safe_folder);
    state
        .swarm
        .start_seeding(target_dir.to_str().unwrap_or(""))
        .map_err(|e| AppError::Config(e))
}

#[tauri::command]
pub fn swarm_seed_stop(state: State<'_, AppState>, torrent_name: String) -> Value {
    let ok = state.swarm.stop_seeding(&torrent_name);
    json!({ "success": ok })
}

#[tauri::command]
pub fn swarm_seed_status(state: State<'_, AppState>) -> Vec<Value> {
    state.swarm.get_seeding_status()
}

#[tauri::command]
pub async fn get_friends(state: State<'_, AppState>) -> Result<Vec<nexus::FriendDto>, AppError> {
    nexus::nexus_get_friends(state).await
}

#[tauri::command]
pub async fn add_friend(state: State<'_, AppState>, name: String) -> Result<nexus::FriendActionDto, AppError> {
    nexus::nexus_add_friend(state, name).await
}

#[tauri::command]
pub async fn remove_friend(state: State<'_, AppState>, name: String) -> Result<nexus::FriendActionDto, AppError> {
    nexus::nexus_remove_friend(state, name).await
}