use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::path::Path;
use std::fs;
use tauri::{AppHandle, State, Emitter};
use serde_json::{json, Value};
use parking_lot::Mutex;
use sha1::{Sha1, Digest};

use crate::config;
use crate::database::DatabaseManager;
use crate::auth_manager::AuthManager;
use crate::api_manager::ApiManager;
use crate::instance_manager::InstanceManager;
use crate::mod_manager::ModManager;
use crate::doctor_manager::DoctorManager;
use crate::shield_manager::ShieldManager;
use crate::swarm_manager::SwarmManager;
use crate::tunnel_manager::TunnelManager;
use crate::world_manager::WorldManager;
use crate::media_manager::MediaManager;
use crate::vcs_manager::VCSManager;
use crate::cartographer_manager::CartographerManager;
use crate::builder_manager::BuilderManager;
use crate::monitor_service::MonitorService;

pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub auth: Arc<AuthManager>,
    pub api: Arc<ApiManager>,
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
    pub stop_signal: Arc<AtomicBool>,
    pub cached_kip_profile: Arc<Mutex<Option<Value>>>,
    pub cached_ms_profile: Arc<Mutex<Option<Value>>>,
}

#[tauri::command]
pub async fn kip_login(state: State<'_, AppState>, username: String, password: String) -> Result<Value, String> {
    let clean_user = username.trim();
    if clean_user.is_empty() || password.is_empty() {
        return Ok(json!({ "success": false, "msg": "Username and password cannot be empty." }));
    }

    let res = state.api.kip_auth_login(clean_user, &password).await?;
    if res["success"].as_bool().unwrap_or(false) {
        if let Some(token) = res["token"].as_str() {
            config::set_secret("kip_token", token);
        }
        let mut cfg = config::load_app_config();
        cfg.kip_username = res["username"].as_str().unwrap_or(clean_user).to_string();
        config::save_app_config(&cfg);
        *state.cached_kip_profile.lock() = Some(res.clone());
    }
    Ok(res)
}

#[tauri::command]
pub async fn kip_register(state: State<'_, AppState>, username: String, email: Option<String>, password: String) -> Result<Value, String> {
    let clean_user = username.trim();
    if clean_user.len() < 3 || clean_user.len() > 24 {
        return Ok(json!({ "success": false, "msg": "Username must be between 3 and 24 characters." }));
    }
    if password.len() < 6 {
        return Ok(json!({ "success": false, "msg": "Password must be at least 6 characters long." }));
    }

    let email_str = email.unwrap_or_default();
    let res = state.api.kip_auth_register(clean_user, &email_str, &password).await?;
    if res["success"].as_bool().unwrap_or(false) {
        if let Some(token) = res["token"].as_str() {
            config::set_secret("kip_token", token);
        }
        let mut cfg = config::load_app_config();
        cfg.kip_username = res["username"].as_str().unwrap_or(clean_user).to_string();
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
        let name = if !cfg.kip_username.is_empty() { cfg.kip_username } else { "Authenticated User".to_string() };
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
pub async fn get_mc_versions() -> Vec<String> {
    let client = reqwest::Client::new();
    if let Ok(res) = client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").send().await {
        if let Ok(data) = res.json::<Value>().await {
            if let Some(versions) = data["versions"].as_array() {
                return versions.iter()
                    .filter(|v| v["type"].as_str() == Some("release"))
                    .filter_map(|v| v["id"].as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
    }
    vec![
        "1.21.1".to_string(), "1.21".to_string(), "1.20.4".to_string(), "1.20.1".to_string(),
        "1.19.4".to_string(), "1.19.2".to_string(), "1.18.2".to_string(), "1.16.5".to_string(),
        "1.12.2".to_string(), "1.8.9".to_string(),
    ]
}

#[tauri::command]
pub async fn get_loader_versions(loader: String, mc_version: String) -> Vec<String> {
    let client = reqwest::Client::new();
    match loader.to_lowercase().as_str() {
        "fabric" => {
            let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(arr) = data.as_array() {
                        return arr.iter()
                            .filter_map(|v| v["loader"]["version"].as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            }
        }
        "quilt" => {
            let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(arr) = data.as_array() {
                        return arr.iter()
                            .filter_map(|v| v["loader"]["version"].as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            }
        }
        "neoforge" => {
            let url = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
            if let Ok(res) = client.get(url).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(arr) = data["versions"].as_array() {
                        let mut versions: Vec<String> = arr.iter()
                            .filter_map(|v| v.as_str())
                            .filter(|s| s.starts_with(&mc_version))
                            .map(|s| s.to_string())
                            .collect();
                        versions.sort();
                        versions.reverse();
                        return versions;
                    }
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
                    return list;
                }
            }
        }
        _ => {}
    }
    Vec::new()
}

#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app: AppHandle,
    version: String,
    loader: String,
    loader_version: Option<String>,
) -> Result<Value, String> {
    let token = config::get_secret("ms_access_token");
    let mut account = json!({
        "name": "Player",
        "uuid": "00000000000000000000000000000000",
        "access_token": "0"
    });

    if !token.is_empty() {
        if let Ok(acc) = state.auth.authenticate_minecraft(&token).await {
            account = acc;
        }
    }

    if account["access_token"].as_str() == Some("0") {
        let cfg = config::load_app_config();
        let off_name = if !cfg.offline_username.trim().is_empty() { cfg.offline_username } else { "Player".to_string() };
        let mut hasher = Sha1::new();
        hasher.update(off_name.as_bytes());
        let hash = hasher.finalize();
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash[..16]);
        let offline_uuid = uuid::Uuid::from_bytes(uuid_bytes).simple().to_string();
        account["name"] = json!(off_name);
        account["uuid"] = json!(offline_uuid);
    }

    let cfg = config::load_app_config();
    let mc_dir = cfg.current_instance.clone();

    let _ = app.emit("updateLaunchStatus", "Starting Minecraft process...");

    let lv = loader_version.unwrap_or_default();
    match state.instance.launch_game(&version, &loader, &lv, &mc_dir, &account, &app).await {
        Ok((msg, pid)) => {
            state.monitor.set_game_pid(pid);
            let _ = app.emit("updateLaunchStatus", "Launched successfully");
            Ok(json!({ "success": true, "message": msg }))
        }
        Err(e) => {
            let _ = app.emit("updateLaunchStatus", format!("Error: {}", e));
            Ok(json!({ "success": false, "message": e }))
        }
    }
}

#[tauri::command]
pub fn change_instance(new_dir: String) -> bool {
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
}

#[tauri::command]
pub fn get_local_mods() -> Vec<Value> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(mods_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            if name.ends_with(".jar") || name.ends_with(".jar.disabled") {
                let is_disabled = name.ends_with(".disabled");
                let icon = ModManager::extract_icon(p.to_str().unwrap_or(""));
                result.push(json!({
                    "filename": name,
                    "name": name.replace(".jar", "").replace(".disabled", ""),
                    "version": "?",
                    "author": "?",
                    "loaders": ["fabric"],
                    "disabled": is_disabled,
                    "icon": icon
                }));
            }
        }
    }
    result
}

#[tauri::command]
pub fn toggle_mod(filename: String) -> bool {
    let safe_filename = match Path::new(&filename).file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return false,
    };

    let cfg = config::load_app_config();
    let filepath = Path::new(&cfg.current_instance).join("mods").join(safe_filename);
    ModManager::toggle_mod(filepath.to_str().unwrap_or(""))
}

#[tauri::command]
pub fn delete_mod(filename: String) -> Value {
    let safe_filename = match Path::new(&filename).file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return json!({ "success": false, "msg": "Invalid filename." }),
    };

    let cfg = config::load_app_config();
    let filepath = Path::new(&cfg.current_instance).join("mods").join(safe_filename);
    if ModManager::delete_mod(filepath.to_str().unwrap_or("")) {
        json!({ "success": true, "msg": "Mod deleted successfully." })
    } else {
        json!({ "success": false, "msg": "File not found or failed to delete." })
    }
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
) -> Result<Value, String> {
    let q = query.unwrap_or_default();
    let pt = project_type.unwrap_or_else(|| "mod".to_string());
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();
    let c = category.unwrap_or_default();
    let si = sort_index.unwrap_or_else(|| "relevance".to_string());
    let off = offset.unwrap_or(0);

    let res = if provider == "curseforge" {
        state.api.search_curseforge(&q, &pt, &l, &gv, &c, &si, off).await
    } else {
        state.api.search_modrinth(&q, &pt, &l, &gv, &c, &si, off).await
    };

    match res {
        Ok(val) => Ok(val),
        Err(err_msg) => Ok(json!({
            "success": false,
            "hits": [],
            "msg": err_msg
        }))
    }
}

#[tauri::command]
pub async fn get_store_full_details(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<Value, String> {
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();

    if provider == "curseforge" {
        state.api.get_curseforge_details(&project_id, &l, &gv).await
    } else {
        state.api.get_modrinth_details(&project_id, &l, &gv).await
    }
}

#[tauri::command]
pub async fn download_store_item(
    url: String,
    filename: String,
    project_type: String,
) -> Result<bool, String> {
    let safe_filename = match Path::new(&filename).file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return Err("Invalid filename provided.".to_string()),
    };

    let cfg = config::load_app_config();
    let target_subfolder = match project_type.as_str() {
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        _ => "mods",
    };

    let target_dir = Path::new(&cfg.current_instance).join(target_subfolder);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let dest = target_dir.join(safe_filename);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    fs::write(dest, bytes).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn download_specific_file(
    url: String,
    filename: String,
    project_type: String,
) -> Result<bool, String> {
    download_store_item(url, filename, project_type).await
}

#[tauri::command]
pub async fn generate_auto_build(
    state: State<'_, AppState>,
    prompt: String,
    mc_version: String,
    loader: String,
) -> Result<Value, String> {
    let mods = state.builder.generate_mod_list(&prompt, &mc_version, &loader).await?;
    let foundation = state.builder.get_foundation_mods(&loader);
    Ok(json!({
        "success": true,
        "mods": mods,
        "foundation": foundation
    }))
}

#[tauri::command]
pub fn resolve_keybinds(state: State<'_, AppState>) -> Value {
    let cfg = config::load_app_config();
    let changes = state.builder.resolve_keybinds(&cfg.current_instance);
    json!({ "success": true, "changes": changes })
}

#[tauri::command]
pub fn swarm_seed_start(state: State<'_, AppState>, target_folder: String) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let safe_folder = Path::new(&target_folder).file_name().unwrap_or_default().to_string_lossy().to_string();
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
pub fn get_friends(state: State<'_, AppState>) -> Vec<Value> {
    let names = state.db.get_friends();
    names.into_iter().map(|f| {
        json!({
            "name": f,
            "status": "offline",
            "avatar": format!("https://api.mineatar.io/face/{}?scale=10", f)
        })
    }).collect()
}

#[tauri::command]
pub fn add_friend(state: State<'_, AppState>, name: String) -> Value {
    let clean = name.trim();
    if clean.len() < 3 || clean.len() > 16 || !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return json!({ "success": false, "msg": "Invalid nickname. Must be 3-16 characters and contain only letters, numbers, or underscores." });
    }
    if state.db.add_friend(clean) {
        json!({ "success": true, "msg": format!("Added {} to friends.", clean) })
    } else {
        json!({ "success": false, "msg": "Friend already exists." })
    }
}

#[tauri::command]
pub fn remove_friend(state: State<'_, AppState>, name: String) -> Value {
    let clean = name.trim();
    if state.db.remove_friend(clean) {
        json!({ "success": true, "msg": format!("Removed {}.", clean) })
    } else {
        json!({ "success": false, "msg": "Friend not found." })
    }
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
pub fn deploy_docker_server(state: State<'_, AppState>, core: String, version: String, port: String) -> Value {
    let cfg = config::load_app_config();
    match state.instance.deploy_docker_server(&core, &version, &port, &cfg.current_instance) {
        Ok(msg) => json!({ "success": true, "msg": msg }),
        Err(e) => json!({ "success": false, "msg": e }),
    }
}

#[tauri::command]
pub async fn ptero_connect(state: State<'_, AppState>, url: String, key: String) -> Result<Value, String> {
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
pub async fn ptero_action(state: State<'_, AppState>, action: String, server_id: String) -> Result<Value, String> {
    let cfg = config::load_app_config();
    let key = config::get_secret("ptero_key");
    let ok = state.api.send_ptero_power_action(&cfg.ptero_url, &server_id, &action, &key).await?;
    Ok(json!({ "success": ok }))
}