pub mod libraries;
pub mod loaders;
pub mod process;

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::launch::libraries::LibraryManager;

pub struct ManifestMerger;

impl ManifestMerger {
    pub fn merge(child: &Value, parent: &Value) -> Value {
        let mut merged = parent.clone();

        if let Some(id) = child["id"].as_str() {
            merged["id"] = json!(id);
        }
        if let Some(main_class) = child["mainClass"].as_str() {
            merged["mainClass"] = json!(main_class);
        }

        if let Some(minecraft_arguments) = child["minecraftArguments"].as_str() {
            merged["minecraftArguments"] = json!(minecraft_arguments);
            if let Some(args_obj) = merged.get_mut("arguments").and_then(|a| a.as_object_mut()) {
                args_obj.remove("game");
            }
        }

        if let Some(jar) = child["jar"].as_str() {
            merged["jar"] = json!(jar);
        }
        if let Some(assets) = child["assets"].as_str() {
            merged["assets"] = json!(assets);
        }
        if let Some(asset_index) = child.get("assetIndex") {
            merged["assetIndex"] = asset_index.clone();
        }

        if let (Some(p_libs), Some(c_libs)) = (merged["libraries"].as_array_mut(), child["libraries"].as_array()) {
            let mut existing_keys = HashMap::new();
            for (idx, lib) in p_libs.iter().enumerate() {
                if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    existing_keys.insert(LibraryManager::get_library_key(name), idx);
                }
            }

            for c_lib in c_libs {
                if let Some(name) = c_lib.get("name").and_then(|n| n.as_str()) {
                    let key = LibraryManager::get_library_key(name);
                    if let Some(&idx) = existing_keys.get(&key) {
                        p_libs[idx] = c_lib.clone();
                    } else {
                        p_libs.push(c_lib.clone());
                    }
                } else {
                    p_libs.push(c_lib.clone());
                }
            }
        } else if child["libraries"].is_array() {
            merged["libraries"] = child["libraries"].clone();
        }

        if let Some(c_args) = child.get("arguments") {
            if let Some(c_jvm) = c_args.get("jvm").and_then(|j| j.as_array()) {
                if let Some(p_jvm) = merged["arguments"]["jvm"].as_array_mut() {
                    let mut combined_jvm = Vec::new();
                    for arg in c_jvm {
                        combined_jvm.push(arg.clone());
                    }
                    for arg in p_jvm.iter() {
                        combined_jvm.push(arg.clone());
                    }
                    merged["arguments"]["jvm"] = json!(combined_jvm);
                } else {
                    merged["arguments"]["jvm"] = json!(c_jvm);
                }
            }
            if let Some(c_game) = c_args.get("game").and_then(|g| g.as_array()) {
                if let Some(p_game) = merged["arguments"]["game"].as_array_mut() {
                    let mut combined_game = Vec::new();
                    for arg in c_game {
                        combined_game.push(arg.clone());
                    }
                    for arg in p_game.iter() {
                        combined_game.push(arg.clone());
                    }
                    merged["arguments"]["game"] = json!(combined_game);
                } else {
                    merged["arguments"]["game"] = json!(c_game);
                }
            }
        }

        merged
    }
}