use std::fs::{self, File};
use std::path::Path;
use std::io::{Cursor, Read, Write};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use image::ImageFormat;
use fastnbt::Value as NbtValue;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use walkdir::WalkDir;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCardData {
    pub name: String,
    pub seed: String,
    pub mode: String,
    pub hardcore: bool,
    pub difficulty: String,
    pub day_count: i64,
    pub mc_version: String,
    pub size_mb: f64,
    pub last_played: String,
    pub datapacks: usize,
    pub icon: String,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
}

pub struct WorldManager;

impl WorldManager {
    fn calculate_folder_size_mb(path: &Path) -> f64 {
        let mut total_bytes = 0u64;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_bytes += meta.len();
                }
            }
        }
        ((total_bytes as f64) / (1024.0 * 1024.0) * 10.0).round() / 10.0
    }

    pub fn get_world_info(world_path: &str) -> Value {
        let mut seed = "Unknown".to_string();
        let mut mode = "Survival".to_string();
        let mut hardcore = false;
        let mut difficulty = "Normal".to_string();
        let mut day_count = 0i64;
        let mut mc_version = "1.21.1".to_string();
        let mut spawn_x = 0i32;
        let mut spawn_y = 64i32;
        let mut spawn_z = 0i32;
        let mut datapacks = Vec::new();

        let level_dat = Path::new(world_path).join("level.dat");

        if level_dat.exists() {
            if let Ok(file) = File::open(&level_dat) {
                let mut decoder = GzDecoder::new(file);
                let mut buffer = Vec::new();
                if decoder.read_to_end(&mut buffer).is_ok() {
                    if let Ok(nbt) = fastnbt::from_bytes::<std::collections::HashMap<String, NbtValue>>(&buffer) {
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
                            if let Some(NbtValue::Compound(world_gen)) = data.get("WorldGenSettings") {
                                if let Some(NbtValue::Long(s)) = world_gen.get("seed") {
                                    seed = s.to_string();
                                }
                            } else if let Some(NbtValue::Long(s)) = data.get("RandomSeed") {
                                seed = s.to_string();
                            }

                            if let Some(NbtValue::Byte(hc)) = data.get("hardcore") {
                                hardcore = *hc == 1;
                            }

                            if let Some(NbtValue::Byte(diff)) = data.get("Difficulty") {
                                difficulty = match diff {
                                    0 => "Peaceful".to_string(),
                                    1 => "Easy".to_string(),
                                    2 => "Normal".to_string(),
                                    3 => "Hard".to_string(),
                                    _ => "Normal".to_string(),
                                };
                            }

                            if let Some(NbtValue::Long(day_time)) = data.get("DayTime") {
                                day_count = (day_time / 24000).max(0);
                            } else if let Some(NbtValue::Long(time)) = data.get("Time") {
                                day_count = (time / 24000).max(0);
                            }

                            if let Some(NbtValue::Compound(ver)) = data.get("Version") {
                                if let Some(NbtValue::String(ver_name)) = ver.get("Name") {
                                    mc_version = ver_name.clone();
                                }
                            }

                            if let Some(NbtValue::Int(sx)) = data.get("SpawnX") {
                                spawn_x = *sx;
                            }
                            if let Some(NbtValue::Int(sy)) = data.get("SpawnY") {
                                spawn_y = *sy;
                            }
                            if let Some(NbtValue::Int(sz)) = data.get("SpawnZ") {
                                spawn_z = *sz;
                            }

                            if let Some(NbtValue::Int(gametype)) = data.get("GameType") {
                                mode = match gametype {
                                    0 => "Survival".to_string(),
                                    1 => "Creative".to_string(),
                                    2 => "Adventure".to_string(),
                                    3 => "Spectator".to_string(),
                                    _ => "Survival".to_string(),
                                };
                            }

                            if let Some(NbtValue::Compound(dp_data)) = data.get("DataPacks") {
                                if let Some(NbtValue::List(enabled)) = dp_data.get("Enabled") {
                                    for item in enabled {
                                        if let NbtValue::String(dp_name) = item {
                                            if dp_name != "vanilla" {
                                                datapacks.push(dp_name.clone());
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

        json!({
            "seed": seed,
            "mode": mode,
            "hardcore": hardcore,
            "difficulty": difficulty,
            "day_count": day_count,
            "mc_version": mc_version,
            "spawn_x": spawn_x,
            "spawn_y": spawn_y,
            "spawn_z": spawn_z,
            "datapacks": datapacks
        })
    }

    pub fn get_worlds(saves_dir: &str) -> Vec<Value> {
        let mut worlds = Vec::new();
        let path = Path::new(saves_dir);

        if !path.exists() {
            return worlds;
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let world_path = entry.path();
                if world_path.is_dir() {
                    let dirname = world_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let info = Self::get_world_info(world_path.to_str().unwrap_or(""));
                    let size_mb = Self::calculate_folder_size_mb(&world_path);

                    let mut last_played = "Recently".to_string();
                    let level_dat = world_path.join("level.dat");
                    if let Ok(meta) = fs::metadata(&level_dat) {
                        if let Ok(modified) = meta.modified() {
                            let datetime: chrono::DateTime<chrono::Local> = modified.into();
                            last_played = datetime.format("%Y-%m-%d %H:%M").to_string();
                        }
                    }

                    let mut icon_b64 = String::new();
                    let icon_path = world_path.join("icon.png");

                    if icon_path.exists() {
                        if let Ok(mut file) = File::open(&icon_path) {
                            let mut buffer = Vec::new();
                            if file.read_to_end(&mut buffer).is_ok() {
                                if let Ok(img) = image::load_from_memory(&buffer) {
                                    let mut out_buffer = Cursor::new(Vec::new());
                                    if img.write_to(&mut out_buffer, ImageFormat::Png).is_ok() {
                                        icon_b64 = format!("data:image/png;base64,{}", BASE64.encode(out_buffer.into_inner()));
                                    }
                                }
                            }
                        }
                    }

                    worlds.push(json!({
                        "name": dirname,
                        "seed": info["seed"],
                        "mode": info["mode"],
                        "hardcore": info["hardcore"],
                        "difficulty": info["difficulty"],
                        "day_count": info["day_count"],
                        "mc_version": info["mc_version"],
                        "spawn_x": info["spawn_x"],
                        "spawn_y": info["spawn_y"],
                        "spawn_z": info["spawn_z"],
                        "size_mb": size_mb,
                        "last_played": last_played,
                        "datapacks": info["datapacks"].as_array().map(|a| a.len()).unwrap_or(0),
                        "icon": icon_b64
                    }));
                }
            }
        }

        worlds.sort_by(|a, b| {
            let a_time = a["last_played"].as_str().unwrap_or("");
            let b_time = b["last_played"].as_str().unwrap_or("");
            b_time.cmp(a_time)
        });

        worlds
    }

    pub fn heal_world_player(saves_dir: &str, world_name: &str) -> Value {
        let world_path = Path::new(saves_dir).join(world_name);
        if !world_path.exists() {
            return json!({ "success": false, "msg": "World directory does not exist." });
        }

        let lock_file = world_path.join("session.lock");
        if lock_file.exists() {
            let _ = fs::remove_file(lock_file);
        }

        let level_dat = world_path.join("level.dat");
        if !level_dat.exists() {
            let level_dat_old = world_path.join("level.dat_old");
            if level_dat_old.exists() {
                let _ = fs::copy(&level_dat_old, &level_dat);
            } else {
                return json!({ "success": false, "msg": "level.dat is missing and cannot be recovered." });
            }
        }

        let file = match File::open(&level_dat) {
            Ok(f) => f,
            Err(e) => return json!({ "success": false, "msg": format!("Cannot open level.dat: {}", e) }),
        };

        let mut decoder = GzDecoder::new(file);
        let mut raw_bytes = Vec::new();
        if decoder.read_to_end(&mut raw_bytes).is_err() {
            return json!({ "success": false, "msg": "Failed to decompress level.dat." });
        }

        let mut nbt = match fastnbt::from_bytes::<std::collections::HashMap<String, NbtValue>>(&raw_bytes) {
            Ok(data) => data,
            Err(e) => return json!({ "success": false, "msg": format!("NBT parse failure: {}", e) }),
        };

        let backup_dat = world_path.join("level.dat_rescue_backup");
        let _ = fs::copy(&level_dat, backup_dat);

        let mut spawn_x = 0f64;
        let mut spawn_y = 65f64;
        let mut spawn_z = 0f64;

        if let Some(NbtValue::Compound(root_inner)) = nbt.get_mut("") {
            if let Some(NbtValue::Compound(data)) = root_inner.get_mut("Data") {
                if let Some(NbtValue::Int(sx)) = data.get("SpawnX") { spawn_x = *sx as f64; }
                if let Some(NbtValue::Int(sy)) = data.get("SpawnY") { spawn_y = (*sy as f64).max(64.0); }
                if let Some(NbtValue::Int(sz)) = data.get("SpawnZ") { spawn_z = *sz as f64; }

                if let Some(NbtValue::Compound(player)) = data.get_mut("Player") {
                    player.insert("Health".to_string(), NbtValue::Float(20.0));
                    player.insert("foodLevel".to_string(), NbtValue::Int(20));
                    player.insert("Air".to_string(), NbtValue::Short(300));
                    player.insert("Fire".to_string(), NbtValue::Short(-20));
                    player.insert("FallDistance".to_string(), NbtValue::Float(0.0));
                    player.insert("DeathTime".to_string(), NbtValue::Short(0));
                    player.insert("HurtTime".to_string(), NbtValue::Short(0));
                    player.insert("ActiveEffects".to_string(), NbtValue::List(Vec::new()));
                    player.insert("Motion".to_string(), NbtValue::List(vec![NbtValue::Double(0.0), NbtValue::Double(0.0), NbtValue::Double(0.0)]));

                    let mut needs_teleport = false;
                    if let Some(NbtValue::List(pos_list)) = player.get("Pos") {
                        if let Some(NbtValue::Double(y)) = pos_list.get(1) {
                            if *y < -60.0 || *y > 320.0 {
                                needs_teleport = true;
                            }
                        }
                    } else {
                        needs_teleport = true;
                    }

                    if needs_teleport {
                        player.insert("Pos".to_string(), NbtValue::List(vec![
                            NbtValue::Double(spawn_x + 0.5),
                            NbtValue::Double(spawn_y + 1.5),
                            NbtValue::Double(spawn_z + 0.5),
                        ]));
                    }
                }
            }
        } else if let Some(NbtValue::Compound(data)) = nbt.get_mut("Data") {
            if let Some(NbtValue::Int(sx)) = data.get("SpawnX") { spawn_x = *sx as f64; }
            if let Some(NbtValue::Int(sy)) = data.get("SpawnY") { spawn_y = (*sy as f64).max(64.0); }
            if let Some(NbtValue::Int(sz)) = data.get("SpawnZ") { spawn_z = *sz as f64; }

            if let Some(NbtValue::Compound(player)) = data.get_mut("Player") {
                player.insert("Health".to_string(), NbtValue::Float(20.0));
                player.insert("foodLevel".to_string(), NbtValue::Int(20));
                player.insert("Air".to_string(), NbtValue::Short(300));
                player.insert("Fire".to_string(), NbtValue::Short(-20));
                player.insert("FallDistance".to_string(), NbtValue::Float(0.0));
                player.insert("DeathTime".to_string(), NbtValue::Short(0));
                player.insert("HurtTime".to_string(), NbtValue::Short(0));
                player.insert("ActiveEffects".to_string(), NbtValue::List(Vec::new()));
                player.insert("Motion".to_string(), NbtValue::List(vec![NbtValue::Double(0.0), NbtValue::Double(0.0), NbtValue::Double(0.0)]));

                let mut needs_teleport = false;
                if let Some(NbtValue::List(pos_list)) = player.get("Pos") {
                    if let Some(NbtValue::Double(y)) = pos_list.get(1) {
                        if *y < -60.0 || *y > 320.0 {
                            needs_teleport = true;
                        }
                    }
                } else {
                    needs_teleport = true;
                }

                if needs_teleport {
                    player.insert("Pos".to_string(), NbtValue::List(vec![
                        NbtValue::Double(spawn_x + 0.5),
                        NbtValue::Double(spawn_y + 1.5),
                        NbtValue::Double(spawn_z + 0.5),
                    ]));
                }
            }
        }

        let serialized_bytes = match fastnbt::to_bytes(&nbt) {
            Ok(b) => b,
            Err(e) => return json!({ "success": false, "msg": format!("NBT encode failure: {}", e) }),
        };

        let temp_level_dat = world_path.join("level.dat_tmp");
        let out_file = match File::create(&temp_level_dat) {
            Ok(f) => f,
            Err(e) => return json!({ "success": false, "msg": format!("Cannot create temporary file: {}", e) }),
        };

        let mut encoder = GzEncoder::new(out_file, Compression::default());
        if encoder.write_all(&serialized_bytes).is_err() {
            let _ = fs::remove_file(&temp_level_dat);
            return json!({ "success": false, "msg": "Failed to compress new level.dat." });
        }

        if encoder.finish().is_err() {
            let _ = fs::remove_file(&temp_level_dat);
            return json!({ "success": false, "msg": "Failed to finalize level.dat archive." });
        }

        let _ = fs::remove_file(&level_dat);
        if fs::rename(&temp_level_dat, &level_dat).is_err() {
            return json!({ "success": false, "msg": "Atomic file swap failed." });
        }

        json!({
            "success": true,
            "msg": format!("Player state recovered in '{}'. Health=20, effects purged, void fall neutralized.", world_name)
        })
    }

    pub fn delete_world(saves_dir: &str, world_name: &str) -> Value {
        let world_path = Path::new(saves_dir).join(world_name);
        if world_path.exists() {
            if fs::remove_dir_all(world_path).is_ok() {
                json!({ "success": true, "msg": format!("World {} purged successfully.", world_name) })
            } else {
                json!({ "success": false, "msg": "Failed to delete world directory." })
            }
        } else {
            json!({ "success": false, "msg": "World directory not found." })
        }
    }
}