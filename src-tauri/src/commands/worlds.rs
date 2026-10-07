use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use fastnbt::Value as NbtValue;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldCardDto {
    pub name: String,
    pub seed: String,
    pub mode: String,
    pub hardcore: bool,
    pub difficulty: String,
    pub day_count: i64,
    pub mc_version: String,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub size_mb: f64,
    pub last_played: String,
    pub datapacks: usize,
    pub icon: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VcsCommitDto {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub tree: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldActionResultDto {
    pub success: bool,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapResultDto {
    pub success: bool,
    pub image: Option<String>,
    pub spawn_x: i32,
    pub spawn_z: i32,
    pub msg: Option<String>,
}

fn sanitize_world_name(name: &str) -> Result<String, AppError> {
    let path = Path::new(name);
    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::Config("Invalid world identifier format.".to_string()))?;
    let safe_name = file_name.to_string_lossy().trim().to_string();
    if safe_name.is_empty()
        || safe_name.contains('/')
        || safe_name.contains('\\')
        || safe_name == ".."
        || safe_name == "."
    {
        return Err(AppError::Config(
            "Security violation: path traversal detected in world identifier.".to_string(),
        ));
    }
    Ok(safe_name)
}

#[tauri::command]
pub async fn get_worlds_catalog() -> Result<Vec<WorldCardDto>, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        if !saves_dir.exists() {
            let _ = fs::create_dir_all(&saves_dir);
            return Ok(Vec::new());
        }

        let raw_worlds = crate::world_manager::WorldManager::get_worlds(saves_dir.to_str().unwrap_or(""));
        let mut result = Vec::new();

        for w in raw_worlds {
            let world_name = w["name"].as_str().unwrap_or("World").to_string();
            let is_locked = saves_dir.join(&world_name).join("session.lock").exists();

            result.push(WorldCardDto {
                name: world_name,
                seed: w["seed"].as_str().unwrap_or("Unknown").to_string(),
                mode: w["mode"].as_str().unwrap_or("Survival").to_string(),
                hardcore: w["hardcore"].as_bool().unwrap_or(false),
                difficulty: w["difficulty"].as_str().unwrap_or("Normal").to_string(),
                day_count: w["day_count"].as_i64().unwrap_or(0),
                mc_version: w["mc_version"].as_str().unwrap_or("1.21").to_string(),
                spawn_x: w["spawn_x"].as_i64().unwrap_or(0) as i32,
                spawn_y: w["spawn_y"].as_i64().unwrap_or(64) as i32,
                spawn_z: w["spawn_z"].as_i64().unwrap_or(0) as i32,
                size_mb: w["size_mb"].as_f64().unwrap_or(0.0),
                last_played: w["last_played"].as_str().unwrap_or("Recently").to_string(),
                datapacks: w["datapacks"].as_u64().unwrap_or(0) as usize,
                icon: w["icon"].as_str().unwrap_or("").to_string(),
                is_locked,
            });
        }

        Ok(result)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn heal_world_entity_player(world_name: String) -> Result<WorldActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");

        let res = crate::world_manager::WorldManager::heal_world_player(
            saves_dir.to_str().unwrap_or(""),
            &safe_name,
        );

        let success = res["success"].as_bool().unwrap_or(false);
        let msg = res["msg"]
            .as_str()
            .unwrap_or("Patient recovery protocol failed.")
            .to_string();

        Ok(WorldActionResultDto { success, msg })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn toggle_world_gamemode(
    world_name: String,
    target_mode: i32,
) -> Result<WorldActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let level_dat = saves_dir.join(&safe_name).join("level.dat");

        if !level_dat.exists() {
            return Ok(WorldActionResultDto {
                success: false,
                msg: "level.dat not found in target world.".to_string(),
            });
        }

        let file = File::open(&level_dat)?;
        let mut decoder = GzDecoder::new(file);
        let mut raw_bytes = Vec::new();
        decoder.read_to_end(&mut raw_bytes)?;

        let mut nbt = fastnbt::from_bytes::<HashMap<String, NbtValue>>(&raw_bytes)
            .map_err(|e| AppError::Config(format!("NBT parse failure: {}", e)))?;

        let mode_name = match target_mode {
            0 => "Survival",
            1 => "Creative",
            2 => "Adventure",
            3 => "Spectator",
            _ => "Survival",
        };

        if let Some(NbtValue::Compound(root_inner)) = nbt.get_mut("") {
            if let Some(NbtValue::Compound(data)) = root_inner.get_mut("Data") {
                data.insert("GameType".to_string(), NbtValue::Int(target_mode));
            }
        } else if let Some(NbtValue::Compound(data)) = nbt.get_mut("Data") {
            data.insert("GameType".to_string(), NbtValue::Int(target_mode));
        }

        let serialized = fastnbt::to_bytes(&nbt)
            .map_err(|e| AppError::Config(format!("NBT encode failure: {}", e)))?;

        let temp_dat = saves_dir.join(&safe_name).join("level.dat_modetmp");
        let out_file = File::create(&temp_dat)?;
        let mut encoder = GzEncoder::new(out_file, Compression::default());
        encoder.write_all(&serialized)?;
        encoder.finish()?;

        fs::rename(temp_dat, level_dat)?;

        Ok(WorldActionResultDto {
            success: true,
            msg: format!("World '{}' gamemode switched to {}.", safe_name, mode_name),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn clone_saved_world(world_name: String) -> Result<WorldActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let src_path = saves_dir.join(&safe_name);

        if !src_path.exists() {
            return Ok(WorldActionResultDto {
                success: false,
                msg: "Source world does not exist.".to_string(),
            });
        }

        let mut clone_idx = 1;
        let mut clone_name = format!("{}_Clone", safe_name);
        while saves_dir.join(&clone_name).exists() {
            clone_idx += 1;
            clone_name = format!("{}_Clone_{}", safe_name, clone_idx);
        }

        let dest_path = saves_dir.join(&clone_name);
        fs::create_dir_all(&dest_path)?;

        for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&src_path) {
                if rel.to_string_lossy() == "session.lock" {
                    continue;
                }
                let target = dest_path.join(rel);
                if path.is_dir() {
                    let _ = fs::create_dir_all(&target);
                } else if path.is_file() {
                    let _ = fs::copy(path, &target);
                }
            }
        }

        Ok(WorldActionResultDto {
            success: true,
            msg: format!("Universe successfully cloned into '{}'.", clone_name),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn delete_saved_world(world_name: String) -> Result<WorldActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");

        let res = crate::world_manager::WorldManager::delete_world(
            saves_dir.to_str().unwrap_or(""),
            &safe_name,
        );

        let success = res["success"].as_bool().unwrap_or(false);
        let msg = res["msg"]
            .as_str()
            .unwrap_or("Failed to delete world.")
            .to_string();

        Ok(WorldActionResultDto { success, msg })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn capture_world_vcs_commit(
    world_name: String,
    message: Option<String>,
) -> Result<VcsCommitDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");

        let res = crate::vcs_manager::VCSManager::commit(
            saves_dir.to_str().unwrap_or(""),
            backups_dir.to_str().unwrap_or(""),
            &safe_name,
        );

        if let Some(err) = res.get("error").and_then(|e| e.as_str()) {
            return Err(AppError::Config(err.to_string()));
        }

        let commit_id = res["id"].as_str().unwrap_or("unknown").to_string();
        let timestamp = res["timestamp"].as_str().unwrap_or("").to_string();
        let custom_msg = message.unwrap_or_else(|| "Automated differential snapshot".to_string());

        let tree = res["tree"].as_object().map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

        Ok(VcsCommitDto {
            id: commit_id,
            message: custom_msg,
            timestamp,
            tree,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn get_world_vcs_history(world_name: String) -> Result<Vec<VcsCommitDto>, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match sanitize_world_name(&world_name) {
            Ok(name) => name,
            Err(_) => return Ok(Vec::new()),
        };

        let cfg = config::load_app_config();
        let backups_dir = Path::new(&cfg.current_instance).join("backups_devkit");
        let logs = crate::vcs_manager::VCSManager::get_history(
            backups_dir.to_str().unwrap_or(""),
            &safe_name,
        );

        let mut commits = Vec::new();
        for c in logs {
            let id = c["id"].as_str().unwrap_or("").to_string();
            let timestamp = c["timestamp"].as_str().unwrap_or("").to_string();
            let message = c["message"]
                .as_str()
                .unwrap_or("Manual snapshot")
                .to_string();

            let tree = c["tree"].as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            });

            commits.push(VcsCommitDto {
                id,
                message,
                timestamp,
                tree,
            });
        }

        Ok(commits)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn restore_world_vcs_commit(
    world_name: String,
    commit_id: String,
) -> Result<WorldActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let clean_commit_id = commit_id.trim();

        if clean_commit_id.is_empty() || !clean_commit_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(WorldActionResultDto {
                success: false,
                msg: "Invalid commit hexadecimal format.".to_string(),
            });
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
            Ok(WorldActionResultDto {
                success: true,
                msg: format!(
                    "World '{}' successfully reverted to snapshot {}.",
                    safe_name, clean_commit_id
                ),
            })
        } else {
            Ok(WorldActionResultDto {
                success: false,
                msg: "Failed to restore world state from VCS object storage.".to_string(),
            })
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn generate_world_satellite_map(
    world_name: String,
    radius: Option<i32>,
) -> Result<WorldMapResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = sanitize_world_name(&world_name)?;
        let cfg = config::load_app_config();
        let saves_dir = Path::new(&cfg.current_instance).join("saves");
        let world_path = saves_dir.join(&safe_name);

        let cm = crate::cartographer_manager::CartographerManager::new();
        let target_radius = radius.unwrap_or(1).clamp(1, 3);
        let b64 = cm.generate_map(saves_dir.to_str().unwrap_or(""), &safe_name, target_radius);

        let mut spawn_x = 0;
        let mut spawn_z = 0;
        let level_dat = world_path.join("level.dat");

        if level_dat.exists() {
            if let Ok(file) = File::open(&level_dat) {
                let mut decoder = GzDecoder::new(file);
                let mut raw = Vec::new();
                if decoder.read_to_end(&mut raw).is_ok() {
                    if let Ok(nbt) = fastnbt::from_bytes::<HashMap<String, NbtValue>>(&raw) {
                        let data_compound = if let Some(NbtValue::Compound(root_inner)) = nbt.get("") {
                            root_inner.get("Data").and_then(|v| match v {
                                NbtValue::Compound(d) => Some(d),
                                _ => None,
                            })
                        } else {
                            nbt.get("Data").and_then(|v| match v {
                                NbtValue::Compound(d) => Some(d),
                                _ => None,
                            })
                        };

                        if let Some(data) = data_compound {
                            if let Some(NbtValue::Int(sx)) = data.get("SpawnX") {
                                spawn_x = *sx;
                            }
                            if let Some(NbtValue::Int(sz)) = data.get("SpawnZ") {
                                spawn_z = *sz;
                            }
                        }
                    }
                }
            }
        }

        if !b64.is_empty() {
            Ok(WorldMapResultDto {
                success: true,
                image: Some(b64),
                spawn_x,
                spawn_z,
                msg: None,
            })
        } else {
            Ok(WorldMapResultDto {
                success: false,
                image: None,
                spawn_x: 0,
                spawn_z: 0,
                msg: Some("Cartographer unpopulated: no generated MCA chunk regions found.".to_string()),
            })
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}