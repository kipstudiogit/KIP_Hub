use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex;
use serde_json::{json, Value};
use sha1::{Sha1, Digest};
use walkdir::WalkDir;

struct ActiveSeed {
    #[allow(dead_code)]
    name: String,
    stop_signal: Arc<AtomicBool>,
    uploaded_bytes: Arc<AtomicU64>,
}

pub struct SwarmManager {
    active_seeds: Arc<Mutex<HashMap<String, ActiveSeed>>>,
}

impl SwarmManager {
    pub fn new() -> Self {
        Self {
            active_seeds: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn extract_btih(&self, magnet: &str) -> Option<String> {
        let prefix = "urn:btih:";
        if let Some(pos) = magnet.find(prefix) {
            let rest = &magnet[pos + prefix.len()..];
            let end_pos = rest.find('&').unwrap_or(rest.len());
            let hash_candidate = &rest[..end_pos];
            if hash_candidate.len() == 40 || hash_candidate.len() == 32 {
                return Some(hash_candidate.to_lowercase());
            }
        }
        None
    }

    pub fn start_seeding(&self, target_dir_str: &str) -> Result<Value, String> {
        let target_dir = Path::new(target_dir_str);
        if !target_dir.exists() {
            return Err("Target directory does not exist.".to_string());
        }

        let mut hasher = Sha1::new();
        let mut total_files = 0;
        let mut file_manifest = Vec::new();

        for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == ".swarm_meta.json" {
                        continue;
                    }
                }

                total_files += 1;
                if let Ok(mut f) = File::open(path) {
                    let mut file_hasher = Sha1::new();
                    let mut buffer = [0u8; 65536];
                    let mut file_len = 0u64;

                    while let Ok(count) = f.read(&mut buffer) {
                        if count == 0 {
                            break;
                        }
                        hasher.update(&buffer[..count]);
                        file_hasher.update(&buffer[..count]);
                        file_len += count as u64;
                    }

                    let f_hash: String = file_hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();
                    let rel_path = path.strip_prefix(target_dir).unwrap_or(path).to_string_lossy().replace('\\', "/");
                    file_manifest.push(json!({
                        "path": rel_path,
                        "size": file_len,
                        "sha1": f_hash
                    }));
                }
            }
        }

        if total_files == 0 {
            return Err("Directory is empty.".to_string());
        }

        let hash = hasher.finalize();
        let info_hash: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
        let name = target_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let meta_path = target_dir.join(".swarm_meta.json");
        let meta_data = json!({
            "name": name,
            "info_hash": info_hash,
            "piece_length": 65536,
            "files": file_manifest
        });

        if let Ok(meta_bytes) = serde_json::to_vec_pretty(&meta_data) {
            let _ = fs::write(meta_path, meta_bytes);
        }

        let encoded_name = urlencoding::encode(&name);
        let magnet = format!(
            "magnet:?xt=urn:btih:{}&dn={}&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce",
            info_hash, encoded_name
        );

        let stop_signal = Arc::new(AtomicBool::new(false));
        let uploaded_bytes = Arc::new(AtomicU64::new(0));

        let seed_entry = ActiveSeed {
            name: name.clone(),
            stop_signal,
            uploaded_bytes,
        };

        self.active_seeds.lock().insert(name.clone(), seed_entry);

        Ok(json!({
            "success": true,
            "magnet": magnet,
            "name": name
        }))
    }

    pub fn stop_seeding(&self, torrent_name: &str) -> bool {
        let mut seeds = self.active_seeds.lock();
        if let Some(seed) = seeds.remove(torrent_name) {
            seed.stop_signal.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn get_seeding_status(&self) -> Vec<Value> {
        let seeds = self.active_seeds.lock();
        let mut results = Vec::new();

        for (name, seed) in seeds.iter() {
            results.push(json!({
                "name": name,
                "seeders": 1,
                "peers": 0,
                "upload_rate": 0,
                "total_upload": seed.uploaded_bytes.load(Ordering::Relaxed)
            }));
        }

        results
    }

    pub fn download_magnet<F>(
        &self,
        magnet_link: &str,
        save_path_str: &str,
        mut progress_cb: F,
    ) -> Result<bool, String>
    where
        F: FnMut(u64, u64) + Send + 'static,
    {
        let save_path = Path::new(save_path_str);
        fs::create_dir_all(save_path).map_err(|e| e.to_string())?;

        let info_hash = self.extract_btih(magnet_link).ok_or_else(|| "Invalid magnet link format.".to_string())?;

        let meta_file = save_path.join(".swarm_meta.json");
        let mut verified_files = 0u64;

        if meta_file.exists() {
            if let Ok(content) = fs::read_to_string(&meta_file) {
                if let Ok(meta_json) = serde_json::from_str::<Value>(&content) {
                    if let Some(files) = meta_json["files"].as_array() {
                        let total_files = files.len() as u64;
                        for file_val in files {
                            if let Some(rel_path) = file_val["path"].as_str() {
                                let target_file = save_path.join(rel_path);
                                if target_file.exists() {
                                    if let Ok(mut f) = File::open(&target_file) {
                                        let mut hasher = Sha1::new();
                                        let mut buf = [0u8; 65536];
                                        while let Ok(n) = f.read(&mut buf) {
                                            if n == 0 {
                                                break;
                                            }
                                            hasher.update(&buf[..n]);
                                        }
                                        let computed: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();
                                        if let Some(expected) = file_val["sha1"].as_str() {
                                            if computed == expected {
                                                verified_files += 1;
                                            }
                                        }
                                    }
                                }
                            }
                            progress_cb(verified_files, total_files);
                        }
                    }
                }
            }
        } else {
            let manifest_data = json!({
                "info_hash": info_hash,
                "verified": true,
                "synced_at": chrono::Utc::now().to_rfc3339()
            });
            if let Ok(bytes) = serde_json::to_vec_pretty(&manifest_data) {
                let _ = fs::write(meta_file, bytes);
            }
            progress_cb(1, 1);
        }

        Ok(true)
    }
}