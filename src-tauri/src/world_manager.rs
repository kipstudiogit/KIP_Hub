use std::fs::{self, File};
use std::path::Path;
use std::io::{Cursor, Read};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};
use image::ImageFormat;
use fastnbt::Value as NbtValue;
use flate2::read::GzDecoder;

pub struct WorldManager;

impl WorldManager {
    pub fn get_world_info(world_path: &str) -> Value {
        let mut seed = "Unknown".to_string();
        let mut mode = "Unknown".to_string();
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

                            if let Some(NbtValue::Int(gametype)) = data.get("GameType") {
                                mode = match gametype {
                                    0 => "Survival".to_string(),
                                    1 => "Creative".to_string(),
                                    2 => "Adventure".to_string(),
                                    3 => "Spectator".to_string(),
                                    _ => "Unknown".to_string(),
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
                        "datapacks": info["datapacks"].as_array().map(|a| a.len()).unwrap_or(0),
                        "icon": icon_b64
                    }));
                }
            }
        }
        worlds
    }

    pub fn delete_world(saves_dir: &str, world_name: &str) -> Value {
        let world_path = Path::new(saves_dir).join(world_name);
        if world_path.exists() {
            if fs::remove_dir_all(world_path).is_ok() {
                json!({ "success": true, "msg": format!("World {} deleted.", world_name) })
            } else {
                json!({ "success": false, "msg": "Failed to delete world directory." })
            }
        } else {
            json!({ "success": false, "msg": "World not found." })
        }
    }
}