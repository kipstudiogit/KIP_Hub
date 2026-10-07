use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use tauri::ipc::Channel;
use tauri::State;
use zip::ZipArchive;

use crate::commands::AppState;
use crate::config;
use crate::doctor_manager::parser::JarParser;
use crate::error::AppError;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeColor {
    pub background: String,
    pub border: String,
    pub highlight: GraphNodeHighlightColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeHighlightColor {
    pub background: String,
    pub border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNodeFont {
    pub color: String,
    pub size: u32,
    pub face: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub title: String,
    pub shape: String,
    pub size: u32,
    pub color: GraphNodeColor,
    pub font: GraphNodeFont,
    pub version: String,
    pub mod_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdgeColor {
    pub color: String,
    pub highlight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub color: GraphEdgeColor,
    pub arrows: String,
    pub dashes: bool,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModGraphDataDto {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateItemDto {
    pub filename: String,
    pub project_id: String,
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub new_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdatesCheckDto {
    pub success: bool,
    pub updates: Vec<ModUpdateItemDto>,
    pub checked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUpdateProgressDto {
    pub current_file: String,
    pub index: usize,
    pub total: usize,
    pub percent: f64,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentActionResultDto {
    pub success: bool,
    pub msg: String,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackRecordDto {
    pub filename: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub mc_version: String,
    pub loader: String,
    pub mod_count: usize,
    pub icon: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportArchiveResultDto {
    pub success: bool,
    #[serde(rename = "mc_version", alias = "mcVersion")]
    pub mc_version: Option<String>,
    pub loader: Option<String>,
    pub message: String,
}

fn sanitize_file_name(raw_name: &str) -> Option<String> {
    let path = Path::new(raw_name);
    let file_name = path.file_name()?;
    let name_str = file_name.to_string_lossy().trim().to_string();
    if name_str.is_empty()
        || name_str.contains('/')
        || name_str.contains('\\')
        || name_str == ".."
        || name_str == "."
    {
        None
    } else {
        Some(name_str)
    }
}

fn compute_sha1(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 65536];
    while let Ok(count) = file.read(&mut buffer) {
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Some(hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect())
}

#[tauri::command]
pub async fn get_mod_graph_data(_state: State<'_, AppState>) -> Result<ModGraphDataDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut id_map = HashMap::new();
        let mut provides_map = HashMap::new();
        let mut parsed_mods = Vec::new();

        let parser = JarParser::new();

        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let descriptor = parser.parse(&path);
                    let mod_id = if descriptor.id != "?" {
                        descriptor.id.clone()
                    } else {
                        descriptor.name.to_lowercase().replace(' ', "-")
                    };

                    id_map.insert(mod_id.clone(), descriptor.name.clone());
                    for provided in &descriptor.provides {
                        provides_map.insert(provided.clone(), mod_id.clone());
                    }
                    parsed_mods.push((mod_id, descriptor));
                }
            }
        }

        for (mod_id, descriptor) in &parsed_mods {
            let is_optimization = mod_id.contains("sodium")
                || mod_id.contains("lithium")
                || mod_id.contains("ferrite")
                || mod_id.contains("culling")
                || mod_id.contains("krypton")
                || mod_id.contains("iris");

            let is_core_lib = mod_id.contains("fabric-api")
                || mod_id.contains("cloth")
                || mod_id.contains("architectury")
                || mod_id.contains("forgeconfig")
                || mod_id.contains("kotlin");

            let (bg, border, highlight_bg, highlight_border) = if is_optimization {
                ("#10B981".to_string(), "#34D399".to_string(), "#059669".to_string(), "#6EE7B7".to_string())
            } else if is_core_lib {
                ("#A855F7".to_string(), "#C084FC".to_string(), "#9333EA".to_string(), "#E9D5FF".to_string())
            } else {
                ("#6366F1".to_string(), "#818CF8".to_string(), "#4F46E5".to_string(), "#C7D2FE".to_string())
            };

            nodes.push(GraphNode {
                id: mod_id.clone(),
                label: descriptor.name.clone(),
                title: format!(
                    "<b>{}</b><br>ID: {}<br>Version: {}<br>File: {}",
                    descriptor.name, mod_id, descriptor.version, descriptor.filename
                ),
                shape: if is_core_lib { "hexagon".to_string() } else { "dot".to_string() },
                size: if is_core_lib { 22 } else if is_optimization { 18 } else { 15 },
                color: GraphNodeColor {
                    background: bg,
                    border,
                    highlight: GraphNodeHighlightColor {
                        background: highlight_bg,
                        border: highlight_border,
                    },
                },
                font: GraphNodeFont {
                    color: "#FFFFFF".to_string(),
                    size: 11,
                    face: "ui-monospace, monospace".to_string(),
                },
                version: descriptor.version.clone(),
                mod_type: if is_optimization {
                    "Optimization".to_string()
                } else if is_core_lib {
                    "Library".to_string()
                } else {
                    "Mod".to_string()
                },
            });

            for dep_id in descriptor.depends.keys() {
                if dep_id == "minecraft"
                    || dep_id == "java"
                    || dep_id == "fabricloader"
                    || dep_id == "quilt_loader"
                    || dep_id == "forge"
                    || dep_id == "neoforge"
                {
                    continue;
                }

                let target_node_id = if id_map.contains_key(dep_id) {
                    Some(dep_id.clone())
                } else {
                    provides_map.get(dep_id).cloned()
                };

                if let Some(target) = target_node_id {
                    edges.push(GraphEdge {
                        from: mod_id.clone(),
                        to: target,
                        color: GraphEdgeColor {
                            color: "rgba(99, 102, 241, 0.4)".to_string(),
                            highlight: "#818CF8".to_string(),
                        },
                        arrows: "to".to_string(),
                        dashes: false,
                        label: None,
                    });
                } else {
                    let missing_node_id = format!("missing_{}", dep_id);
                    if !nodes.iter().any(|n| n.id == missing_node_id) {
                        nodes.push(GraphNode {
                            id: missing_node_id.clone(),
                            label: format!("{} (?)", dep_id),
                            title: format!("<b>Missing Dependency</b><br>ID: {}", dep_id),
                            shape: "diamond".to_string(),
                            size: 14,
                            color: GraphNodeColor {
                                background: "#F59E0B".to_string(),
                                border: "#FBBF24".to_string(),
                                highlight: GraphNodeHighlightColor {
                                    background: "#D97706".to_string(),
                                    border: "#FDE68A".to_string(),
                                },
                            },
                            font: GraphNodeFont {
                                color: "#FDE68A".to_string(),
                                size: 10,
                                face: "ui-monospace, monospace".to_string(),
                            },
                            version: "Missing".to_string(),
                            mod_type: "Dependency".to_string(),
                        });
                    }

                    edges.push(GraphEdge {
                        from: mod_id.clone(),
                        to: missing_node_id,
                        color: GraphEdgeColor {
                            color: "rgba(245, 158, 11, 0.6)".to_string(),
                            highlight: "#F59E0B".to_string(),
                        },
                        arrows: "to".to_string(),
                        dashes: true,
                        label: Some("requires".to_string()),
                    });
                }
            }

            for break_id in descriptor.breaks.keys() {
                let target_node_id = if id_map.contains_key(break_id) {
                    Some(break_id.clone())
                } else {
                    provides_map.get(break_id).cloned()
                };

                if let Some(target) = target_node_id {
                    edges.push(GraphEdge {
                        from: mod_id.clone(),
                        to: target,
                        color: GraphEdgeColor {
                            color: "rgba(239, 68, 68, 0.8)".to_string(),
                            highlight: "#EF4444".to_string(),
                        },
                        arrows: "to".to_string(),
                        dashes: true,
                        label: Some("collision".to_string()),
                    });
                }
            }
        }

        Ok(ModGraphDataDto { nodes, edges })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn import_dropped_content(files: Vec<String>) -> Result<ContentActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = PathBuf::from(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let modpacks_dir = mc_dir.join("modpacks");
        let resourcepacks_dir = mc_dir.join("resourcepacks");
        let shaderpacks_dir = mc_dir.join("shaderpacks");

        fs::create_dir_all(&mods_dir)?;
        fs::create_dir_all(&modpacks_dir)?;
        fs::create_dir_all(&resourcepacks_dir)?;
        fs::create_dir_all(&shaderpacks_dir)?;

        let mut imported = 0;
        for file_path in files {
            let src = Path::new(&file_path);
            if !src.exists() || !src.is_file() {
                continue;
            }

            let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let file_name_os = match src.file_name() {
                Some(n) => n,
                None => continue,
            };

            let safe_name = match sanitize_file_name(&file_name_os.to_string_lossy()) {
                Some(n) => n,
                None => continue,
            };

            if ext == "jar" {
                let dest = mods_dir.join(&safe_name);
                if fs::copy(src, dest).is_ok() {
                    imported += 1;
                }
            } else if ext == "mrpack" {
                let dest = modpacks_dir.join(&safe_name);
                if fs::copy(src, dest).is_ok() {
                    imported += 1;
                }
            } else if ext == "zip" {
                let is_shader = safe_name.to_lowercase().contains("shader")
                    || safe_name.to_lowercase().contains("complementary");
                let is_modpack = safe_name.to_lowercase().contains("modpack")
                    || safe_name.to_lowercase().contains("pack");

                let dest = if is_shader {
                    shaderpacks_dir.join(&safe_name)
                } else if is_modpack {
                    modpacks_dir.join(&safe_name)
                } else {
                    resourcepacks_dir.join(&safe_name)
                };

                if fs::copy(src, dest).is_ok() {
                    imported += 1;
                }
            }
        }

        Ok(ContentActionResultDto {
            success: true,
            msg: format!("Successfully imported {} packages into active environment.", imported),
            count: Some(imported),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn import_modpack_or_archive(file_path: String) -> Result<ImportArchiveResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let src = Path::new(&file_path);
        if !src.exists() || !src.is_file() {
            return Err(AppError::Config("Specified package file does not exist on disk.".to_string()));
        }

        let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let file_name = src.file_name().unwrap_or_default().to_string_lossy().to_string();
        let safe_name = match sanitize_file_name(&file_name) {
            Some(n) => n,
            None => return Err(AppError::Config("Invalid package filename identifier.".to_string())),
        };

        let cfg = config::load_app_config();
        let mc_dir = PathBuf::from(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let modpacks_dir = mc_dir.join("modpacks");
        fs::create_dir_all(&mods_dir)?;
        fs::create_dir_all(&modpacks_dir)?;

        if ext == "jar" {
            let dest = mods_dir.join(&safe_name);
            fs::copy(src, dest)?;
            return Ok(ImportArchiveResultDto {
                success: true,
                mc_version: None,
                loader: None,
                message: format!("Standalone modification '{}' integrated into instance.", safe_name),
            });
        }

        if ext == "mrpack" || ext == "zip" {
            let dest_pack = modpacks_dir.join(&safe_name);
            let _ = fs::copy(src, &dest_pack);

            let file = File::open(src)?;
            let mut archive = ZipArchive::new(file)
                .map_err(|e| AppError::Launch(format!("Archive decompression failed: {}", e)))?;

            let mut mc_version = None;
            let mut loader = None;
            let mut extracted_count = 0;

            if let Ok(mut idx_file) = archive.by_name("modrinth.index.json") {
                let mut s = String::new();
                if idx_file.read_to_string(&mut s).is_ok() {
                    if let Ok(val) = serde_json::from_str::<Value>(&s) {
                        if let Some(deps) = val["dependencies"].as_object() {
                            if let Some(mv) = deps.get("minecraft").and_then(|v| v.as_str()) {
                                mc_version = Some(mv.to_string());
                            }
                            if deps.contains_key("fabric-loader") {
                                loader = Some("fabric".to_string());
                            } else if deps.contains_key("neoforge") {
                                loader = Some("neoforge".to_string());
                            } else if deps.contains_key("forge") {
                                loader = Some("forge".to_string());
                            } else if deps.contains_key("quilt-loader") {
                                loader = Some("quilt".to_string());
                            }
                        }
                    }
                }
            }

            if mc_version.is_none() {
                if let Ok(mut man_file) = archive.by_name("manifest.json") {
                    let mut s = String::new();
                    if man_file.read_to_string(&mut s).is_ok() {
                        if let Ok(val) = serde_json::from_str::<Value>(&s) {
                            if let Some(mv) = val["minecraft"]["version"].as_str() {
                                mc_version = Some(mv.to_string());
                            }
                            if let Some(loaders_arr) = val["minecraft"]["modLoaders"].as_array() {
                                if let Some(first) = loaders_arr.first().and_then(|l| l["id"].as_str()) {
                                    loader = Some(first.split('-').next().unwrap_or(first).to_lowercase());
                                }
                            }
                        }
                    }
                }
            }

            for i in 0..archive.len() {
                if let Ok(mut zf) = archive.by_index(i) {
                    let name = zf.name().to_string();
                    if name.ends_with('/') {
                        continue;
                    }

                    let target_rel = if let Some(stripped) = name.strip_prefix("overrides/") {
                        Some(PathBuf::from(stripped))
                    } else if let Some(stripped) = name.strip_prefix("client-overrides/") {
                        Some(PathBuf::from(stripped))
                    } else if name.ends_with(".jar") && !name.contains('/') && !name.contains('\\') {
                        Some(Path::new("mods").join(&name))
                    } else {
                        None
                    };

                    if let Some(rel) = target_rel {
                        let out_path = mc_dir.join(&rel);
                        if let Some(p) = out_path.parent() {
                            let _ = fs::create_dir_all(p);
                        }
                        if let Ok(mut outfile) = File::create(&out_path) {
                            if std::io::copy(&mut zf, &mut outfile).is_ok() {
                                extracted_count += 1;
                            }
                        }
                    }
                }
            }

            return Ok(ImportArchiveResultDto {
                success: true,
                mc_version,
                loader,
                message: format!("Mounted {} files and overrides from archive '{}'.", extracted_count, safe_name),
            });
        }

        Err(AppError::Config(format!("Unsupported package container format: .{}", ext)))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn check_content_updates(
    mc_version: Option<String>,
    loader: Option<String>,
) -> Result<ModUpdatesCheckDto, AppError> {
    let target_mc = mc_version.unwrap_or_else(|| "26.3".to_string());
    let target_loader = loader.unwrap_or_else(|| "fabric".to_string());

    let (file_hashes, hash_to_file) = tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mods_dir = Path::new(&cfg.current_instance).join("mods");
        let mut hashes = Vec::new();
        let mut map = HashMap::new();

        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("jar") {
                    let filename = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if !filename.ends_with(".disabled") {
                        if let Some(hash) = compute_sha1(&p) {
                            hashes.push(hash.clone());
                            map.insert(hash, filename);
                        }
                    }
                }
            }
        }
        (hashes, map)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?;

    if file_hashes.is_empty() {
        return Ok(ModUpdatesCheckDto {
            success: true,
            updates: Vec::new(),
            checked_count: 0,
        });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(format!("KIPStudio/KIP_Hub/{} (contact@kip.studio)", config::APP_VERSION))
        .build()?;

    let payload = json!({
        "hashes": file_hashes,
        "algorithm": "sha1",
        "loaders": [target_loader],
        "game_versions": [target_mc]
    });

    let mut updates = Vec::new();
    let resp = client
        .post("https://api.modrinth.com/v2/version_files/update")
        .json(&payload)
        .send()
        .await?;

    if resp.status().is_success() {
        let updates_map: HashMap<String, Value> = resp.json().await?;
        for (old_hash, new_ver) in updates_map {
            if let Some(old_filename) = hash_to_file.get(&old_hash) {
                if let Some(files) = new_ver["files"].as_array() {
                    let candidate = files.iter().find(|f| {
                        let fname = f["filename"].as_str().unwrap_or("").to_lowercase();
                        !fname.ends_with("-sources.jar")
                            && !fname.ends_with("-javadoc.jar")
                            && !fname.ends_with("-dev.jar")
                            && f["primary"].as_bool().unwrap_or(false)
                    }).or_else(|| {
                        files.iter().find(|f| {
                            let fname = f["filename"].as_str().unwrap_or("").to_lowercase();
                            !fname.ends_with("-sources.jar")
                                && !fname.ends_with("-javadoc.jar")
                                && !fname.ends_with("-dev.jar")
                                && fname.ends_with(".jar")
                        })
                    });

                    if let Some(file) = candidate {
                        let new_hash = file["hashes"]["sha1"].as_str().unwrap_or("");
                        if new_hash != old_hash {
                            let new_filename = file["filename"].as_str().unwrap_or("").to_string();
                            let download_url = file["url"].as_str().unwrap_or("").to_string();
                            let project_id = new_ver["project_id"].as_str().unwrap_or("").to_string();
                            let new_version = new_ver["version_number"].as_str().unwrap_or("").to_string();
                            let mod_name = old_filename.split('-').next().unwrap_or(old_filename).to_string();

                            updates.push(ModUpdateItemDto {
                                filename: old_filename.clone(),
                                project_id,
                                name: mod_name,
                                current_version: "Active".to_string(),
                                latest_version: new_version,
                                download_url,
                                new_filename,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(ModUpdatesCheckDto {
        success: true,
        checked_count: file_hashes.len(),
        updates,
    })
}

#[tauri::command]
pub async fn apply_content_updates_stream(
    updates: Vec<ModUpdateItemDto>,
    progress_channel: Channel<ContentUpdateProgressDto>,
) -> Result<ContentActionResultDto, AppError> {
    let cfg = config::load_app_config();
    let mods_dir = PathBuf::from(&cfg.current_instance).join("mods");
    fs::create_dir_all(&mods_dir)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("KIPStudio/KIP_Hub/{} (contact@kip.studio)", config::APP_VERSION))
        .build()?;

    let total = updates.len();
    let mut updated_count = 0;

    for (index, u) in updates.into_iter().enumerate() {
        let percent = if total > 0 { ((index as f64) / (total as f64)) * 100.0 } else { 0.0 };
        let _ = progress_channel.send(ContentUpdateProgressDto {
            current_file: u.new_filename.clone(),
            index,
            total,
            percent,
            completed: false,
        });

        if u.download_url.is_empty() || u.new_filename.is_empty() {
            continue;
        }

        let safe_new_name = match sanitize_file_name(&u.new_filename) {
            Some(n) => n,
            None => continue,
        };

        let temp_path = mods_dir.join(format!("{}.dl_tmp", safe_new_name));
        let dest_path = mods_dir.join(&safe_new_name);

        if let Ok(resp) = client.get(&u.download_url).send().await {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {
                    if bytes.len() >= 1000 && &bytes[0..4] == &[0x50, 0x4B, 0x03, 0x04] {
                        if let Ok(mut file) = File::create(&temp_path) {
                            if file.write_all(&bytes).is_ok() && file.sync_all().is_ok() {
                                drop(file);
                                if fs::rename(&temp_path, &dest_path).is_ok() {
                                    if !u.filename.is_empty() && u.filename != safe_new_name {
                                        if let Some(safe_old) = sanitize_file_name(&u.filename) {
                                            let old_path = mods_dir.join(safe_old);
                                            if old_path.exists() {
                                                let _ = fs::remove_file(old_path);
                                            }
                                        }
                                    }
                                    updated_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = fs::remove_file(&temp_path);
    }

    let _ = progress_channel.send(ContentUpdateProgressDto {
        current_file: String::new(),
        index: total,
        total,
        percent: 100.0,
        completed: true,
    });

    Ok(ContentActionResultDto {
        success: true,
        msg: format!("Successfully upgraded {} modifications.", updated_count),
        count: Some(updated_count),
    })
}

#[tauri::command]
pub async fn get_installed_modpacks() -> Result<Vec<ModpackRecordDto>, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mc_dir = PathBuf::from(&cfg.current_instance);
        let modpacks_dir = mc_dir.join("modpacks");
        fs::create_dir_all(&modpacks_dir)?;

        let mut modpacks = Vec::new();
        if let Ok(entries) = fs::read_dir(&modpacks_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    let filename = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let size_bytes = p.metadata().map(|m| m.len()).unwrap_or(0);

                    if ext == "mrpack" || ext == "zip" {
                        let mut name = filename.replace(".mrpack", "").replace(".zip", "");
                        let mut version = "1.0.0".to_string();
                        let mut author = "Community".to_string();
                        let mut description = "Minecraft Modpack Archive Container.".to_string();
                        let mut mc_version = "26.3".to_string();
                        let mut loader = "fabric".to_string();
                        let mut mod_count = 0;

                        if let Ok(file) = File::open(&p) {
                            if let Ok(mut archive) = ZipArchive::new(file) {
                                for i in 0..archive.len() {
                                    if let Ok(zf) = archive.by_index(i) {
                                        let zname = zf.name();
                                        if zname.ends_with(".jar") || zname.starts_with("overrides/mods/") {
                                            mod_count += 1;
                                        }
                                    }
                                }

                                let mut modrinth_index_str = None;
                                if let Ok(mut index_file) = archive.by_name("modrinth.index.json") {
                                    let mut s = String::new();
                                    if index_file.read_to_string(&mut s).is_ok() {
                                        modrinth_index_str = Some(s);
                                    }
                                }

                                let mut curseforge_manifest_str = None;
                                if modrinth_index_str.is_none() {
                                    if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
                                        let mut s = String::new();
                                        if manifest_file.read_to_string(&mut s).is_ok() {
                                            curseforge_manifest_str = Some(s);
                                        }
                                    }
                                }

                                if let Some(index_str) = modrinth_index_str {
                                    if let Ok(idx_val) = serde_json::from_str::<Value>(&index_str) {
                                        if let Some(n) = idx_val["name"].as_str() { name = n.to_string(); }
                                        if let Some(v) = idx_val["versionId"].as_str() { version = v.to_string(); }
                                        if let Some(s) = idx_val["summary"].as_str() { description = s.to_string(); }
                                        if let Some(deps) = idx_val["dependencies"].as_object() {
                                            if let Some(mv) = deps.get("minecraft").and_then(|v| v.as_str()) { mc_version = mv.to_string(); }
                                            if deps.contains_key("fabric-loader") { loader = "fabric".to_string(); }
                                            else if deps.contains_key("neoforge") { loader = "neoforge".to_string(); }
                                            else if deps.contains_key("forge") { loader = "forge".to_string(); }
                                            else if deps.contains_key("quilt-loader") { loader = "quilt".to_string(); }
                                        }
                                        if let Some(files) = idx_val["files"].as_array() {
                                            mod_count = files.len();
                                        }
                                    }
                                } else if let Some(manifest_str) = curseforge_manifest_str {
                                    if let Ok(man_val) = serde_json::from_str::<Value>(&manifest_str) {
                                        if let Some(n) = man_val["name"].as_str() { name = n.to_string(); }
                                        if let Some(v) = man_val["version"].as_str() { version = v.to_string(); }
                                        if let Some(a) = man_val["author"].as_str() { author = a.to_string(); }
                                        if let Some(mv) = man_val["minecraft"]["version"].as_str() { mc_version = mv.to_string(); }
                                        if let Some(loaders_arr) = man_val["minecraft"]["modLoaders"].as_array() {
                                            if let Some(first) = loaders_arr.first().and_then(|l| l["id"].as_str()) {
                                                loader = first.to_lowercase();
                                            }
                                        }
                                        if let Some(files) = man_val["files"].as_array() {
                                            mod_count = files.len();
                                        }
                                    }
                                }
                            }
                        }

                        modpacks.push(ModpackRecordDto {
                            filename,
                            name,
                            version,
                            author,
                            description,
                            mc_version,
                            loader,
                            mod_count,
                            icon: String::new(),
                            size_bytes,
                        });
                    }
                }
            }
        }

        Ok(modpacks)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn mount_modpack_archive(filename: String) -> Result<ContentActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_name = match sanitize_file_name(&filename) {
            Some(n) => n,
            None => return Err(AppError::Config("Invalid modpack filename identifier.".to_string())),
        };

        let cfg = config::load_app_config();
        let mc_dir = PathBuf::from(&cfg.current_instance);
        let modpacks_dir = mc_dir.join("modpacks");
        let pack_path = modpacks_dir.join(&safe_name);

        if !pack_path.exists() || !pack_path.is_file() {
            return Err(AppError::Config("Specified modpack archive was not found on disk.".to_string()));
        }

        let zip_file = File::open(&pack_path)?;
        let mut archive = ZipArchive::new(zip_file).map_err(|e| AppError::Launch(e.to_string()))?;
        let mut extracted_count = 0;

        for i in 0..archive.len() {
            if let Ok(mut zf) = archive.by_index(i) {
                let name = zf.name().to_string();
                if name.ends_with('/') {
                    continue;
                }

                let target_relative = if let Some(stripped) = name.strip_prefix("overrides/") {
                    Some(PathBuf::from(stripped))
                } else if let Some(stripped) = name.strip_prefix("client-overrides/") {
                    Some(PathBuf::from(stripped))
                } else if name.ends_with(".jar") && !name.contains('/') && !name.contains('\\') {
                    Some(Path::new("mods").join(&name))
                } else {
                    None
                };

                if let Some(rel) = target_relative {
                    let dest = mc_dir.join(&rel);
                    if let Some(parent) = dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(mut outfile) = File::create(&dest) {
                        if std::io::copy(&mut zf, &mut outfile).is_ok() {
                            extracted_count += 1;
                        }
                    }
                }
            }
        }

        Ok(ContentActionResultDto {
            success: true,
            msg: format!("Mounted {} files and overrides from archive '{}' into instance.", extracted_count, safe_name),
            count: Some(extracted_count),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn export_active_modpack() -> Result<ContentActionResultDto, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let export_dir = config::get_app_data_dir().join("exports");
        fs::create_dir_all(&export_dir)?;

        let zip_name = format!("modpack_export_{}.zip", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let zip_dest = export_dir.join(&zip_name);

        let size_mb = ToolManager::create_backup(
            mc_dir.join("mods").to_str().unwrap_or(""),
            export_dir.to_str().unwrap_or(""),
        );

        Ok(ContentActionResultDto {
            success: true,
            msg: format!("Exported modpack successfully ({:.1} MB) to {:?}", size_mb, zip_dest),
            count: None,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}