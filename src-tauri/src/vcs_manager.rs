use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};
use serde_json::{json, Value};
use chrono::Utc;
use walkdir::WalkDir;

pub struct VCSManager;

impl VCSManager {
    fn get_hash(filepath: &Path) -> Option<String> {
        let mut file = File::open(filepath).ok()?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        while let Ok(count) = file.read(&mut buffer) {
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let hash = hasher.finalize();
        Some(hash.iter().map(|b| format!("{:02x}", b)).collect())
    }

    fn read_logs(log_path: &Path) -> Vec<Value> {
        if log_path.exists() {
            if let Ok(content) = fs::read_to_string(log_path) {
                if let Ok(Value::Array(logs)) = serde_json::from_str(&content) {
                    return logs;
                }
            }
        }
        Vec::new()
    }

    pub fn commit(saves_dir: &str, backups_dir: &str, world_name: &str) -> Value {
        let world_path = Path::new(saves_dir).join(world_name);
        let vcs_dir = Path::new(backups_dir).join("vcs").join(world_name);
        let obj_dir = vcs_dir.join("objects");

        if let Err(e) = fs::create_dir_all(&obj_dir) {
            return json!({ "error": e.to_string() });
        }

        let mut tree = BTreeMap::new();

        for entry in WalkDir::new(&world_path).into_iter().filter_map(|e| e.ok()) {
            let fp = entry.path();
            if fp.is_file() {
                if let Some(name) = fp.file_name().and_then(|n| n.to_str()) {
                    if name == "session.lock" {
                        continue;
                    }
                }

                if let Ok(rel_path) = fp.strip_prefix(&world_path) {
                    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                    if let Some(file_hash) = Self::get_hash(fp) {
                        let obj_path = obj_dir.join(&file_hash);
                        if !obj_path.exists() {
                            let _ = fs::copy(fp, &obj_path);
                        }
                        tree.insert(rel_path_str, file_hash);
                    }
                }
            }
        }

        let tree_json = serde_json::to_string(&tree).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(tree_json.as_bytes());
        let hash = hasher.finalize();
        let hex_str: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        let commit_id = hex_str[..12].to_string();

        let commit_data = json!({
            "id": commit_id,
            "timestamp": Utc::now().to_rfc3339(),
            "tree": tree
        });

        let log_path = vcs_dir.join("commits.json");
        let mut logs = Self::read_logs(&log_path);

        if !logs.iter().any(|c| c["id"].as_str() == Some(&commit_id)) {
            logs.insert(0, commit_data.clone());
            if let Ok(json_str) = serde_json::to_string_pretty(&logs) {
                let _ = fs::write(&log_path, json_str);
            }
        }

        commit_data
    }

    pub fn get_history(backups_dir: &str, world_name: &str) -> Vec<Value> {
        let log_path = Path::new(backups_dir).join("vcs").join(world_name).join("commits.json");
        Self::read_logs(&log_path)
    }

    pub fn checkout(saves_dir: &str, backups_dir: &str, world_name: &str, commit_id: &str) -> bool {
        let vcs_dir = Path::new(backups_dir).join("vcs").join(world_name);
        let log_path = vcs_dir.join("commits.json");
        
        let logs = Self::read_logs(&log_path);
        let target_commit = logs.into_iter().find(|c| c["id"].as_str() == Some(commit_id));

        if let Some(commit) = target_commit {
            let world_path = Path::new(saves_dir).join(world_name);
            let recovery_path = Path::new(saves_dir).join(format!("{}_recovery_temp", world_name));

            if world_path.exists() {
                if fs::rename(&world_path, &recovery_path).is_err() {
                    return false;
                }
            }

            if fs::create_dir_all(&world_path).is_err() {
                let _ = fs::rename(&recovery_path, &world_path);
                return false;
            }

            let obj_dir = vcs_dir.join("objects");
            let mut restore_success = true;

            if let Some(tree) = commit["tree"].as_object() {
                for (rel_path, file_hash_val) in tree {
                    if let Some(file_hash) = file_hash_val.as_str() {
                        let dest = world_path.join(rel_path);
                        if let Some(parent) = dest.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let src = obj_dir.join(file_hash);
                        if src.exists() {
                            if fs::copy(&src, &dest).is_err() {
                                restore_success = false;
                                break;
                            }
                        } else {
                            restore_success = false;
                            break;
                        }
                    }
                }
            }

            if restore_success {
                if recovery_path.exists() {
                    let _ = fs::remove_dir_all(&recovery_path);
                }
                true
            } else {
                let _ = fs::remove_dir_all(&world_path);
                let _ = fs::rename(&recovery_path, &world_path);
                false
            }
        } else {
            false
        }
    }
}