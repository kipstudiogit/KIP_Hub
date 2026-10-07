use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendDto {
    pub name: String,
    pub avatar: String,
    pub status: String,
    pub activity: Option<String>,
    pub is_favorite: bool,
    pub note: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendActionDto {
    pub success: bool,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyStateDto {
    pub active: bool,
    pub room_id: String,
    pub host_endpoint: Option<String>,
    pub members_count: usize,
    pub installed_mods_count: usize,
}

#[tauri::command]
pub async fn nexus_get_friends(state: State<'_, AppState>) -> Result<Vec<FriendDto>, AppError> {
    let db = state.db.clone();
    let is_running = state.monitor.is_mc_running.load(std::sync::atomic::Ordering::Relaxed);

    tokio::task::spawn_blocking(move || {
        let records = db.get_friends();
        let friends: Vec<FriendDto> = records
            .into_iter()
            .map(|r| {
                let status = if is_running && r.is_favorite {
                    "in-game".to_string()
                } else {
                    "online".to_string()
                };

                let activity = if status == "in-game" {
                    Some("Minecraft Active Session".to_string())
                } else {
                    Some("In Hub Matrix".to_string())
                };

                FriendDto {
                    name: r.name.clone(),
                    avatar: format!("https://api.mineatar.io/face/{}?scale=10", r.name),
                    status,
                    activity,
                    is_favorite: r.is_favorite,
                    note: r.note,
                    added_at: r.added_at,
                }
            })
            .collect();

        Ok(friends)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn nexus_add_friend(state: State<'_, AppState>, name: String) -> Result<FriendActionDto, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean = name.trim();
        if clean.len() < 3 || clean.len() > 16 || !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Ok(FriendActionDto {
                success: false,
                msg: "Invalid nickname: 3-16 alphanumeric characters required.".to_string(),
            });
        }

        if db.add_friend(clean) {
            Ok(FriendActionDto {
                success: true,
                msg: format!("Operator {} linked to social matrix.", clean),
            })
        } else {
            Ok(FriendActionDto {
                success: false,
                msg: "Operator is already linked in social roster.".to_string(),
            })
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn nexus_remove_friend(state: State<'_, AppState>, name: String) -> Result<FriendActionDto, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean = name.trim();
        if db.remove_friend(clean) {
            Ok(FriendActionDto {
                success: true,
                msg: format!("Operator {} purged from social matrix.", clean),
            })
        } else {
            Ok(FriendActionDto {
                success: false,
                msg: "Operator not found.".to_string(),
            })
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn nexus_toggle_favorite(state: State<'_, AppState>, name: String) -> Result<bool, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean = name.trim();
        Ok(db.toggle_favorite_friend(clean))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn nexus_update_note(
    state: State<'_, AppState>,
    name: String,
    note: String,
) -> Result<bool, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let clean_name = name.trim();
        let clean_note = note.trim();
        Ok(db.update_friend_note(clean_name, clean_note))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn nexus_get_party_state(state: State<'_, AppState>) -> Result<PartyStateDto, AppError> {
    let tunnel = state.tunnel.clone();

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");

        let mut installed_mods_count = 0;
        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".jar") && !name.ends_with(".disabled") {
                    installed_mods_count += 1;
                }
            }
        }

        let tunnel_active = tunnel.running.load(std::sync::atomic::Ordering::Relaxed);
        let endpoint = tunnel.active_endpoint.lock().clone();

        Ok(PartyStateDto {
            active: tunnel_active,
            room_id: "kip-party".to_string(),
            host_endpoint: endpoint,
            members_count: 1,
            installed_mods_count,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}