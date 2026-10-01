use std::fs::File;
use std::io::Read;
use std::path::Path;
use sha2::{Sha256, Digest};
use zip::ZipArchive;
use regex::bytes::Regex as BytesRegex;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};
use walkdir::WalkDir;

pub struct ShieldManager {
    webhook_pattern: BytesRegex,
    b64_pattern: BytesRegex,
    dangerous_apis: Vec<&'static [u8]>,
    stealer_keywords: Vec<&'static [u8]>,
    whitelist_hashes: Vec<&'static str>,
    ignored_prefixes: Vec<&'static str>,
}

impl ShieldManager {
    pub fn new() -> Self {
        Self {
            webhook_pattern: BytesRegex::new(r"https?://(?:ptb\.|canary\.)?discord(?:app)?\.com/api/webhooks/\d+/[a-zA-Z0-9_-]+").unwrap(),
            b64_pattern: BytesRegex::new(r"(?:[A-Za-z0-9+/]{4}){10,}(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?").unwrap(),
            dangerous_apis: vec![
                b"java/lang/Runtime.exec",
                b"java/lang/ProcessBuilder",
                b"sun/misc/Unsafe",
                b"java/lang/ClassLoader.defineClass",
            ],
            stealer_keywords: vec![
                b"LoginRadius",
                b"launcher_accounts.json",
                b"Essential/credentials.json",
                b"FileZilla/recentservers.xml",
                b"AppData/Local/Google/Chrome/User Data",
            ],
            whitelist_hashes: vec![
                "2a98f4cd5c95738ab8243302636fa0982bbfe3b52d9a5b3a4a5b6c7d8e9f0a1b",
            ],
            ignored_prefixes: vec![
                "kotlin/",
                "org/spongepowered/",
                "org/objectweb/asm/",
                "it/unimi/dsi/fastutil/",
                "io/netty/",
                "com/google/",
                "com/mojang/",
            ],
        }
    }

    pub fn get_sha256(&self, filepath: &str) -> String {
        let mut file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        while let Ok(count) = file.read(&mut buffer) {
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let hash = hasher.finalize();
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mut entropy = 0.0;
        let len = data.len() as f64;
        let mut counts = [0usize; 256];

        for &byte in data {
            counts[byte as usize] += 1;
        }

        for &count in &counts {
            if count > 0 {
                let p = (count as f64) / len;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    fn is_ignored_class(&self, name: &str) -> bool {
        self.ignored_prefixes.iter().any(|prefix| name.starts_with(prefix))
    }

    fn scan_content(&self, data: &[u8], filename: &str, is_essential: bool, safe: &mut bool, threats: &mut Vec<String>) {
        if self.is_ignored_class(filename) {
            return;
        }

        for _ in self.webhook_pattern.find_iter(data) {
            *safe = false;
            threats.push(format!("Discord Webhook embedded in {}", filename));
        }

        if !is_essential {
            for &keyword in &self.stealer_keywords {
                if data.windows(keyword.len()).any(|w| w == keyword) {
                    *safe = false;
                    let kw_str = String::from_utf8_lossy(keyword);
                    threats.push(format!("Data stealing routine (Targeting: {}) in {}", kw_str, filename));
                }
            }
        }

        let mut api_hits = 0;
        for &api in &self.dangerous_apis {
            if data.windows(api.len()).any(|w| w == api) {
                api_hits += 1;
            }
        }
        if api_hits >= 3 {
            *safe = false;
            threats.push(format!("Suspicious API cluster (RCE/Downloader capabilities) in {}", filename));
        }

        for mat in self.b64_pattern.find_iter(data) {
            if let Ok(decoded) = BASE64.decode(mat.as_bytes()) {
                if self.webhook_pattern.is_match(&decoded) {
                    *safe = false;
                    threats.push(format!("Obfuscated Discord Webhook in {}", filename));
                }
            }
        }
    }

    pub fn scan_file(&self, filepath: &str) -> Value {
        let path = Path::new(filepath);
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let mut safe = true;
        let mut threats = Vec::new();
        let mut error = None;

        if !path.exists() {
            return json!({ "file": file_name, "safe": true, "threats": threats, "hash": "", "error": "File not found" });
        }

        let file_hash = self.get_sha256(filepath);
        if self.whitelist_hashes.contains(&file_hash.as_str()) {
            return json!({ "file": file_name, "safe": true, "threats": threats, "hash": file_hash, "error": Value::Null });
        }

        match File::open(path) {
            Ok(file) => {
                match ZipArchive::new(file) {
                    Ok(mut archive) => {
                        let mut is_essential = false;
                        let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

                        if file_names.contains(&"essential/metadata.json".to_string()) || file_names.contains(&"gg/essential/Essential.class".to_string()) {
                            is_essential = true;
                        }

                        for i in 0..archive.len() {
                            if let Ok(mut zfile) = archive.by_index(i) {
                                let name = zfile.name().to_string();

                                if name.ends_with(".class") || name.ends_with(".json") || name.ends_with(".txt") {
                                    if zfile.size() <= 52428800 {
                                        let mut raw_data = Vec::new();
                                        if zfile.read_to_end(&mut raw_data).is_ok() {
                                            self.scan_content(&raw_data, &name, is_essential, &mut safe, &mut threats);

                                            if name.ends_with(".class") && raw_data.len() > 8192 && !self.is_ignored_class(&name) {
                                                if self.calculate_entropy(&raw_data) > 7.96 {
                                                    safe = false;
                                                    threats.push(format!("Encrypted payload detected in {}", name));
                                                }
                                            }
                                        }
                                    }
                                } else if name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib") || name.ends_with(".exe") {
                                    safe = false;
                                    threats.push(format!("Native OS executable packed inside Java mod: {}", name));
                                } else if name.ends_with(".jar") && !name.starts_with("META-INF/") && !name.starts_with("fabric-") && !name.contains("jars/") && !name.contains("jij/") {
                                    safe = false;
                                    threats.push(format!("Suspicious nested JAR (Dropper behavior): {}", name));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        error = Some("Not a valid ZIP archive".to_string());
                    }
                }
            }
            Err(e) => {
                error = Some(e.to_string());
            }
        }

        threats.sort();
        threats.dedup();

        json!({
            "file": file_name,
            "safe": safe,
            "threats": threats,
            "hash": file_hash,
            "error": error
        })
    }

    pub fn scan_directory(&self, dir_path: &str) -> Vec<Value> {
        let mut results = Vec::new();
        let path = Path::new(dir_path);

        if !path.exists() {
            return results;
        }

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "jar" {
                        let res = self.scan_file(file_path.to_str().unwrap_or(""));
                        if !res["safe"].as_bool().unwrap_or(true) || !res["error"].is_null() {
                            results.push(res);
                        }
                    }
                }
            }
        }
        results
    }
}