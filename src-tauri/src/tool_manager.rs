use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use chrono::Local;
use regex::Regex;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

pub struct ToolManager;

impl ToolManager {
    pub fn clean_logs(logs_dir: &str) -> (i32, f64) {
        let path = Path::new(logs_dir);
        if !path.exists() {
            return (0, 0.0);
        }

        let mut deleted_count = 0;
        let mut freed_bytes = 0u64;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".gz") || (name.ends_with(".log") && name != "latest.log") {
                        if let Ok(metadata) = entry.metadata() {
                            let size = metadata.len();
                            if fs::remove_file(&file_path).is_ok() {
                                freed_bytes += size;
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }

        (deleted_count, (freed_bytes as f64) / (1024.0 * 1024.0))
    }

    pub fn wipe_configs(config_dir: &str) -> bool {
        let path = Path::new(config_dir);
        if path.exists() {
            fs::remove_dir_all(path).is_ok()
        } else {
            false
        }
    }

    pub fn unlock_worlds(saves_dir: &str) -> i32 {
        let path = Path::new(saves_dir);
        if !path.exists() {
            return 0;
        }

        let mut unlocked_count = 0;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "session.lock" {
                if fs::remove_file(entry.path()).is_ok() {
                    unlocked_count += 1;
                }
            }
        }
        unlocked_count
    }

    pub fn clean_world_caches(saves_dir: &str) -> i32 {
        let path = Path::new(saves_dir);
        if !path.exists() {
            return 0;
        }

        let mut caches_removed = 0;
        let cache_folders = ["xaerominimap", "xaeroworldmap", "journeymap"];

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_dir() {
                    for cache in &cache_folders {
                        let cache_path = entry.path().join(cache);
                        if cache_path.exists() {
                            if fs::remove_dir_all(&cache_path).is_ok() {
                                caches_removed += 1;
                            }
                        }
                    }
                }
            }
        }
        caches_removed
    }

    pub fn create_backup(saves_dir: &str, backups_dir: &str) -> f64 {
        let saves_path = Path::new(saves_dir);
        if !saves_path.exists() {
            return 0.0;
        }

        let backups_path = Path::new(backups_dir);
        let _ = fs::create_dir_all(backups_path);

        let date_str = Local::now().format("%Y-%m-%d_%H%M%S").to_string();
        let zip_path = backups_path.join(format!("saves_backup_{}.zip", date_str));

        let file = match File::create(&zip_path) {
            Ok(f) => f,
            Err(_) => return 0.0,
        };

        let mut zip = ZipWriter::new(file);
        let options: FileOptions<'_, ()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();

        for entry in WalkDir::new(saves_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(saves_path) {
                let name = rel.to_string_lossy().replace('\\', "/");
                if name.is_empty() {
                    continue;
                }

                if path.is_file() {
                    if zip.start_file(&name, options).is_ok() {
                        if let Ok(mut f) = File::open(path) {
                            buffer.clear();
                            if f.read_to_end(&mut buffer).is_ok() {
                                let _ = zip.write_all(&buffer);
                            }
                        }
                    }
                } else if path.is_dir() {
                    let _ = zip.add_directory(&name, options);
                }
            }
        }

        if zip.finish().is_ok() {
            if let Ok(metadata) = fs::metadata(&zip_path) {
                return (metadata.len() as f64) / (1024.0 * 1024.0);
            }
        }

        let _ = fs::remove_file(&zip_path);
        0.0
    }

    #[allow(dead_code)]
    pub fn analyze_crash_log(log_text: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let text_lower = log_text.to_lowercase();

        if text_lower.contains("outofmemoryerror") {
            hints.push("Allocated Java heap exhausted. Increase RAM allocation in settings.".to_string());
        }
        if text_lower.contains("optifine") && (text_lower.contains("mixin") || text_lower.contains("sponge")) {
            hints.push("OptiFine mixin conflict detected. Consider migrating to Sodium & Iris.".to_string());
        }
        if text_lower.contains("unsupportedclassversionerror") {
            hints.push("Java bytecode mismatch. The game was compiled with a newer OpenJDK version.".to_string());
        }
        if text_lower.contains("ticking entity") {
            hints.push("Corrupted entity ticking in chunk. World healing or backup rollback advised.".to_string());
        }
        if text_lower.contains("multiple entries with same key") {
            hints.push("Mod registry duplicate identifier conflict.".to_string());
        }
        if text_lower.contains("nosuchmethoderror") || text_lower.contains("noclassdeffounderror") {
            hints.push("Missing required API library or outdated dependency version.".to_string());
        }

        let re = Regex::new(r"(?i)suspected mods: (.*?)\n").unwrap();
        if let Some(caps) = re.captures(log_text) {
            if let Some(suspect) = caps.get(1) {
                hints.push(format!("Suspected culprit: {}", suspect.as_str().trim()));
            }
        }

        hints
    }

    pub fn scrub_personal_data(text: &str) -> String {
        let mut scrubbed = text.to_string();

        let re_win = Regex::new(r"(?i)(C:\\[Uu]sers\\)[^\\]+").unwrap();
        scrubbed = re_win.replace_all(&scrubbed, "${1}<USER>").to_string();

        let re_nix = Regex::new(r"(?i)(/home/)[^/]+").unwrap();
        scrubbed = re_nix.replace_all(&scrubbed, "${1}<USER>").to_string();

        let re_ip = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap();
        scrubbed = re_ip.replace_all(&scrubbed, "<IPV4>").to_string();

        let re_token = Regex::new(r"(?i)(token|access_token|client_secret|Authorization|RpsTicket|apikey)[\s:=]+[a-zA-Z0-9_\-\.]+").unwrap();
        scrubbed = re_token.replace_all(&scrubbed, "${1}: <HIDDEN>").to_string();

        let re_jwt = Regex::new(r"(?i)eyJ[a-zA-Z0-9_\-\.]+").unwrap();
        scrubbed = re_jwt.replace_all(&scrubbed, "<JWT_TOKEN>").to_string();

        scrubbed
    }
}