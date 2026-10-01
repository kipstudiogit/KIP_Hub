use std::fs::{self, File};
use std::io::{Read, Cursor};
use std::path::Path;
use sha1::{Sha1, Digest};
use zip::ZipArchive;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::Value;
use image::ImageFormat;

pub struct ModManager;

impl ModManager {
    pub fn get_sha1(filepath: &str) -> String {
        let mut file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 65536];
        while let Ok(count) = file.read(&mut buffer) {
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let hash = hasher.finalize();
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn extract_icon(jar_path: &str) -> String {
        if let Ok(metadata) = fs::metadata(jar_path) {
            if metadata.len() > 262144000 {
                return String::new();
            }
        } else {
            return String::new();
        }

        let file = match File::open(jar_path) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return String::new(),
        };

        let mut icon_path = String::new();

        if let Ok(mut mod_json) = archive.by_name("fabric.mod.json") {
            if mod_json.size() < 10485760 {
                let mut contents = String::new();
                if mod_json.read_to_string(&mut contents).is_ok() {
                    if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                        if let Some(icon) = json.get("icon").and_then(|v| v.as_str()) {
                            icon_path = icon.to_string();
                        }
                    }
                }
            }
        }

        if icon_path.is_empty() {
            if let Ok(mut mod_json) = archive.by_name("quilt.mod.json") {
                if mod_json.size() < 10485760 {
                    let mut contents = String::new();
                    if mod_json.read_to_string(&mut contents).is_ok() {
                        if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                            if let Some(icon) = json.get("quilt_loader").and_then(|v| v.get("metadata")).and_then(|v| v.get("icon")).and_then(|v| v.as_str()) {
                                icon_path = icon.to_string();
                            }
                        }
                    }
                }
            }
        }

        if icon_path.is_empty() {
            let targets = ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"];
            for target in targets {
                let mut found = false;
                if let Ok(mut toml_file) = archive.by_name(target) {
                    if toml_file.size() < 10485760 {
                        let mut contents = String::new();
                        if toml_file.read_to_string(&mut contents).is_ok() {
                            if let Some(start) = contents.find("logoFile=\"") {
                                let sub = &contents[start + 10..];
                                if let Some(end) = sub.find('"') {
                                    icon_path = sub[..end].to_string();
                                    found = true;
                                }
                            }
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }

        if !icon_path.is_empty() {
            if let Ok(mut icon_file) = archive.by_name(&icon_path) {
                if icon_file.size() < 10485760 {
                    let mut buffer = Vec::new();
                    if icon_file.read_to_end(&mut buffer).is_ok() {
                        return Self::image_to_base64(&buffer);
                    }
                }
            }
        }

        let fallbacks = ["icon.png", "logo.png", "pack.png"];
        for fallback in fallbacks {
            if let Ok(mut icon_file) = archive.by_name(fallback) {
                if icon_file.size() < 10485760 {
                    let mut buffer = Vec::new();
                    if icon_file.read_to_end(&mut buffer).is_ok() {
                        return Self::image_to_base64(&buffer);
                    }
                }
            }
        }

        String::new()
    }

    fn image_to_base64(buffer: &[u8]) -> String {
        if let Ok(img) = image::load_from_memory(buffer) {
            let mut out_buffer = Cursor::new(Vec::new());
            if img.write_to(&mut out_buffer, ImageFormat::Png).is_ok() {
                let b64 = BASE64.encode(out_buffer.into_inner());
                return format!("data:image/png;base64,{}", b64);
            }
        }
        String::new()
    }

    pub fn toggle_mod(filepath: &str) -> bool {
        let path = Path::new(filepath);
        if !path.exists() {
            return false;
        }
        
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let new_path = if filename.ends_with(".disabled") {
            path.with_file_name(filename.replace(".disabled", ""))
        } else {
            path.with_file_name(format!("{}.disabled", filename))
        };

        fs::rename(path, new_path).is_ok()
    }

    pub fn delete_mod(filepath: &str) -> bool {
        let path = Path::new(filepath);
        if path.exists() {
            fs::remove_file(path).is_ok()
        } else {
            false
        }
    }
}