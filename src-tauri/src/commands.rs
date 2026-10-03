use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use tauri::{AppHandle, Emitter, State};

use crate::api_manager::ApiManager;
use crate::auth_manager::AuthManager;
use crate::builder_manager::BuilderManager;
use crate::cartographer_manager::CartographerManager;
use crate::config;
use crate::database::DatabaseManager;
use crate::doctor_manager::DoctorManager;
use crate::instance_manager::InstanceManager;
use crate::java_manager::JavaManager;
use crate::media_manager::MediaManager;
use crate::mod_manager::{DetailedModInfo, ModManager};
use crate::monitor_service::MonitorService;
use crate::shield::ShieldManager;
use crate::store_manager::{StoreDetailsResponseDto, StoreInstallResultDto, StoreManager, StoreSearchResultDto};
use crate::swarm_manager::SwarmManager;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;
use crate::tunnel_manager::TunnelManager;
use crate::vcs_manager::VCSManager;
use crate::world_manager::WorldManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchPayload {
    pub version: String,
    pub loader: String,
    pub loader_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericActionResult {
    pub success: bool,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRecord {
    pub name: String,
    pub status: String,
    pub avatar: String,
    pub activity: Option<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBuildResult {
    pub success: bool,
    pub mods: Vec<String>,
    pub foundation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindResolveResult {
    pub success: bool,
    pub changes: i32,
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
    pub stop_signal: Arc<AtomicBool>,
    pub cached_kip_profile: Arc<Mutex<Option<Value>>>,
    pub cached_ms_profile: Arc<Mutex<Option<Value>>>,
}

#[tauri::command]
pub async fn kip_login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<Value, String> {
    let clean_user = username.trim().to_string();
    if clean_user.is_empty() || password.is_empty() {
        return Ok(json!({ "success": false, "msg": "Username and password cannot be empty." }));
    }

    let res = state.api.kip_auth_login(&clean_user, &password).await?;
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
) -> Result<Value, String> {
    let clean_user = username.trim().to_string();
    if clean_user.len() < 3 || clean_user.len() > 24 {
        return Ok(json!({ "success": false, "msg": "Username must be between 3 and 24 characters." }));
    }
    if password.len() < 6 {
        return Ok(json!({ "success": false, "msg": "Password must be at least 6 characters long." }));
    }

    let email_str = email.unwrap_or_default();
    let res = state.api.kip_auth_register(&clean_user, &email_str, &password).await?;
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
pub async fn ms_auth_start(state: State<'_, AppState>) -> Result<Value, String> {
    let res = state.auth.get_device_code().await?;
    if let Some(uri) = res["verification_uri"].as_str() {
        let _ = open::that(uri);
    }
    Ok(res)
}

#[tauri::command]
pub async fn ms_auth_poll(state: State<'_, AppState>, device_code: String) -> Result<bool, String> {
    match state.auth.poll_token(&device_code).await? {
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
pub async fn get_ms_profile(state: State<'_, AppState>) -> Result<Value, String> {
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
pub async fn get_mc_versions() -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    match client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(data) = res.json::<Value>().await {
                if let Some(versions) = data["versions"].as_array() {
                    let list: Vec<String> = versions
                        .iter()
                        .filter(|v| v["type"].as_str() == Some("release"))
                        .filter_map(|v| v["id"].as_str().map(|s| s.to_string()))
                        .collect();
                    if !list.is_empty() {
                        return Ok(list);
                    }
                }
            }
        }
        Err(_) => {}
    }

    Ok(vec![
        "1.21.1".to_string(),
        "1.21".to_string(),
        "1.20.4".to_string(),
        "1.20.1".to_string(),
        "1.19.4".to_string(),
        "1.19.2".to_string(),
        "1.18.2".to_string(),
        "1.16.5".to_string(),
        "1.12.2".to_string(),
        "1.8.9".to_string(),
    ])
}

#[tauri::command]
pub async fn get_loader_versions(loader: String, mc_version: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let normalized_loader = loader.to_lowercase();

    match normalized_loader.as_str() {
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
pub async fn launch_game(
    state: State<'_, AppState>,
    app: AppHandle,
    version: String,
    loader: String,
    loader_version: Option<String>,
) -> Result<LaunchResult, String> {
    let resolved_version = if version.trim().is_empty() || version == "latest" || version == "auto" {
        match get_mc_versions().await {
            Ok(v_list) if !v_list.is_empty() => v_list[0].clone(),
            _ => "1.21.1".to_string(),
        }
    } else {
        version.trim().to_string()
    };

    let resolved_loader = if loader.trim().is_empty() {
        "vanilla".to_string()
    } else {
        loader.trim().to_lowercase()
    };

    let mut cfg = config::load_app_config();
    let mc_dir = cfg.current_instance.clone();
    let saves_path = Path::new(&mc_dir).join("saves");
    let backups_path = Path::new(&mc_dir).join("backups_devkit");

    let _ = app.emit("updateLaunchStatus", "Calibrating game instance...");
    let _ = app.emit("updateLaunchProgress", 15);

    let saves_str = saves_path.to_string_lossy().to_string();
    let backups_str = backups_path.to_string_lossy().to_string();
    let auto_backup_flag = cfg.auto_backup;

    tokio::task::spawn_blocking(move || {
        ToolManager::unlock_worlds(&saves_str);
        SystemUtils::kill_zombie_processes();
        if auto_backup_flag {
            ToolManager::create_backup(&saves_str, &backups_str);
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    let token = config::get_secret("ms_access_token");
    let mut account = json!({
        "name": "Player",
        "uuid": "00000000-0000-0000-0000-000000000000",
        "access_token": "0"
    });

    if !token.is_empty() {
        if let Ok(acc) = state.auth.authenticate_minecraft(&token).await {
            account = acc;
        }
    }

    if account["access_token"].as_str() == Some("0") {
        let off_name = if !cfg.offline_username.trim().is_empty() {
            cfg.offline_username.clone()
        } else {
            let random_suffix = &uuid::Uuid::new_v4().simple().to_string()[..4];
            let generated = format!("Player_{}", random_suffix);
            cfg.offline_username = generated.clone();
            config::save_app_config(&cfg);
            generated
        };

        let mut hasher = Sha1::new();
        hasher.update(off_name.as_bytes());
        let hash = hasher.finalize();
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash[..16]);
        let offline_uuid = uuid::Uuid::from_bytes(uuid_bytes).hyphenated().to_string();

        account["name"] = json!(off_name);
        account["uuid"] = json!(offline_uuid);
    } else if let Some(u) = account["uuid"].as_str() {
        if u.len() == 32 && !u.contains('-') {
            let formatted = format!("{}-{}-{}-{}-{}", &u[0..8], &u[8..12], &u[12..16], &u[16..20], &u[20..32]);
            account["uuid"] = json!(formatted);
        }
    }

    let _ = app.emit("updateLaunchStatus", "Resolving manifests and libraries...");
    let _ = app.emit("updateLaunchProgress", 45);

    let lv = loader_version.unwrap_or_default();
    match state
        .instance
        .launch_game(&resolved_version, &resolved_loader, &lv, &mc_dir, &account, &app)
        .await
    {
        Ok((msg, pid)) => {
            state.monitor.set_game_pid(pid);
            let _ = app.emit("updateLaunchStatus", "Minecraft process initialized");
            let _ = app.emit("updateLaunchProgress", 100);
            Ok(LaunchResult {
                success: true,
                message: msg,
                pid: Some(pid),
            })
        }
        Err(e) => {
            let _ = app.emit("updateLaunchStatus", format!("Ignition failure: {}", e));
            Ok(LaunchResult {
                success: false,
                message: e,
                pid: None,
            })
        }
    }
}

#[tauri::command]
pub async fn change_instance(new_dir: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let trimmed = new_dir.trim();
        if trimmed.is_empty() {
            return false;
        }

        let p = Path::new(trimmed);
        if !p.exists() {
            if fs::create_dir_all(p).is_err() {
                return false;
            }
        }

        let mut cfg = config::load_app_config();
        cfg.current_instance = trimmed.to_string();
        if !cfg.instances.contains(&trimmed.to_string()) {
            cfg.instances.push(trimmed.to_string());
        }
        config::save_app_config(&cfg);
        config::update_paths(trimmed);
        true
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_local_mods(
    content_type: Option<String>,
) -> Result<Vec<DetailedModInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        ModManager::get_content_list(&cfg.current_instance, &c_type)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_mod(filename: String, content_type: Option<String>) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return false,
        };

        let cfg = config::load_app_config();
        let subfolder = match content_type.as_deref() {
            Some("resourcepacks") => "resourcepacks",
            Some("shaderpacks") => "shaderpacks",
            _ => "mods",
        };
        let filepath = Path::new(&cfg.current_instance).join(subfolder).join(safe_filename);
        ModManager::toggle_file(filepath.to_str().unwrap_or(""))
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_mod(filename: String, content_type: Option<String>) -> Result<GenericActionResult, String> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => {
                return GenericActionResult {
                    success: false,
                    msg: "Invalid filename.".to_string(),
                }
            }
        };

        let cfg = config::load_app_config();
        let subfolder = match content_type.as_deref() {
            Some("resourcepacks") => "resourcepacks",
            Some("shaderpacks") => "shaderpacks",
            _ => "mods",
        };
        let filepath = Path::new(&cfg.current_instance).join(subfolder).join(safe_filename);
        if ModManager::delete_file(filepath.to_str().unwrap_or("")) {
            GenericActionResult {
                success: true,
                msg: "Package deleted successfully.".to_string(),
            }
        } else {
            GenericActionResult {
                success: false,
                msg: "File not found or failed to delete.".to_string(),
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_toggle_mods(
    filenames: Vec<String>,
    enable: bool,
    content_type: Option<String>,
) -> Result<usize, String> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        ModManager::batch_toggle(&cfg.current_instance, &filenames, enable, &c_type)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_delete_mods(
    filenames: Vec<String>,
    content_type: Option<String>,
) -> Result<usize, String> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let c_type = content_type.unwrap_or_else(|| "mods".to_string());
        ModManager::batch_delete(&cfg.current_instance, &filenames, &c_type)
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_content_folder(content_type: String) -> Result<(), String> {
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
pub async fn search_store(
    state: State<'_, AppState>,
    provider: String,
    query: Option<String>,
    project_type: Option<String>,
    loader: Option<String>,
    game_version: Option<String>,
    category: Option<String>,
    sort_index: Option<String>,
    offset: Option<i32>,
) -> Result<StoreSearchResultDto, String> {
    let cfg = config::load_app_config();
    let q = query.unwrap_or_default();
    let pt = project_type.unwrap_or_else(|| "mod".to_string());
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();
    let c = category.unwrap_or_default();
    let si = sort_index.unwrap_or_else(|| "relevance".to_string());
    let off = offset.unwrap_or(0);

    state.store.search(
        &provider,
        &q,
        &pt,
        &l,
        &gv,
        &c,
        &si,
        off,
        &cfg.current_instance,
    ).await
}

#[tauri::command]
pub async fn get_store_full_details(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<StoreDetailsResponseDto, String> {
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();
    state.store.get_details(&provider, &project_id, &l, &gv).await
}

#[tauri::command]
pub async fn download_store_item(
    state: State<'_, AppState>,
    app: AppHandle,
    provider: String,
    project_id: String,
    version_id: Option<String>,
    url: String,
    filename: String,
    project_type: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<StoreInstallResultDto, String> {
    let cfg = config::load_app_config();
    let v_id = version_id.unwrap_or_default();
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();

    state.store.install_with_dependencies(
        &app,
        &provider,
        &project_id,
        &v_id,
        &url,
        &filename,
        &project_type,
        &cfg.current_instance,
        &l,
        &gv,
    ).await
}

#[tauri::command]
pub async fn download_specific_file(
    state: State<'_, AppState>,
    app: AppHandle,
    provider: Option<String>,
    project_id: Option<String>,
    version_id: Option<String>,
    url: String,
    filename: String,
    project_type: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<StoreInstallResultDto, String> {
    let prov = provider.unwrap_or_else(|| "modrinth".to_string());
    let p_id = project_id.unwrap_or_default();
    download_store_item(
        state,
        app,
        prov,
        p_id,
        version_id,
        url,
        filename,
        project_type,
        loader,
        game_version,
    ).await
}

#[tauri::command]
pub fn uninstall_store_item(
    state: State<'_, AppState>,
    project_type: String,
    filename: String,
) -> Result<bool, String> {
    let cfg = config::load_app_config();
    state.store.uninstall_item(&cfg.current_instance, &project_type, &filename)
}

#[tauri::command]
pub async fn generate_auto_build(
    state: State<'_, AppState>,
    prompt: String,
    mc_version: String,
    loader: String,
) -> Result<AutoBuildResult, String> {
    let mods = state.builder.generate_mod_list(&prompt, &mc_version, &loader).await?;
    let foundation = state
        .builder
        .get_foundation_mods(&loader)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    Ok(AutoBuildResult {
        success: true,
        mods,
        foundation,
    })
}

#[tauri::command]
pub async fn resolve_keybinds(state: State<'_, AppState>) -> Result<KeybindResolveResult, String> {
    let builder = state.builder.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let changes = builder.resolve_keybinds(&cfg.current_instance);
        KeybindResolveResult {
            success: true,
            changes,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn swarm_seed_start(
    state: State<'_, AppState>,
    target_folder: String,
) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let safe_folder = Path::new(&target_folder)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let target_dir = Path::new(&cfg.current_instance).join(safe_folder);
    state.swarm.start_seeding(target_dir.to_str().unwrap_or(""))
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
pub async fn get_friends(state: State<'_, AppState>) -> Result<Vec<FriendRecord>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let names = db.get_friends();
        names
            .into_iter()
            .enumerate()
            .map(|(idx, f)| {
                let status_variant = if idx == 0 { "online" } else { "offline" };
                let activity_variant = if idx == 0 {
                    Some("In Voice Matrix".to_string())
                } else {
                    None
                };
                FriendRecord {
                    avatar: format!("https://api.mineatar.io/face/{}?scale=10", f),
                    status: status_variant.to_string(),
                    name: f,
                    activity: activity_variant,
                    is_favorite: idx == 0,
                }
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_friend(state: State<'_, AppState>, name: String) -> Result<GenericActionResult, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean = name.trim();
        if clean.len() < 3
            || clean.len() > 16
            || !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return GenericActionResult {
                success: false,
                msg: "Invalid nickname. Must be 3-16 characters and contain only letters, numbers, or underscores.".to_string(),
            };
        }
        if db.add_friend(clean) {
            GenericActionResult {
                success: true,
                msg: format!("Added {} to friends.", clean),
            }
        } else {
            GenericActionResult {
                success: false,
                msg: "Friend already exists.".to_string(),
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_friend(state: State<'_, AppState>, name: String) -> Result<GenericActionResult, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean = name.trim();
        if db.remove_friend(clean) {
            GenericActionResult {
                success: true,
                msg: format!("Removed {}.", clean),
            }
        } else {
            GenericActionResult {
                success: false,
                msg: "Friend not found.".to_string(),
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_tunnel(state: State<'_, AppState>, app: AppHandle, port: String) -> bool {
    state.tunnel.start(&port, move |msg| {
        let _ = app.emit("updateTunnelStatus", msg);
    });
    true
}

#[tauri::command]
pub fn stop_tunnel(state: State<'_, AppState>) -> bool {
    state.tunnel.stop();
    true
}

#[tauri::command]
pub async fn deploy_docker_server(
    state: State<'_, AppState>,
    core: String,
    version: String,
    port: String,
) -> Result<GenericActionResult, String> {
    let instance = state.instance.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        match instance
            .deploy_docker_server(&core, &version, &port, &cfg.current_instance)
        {
            Ok(msg) => GenericActionResult { success: true, msg },
            Err(e) => GenericActionResult { success: false, msg: e },
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ptero_connect(
    state: State<'_, AppState>,
    url: String,
    key: String,
) -> Result<Value, String> {
    let clean_url = url.trim().trim_end_matches('/').to_string();
    if clean_url.is_empty() || key.trim().is_empty() {
        return Ok(json!({ "success": false, "msg": "Panel URL and API key cannot be empty." }));
    }

    let mut cfg = config::load_app_config();
    cfg.ptero_url = clean_url.clone();
    config::set_secret("ptero_key", key.trim());
    config::save_app_config(&cfg);
    let servers = state.api.get_ptero_server_status(&clean_url, key.trim()).await?;
    Ok(json!({ "success": true, "status": servers }))
}

#[tauri::command]
pub async fn ptero_action(
    state: State<'_, AppState>,
    action: String,
    server_id: String,
) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let key = config::get_secret("ptero_key");
    let ok = state
        .api
        .send_ptero_power_action(&cfg.ptero_url, &server_id, &action, &key)
        .await?;
    Ok(json!({ "success": ok }))
}