use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::Path;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Local};
use image::ImageFormat;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedModInfo {
    pub filename: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub loaders: Vec<String>,
    pub disabled: bool,
    pub icon: String,
    pub size_bytes: u64,
    pub date_modified: String,
    pub content_type: String,
    pub dependencies: Vec<String>,
}

pub struct ModManager;

impl ModManager {
    #[allow(dead_code)]
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
                            if let Some(icon) = json
                                .get("quilt_loader")
                                .and_then(|v| v.get("metadata"))
                                .and_then(|v| v.get("icon"))
                                .and_then(|v| v.as_str())
                            {
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

    pub fn parse_file(path: &Path, content_type: &str) -> DetailedModInfo {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let disabled = filename.ends_with(".disabled");
        let default_name = filename
            .replace(".jar", "")
            .replace(".disabled", "")
            .replace(".zip", "");

        let (size_bytes, date_modified) = match fs::metadata(path) {
            Ok(m) => {
                let sz = m.len();
                let dt = m
                    .modified()
                    .ok()
                    .map(|sys_time| {
                        let local_time: DateTime<Local> = sys_time.into();
                        local_time.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                (sz, dt)
            }
            Err(_) => (0, "Unknown".to_string()),
        };

        if content_type == "resourcepacks" || content_type == "shaderpacks" {
            let icon = if path.is_file() {
                Self::extract_icon(path.to_str().unwrap_or(""))
            } else {
                let icon_path = path.join("pack.png");
                if icon_path.exists() {
                    fs::read(&icon_path)
                        .ok()
                        .map(|b| Self::image_to_base64(&b))
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            };

            return DetailedModInfo {
                filename: filename.clone(),
                id: default_name.to_lowercase().replace(' ', "-"),
                name: default_name,
                version: "1.0.0".to_string(),
                author: "Community".to_string(),
                description: format!("Minecraft {} package.", content_type),
                loaders: Vec::new(),
                disabled,
                icon,
                size_bytes,
                date_modified,
                content_type: content_type.to_string(),
                dependencies: Vec::new(),
            };
        }

        let mut id = default_name.to_lowercase().replace(' ', "-");
        let mut name = default_name.clone();
        let mut version = "Unknown".to_string();
        let mut author = "Unknown".to_string();
        let mut description = String::new();
        let mut loaders = Vec::new();
        let mut dependencies = Vec::new();

        if let Ok(file) = File::open(path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                if let Ok(mut f_mod) = archive.by_name("fabric.mod.json") {
                    loaders.push("fabric".to_string());
                    let mut content = String::new();
                    if f_mod.read_to_string(&mut content).is_ok() {
                        if let Ok(data) = serde_json::from_str::<Value>(&content) {
                            if let Some(i) = data["id"].as_str() {
                                id = i.to_string();
                            }
                            if let Some(n) = data["name"].as_str() {
                                name = n.to_string();
                            }
                            if let Some(v) = data["version"].as_str() {
                                version = v.to_string();
                            }
                            if let Some(d) = data["description"].as_str() {
                                description = d.to_string();
                            }
                            if let Some(auths) = data["authors"].as_array() {
                                let mut names = Vec::new();
                                for a in auths {
                                    if let Some(s) = a.as_str() {
                                        names.push(s.to_string());
                                    } else if let Some(s) = a.get("name").and_then(|n| n.as_str()) {
                                        names.push(s.to_string());
                                    }
                                }
                                if !names.is_empty() {
                                    author = names.join(", ");
                                }
                            }
                            if let Some(deps) = data["depends"].as_object() {
                                dependencies.extend(deps.keys().cloned());
                            }
                        }
                    }
                }

                if let Ok(mut q_mod) = archive.by_name("quilt.mod.json") {
                    loaders.push("quilt".to_string());
                    let mut content = String::new();
                    if q_mod.read_to_string(&mut content).is_ok() {
                        if let Ok(data) = serde_json::from_str::<Value>(&content) {
                            let ql = &data["quilt_loader"];
                            if let Some(i) = ql["id"].as_str() {
                                id = i.to_string();
                            }
                            if let Some(v) = ql["version"].as_str() {
                                version = v.to_string();
                            }
                            if let Some(n) = ql["metadata"]["name"].as_str() {
                                name = n.to_string();
                            }
                            if let Some(d) = ql["metadata"]["description"].as_str() {
                                description = d.to_string();
                            }
                            if let Some(deps) = ql["depends"].as_array() {
                                for dp in deps {
                                    if let Some(d_id) = dp["id"].as_str() {
                                        dependencies.push(d_id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                let toml_targets = [
                    ("META-INF/neoforge.mods.toml", "neoforge"),
                    ("META-INF/mods.toml", "forge"),
                ];

                let re_id = Regex::new(r#"modId\s*=\s*"([^"]+)""#).unwrap();
                let re_name = Regex::new(r#"displayName\s*=\s*"([^"]+)""#).unwrap();
                let re_ver = Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap();
                let re_author = Regex::new(r#"authors\s*=\s*"([^"]+)""#).unwrap();
                let re_desc = Regex::new(r#"description\s*=\s*'''([^']+)'''"#).unwrap();

                for (target, loader_label) in toml_targets {
                    if let Ok(mut t_file) = archive.by_name(target) {
                        loaders.push(loader_label.to_string());
                        let mut content = String::new();
                        if t_file.read_to_string(&mut content).is_ok() {
                            if let Some(c) = re_id.captures(&content) {
                                id = c[1].to_string();
                            }
                            if let Some(c) = re_name.captures(&content) {
                                name = c[1].to_string();
                            }
                            if let Some(c) = re_ver.captures(&content) {
                                if &c[1] != "${file.jarVersion}" {
                                    version = c[1].to_string();
                                }
                            }
                            if let Some(c) = re_author.captures(&content) {
                                author = c[1].to_string();
                            }
                            if let Some(c) = re_desc.captures(&content) {
                                description = c[1].trim().to_string();
                            }
                        }
                        break;
                    }
                }
            }
        }

        loaders.dedup();
        dependencies.dedup();

        if loaders.is_empty() {
            loaders.push("mod".to_string());
        }

        let icon = Self::extract_icon(path.to_str().unwrap_or(""));

        DetailedModInfo {
            filename,
            id,
            name,
            version,
            author,
            description,
            loaders,
            disabled,
            icon,
            size_bytes,
            date_modified,
            content_type: content_type.to_string(),
            dependencies,
        }
    }

    pub fn get_content_list(mc_dir_str: &str, content_type: &str) -> Vec<DetailedModInfo> {
        let mc_dir = Path::new(mc_dir_str);
        let subfolder = match content_type {
            "resourcepacks" => "resourcepacks",
            "shaderpacks" => "shaderpacks",
            _ => "mods",
        };
        let target_dir = mc_dir.join(subfolder);
        let mut list = Vec::new();

        if let Ok(entries) = fs::read_dir(target_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                let fname = entry.file_name().to_string_lossy().to_string();
                if subfolder == "mods" {
                    if fname.ends_with(".jar") || fname.ends_with(".jar.disabled") {
                        list.push(Self::parse_file(&p, content_type));
                    }
                } else if p.is_file() || p.is_dir() {
                    list.push(Self::parse_file(&p, content_type));
                }
            }
        }

        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list
    }

    pub fn toggle_file(filepath: &str) -> bool {
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

    pub fn delete_file(filepath: &str) -> bool {
        let path = Path::new(filepath);
        if path.is_file() {
            fs::remove_file(path).is_ok()
        } else if path.is_dir() {
            fs::remove_dir_all(path).is_ok()
        } else {
            false
        }
    }

    pub fn batch_toggle(mc_dir_str: &str, filenames: &[String], enable: bool, content_type: &str) -> usize {
        let mc_dir = Path::new(mc_dir_str);
        let subfolder = match content_type {
            "resourcepacks" => "resourcepacks",
            "shaderpacks" => "shaderpacks",
            _ => "mods",
        };
        let dir = mc_dir.join(subfolder);
        let mut count = 0;

        for fname in filenames {
            let p = dir.join(fname);
            let disabled = fname.ends_with(".disabled");
            if enable && disabled {
                let target = dir.join(fname.replace(".disabled", ""));
                if fs::rename(&p, target).is_ok() {
                    count += 1;
                }
            } else if !enable && !disabled {
                let target = dir.join(format!("{}.disabled", fname));
                if fs::rename(&p, target).is_ok() {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn batch_delete(mc_dir_str: &str, filenames: &[String], content_type: &str) -> usize {
        let mc_dir = Path::new(mc_dir_str);
        let subfolder = match content_type {
            "resourcepacks" => "resourcepacks",
            "shaderpacks" => "shaderpacks",
            _ => "mods",
        };
        let dir = mc_dir.join(subfolder);
        let mut count = 0;

        for fname in filenames {
            let p = dir.join(fname);
            if Self::delete_file(p.to_str().unwrap_or("")) {
                count += 1;
            }
        }
        count
    }
}