use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemFileDto {
    pub filename: String,
    pub url: String,
    pub primary: bool,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemDependencyDto {
    pub project_id: String,
    pub version_id: Option<String>,
    pub dependency_type: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemVersionDto {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub date: String,
    pub changelog: String,
    pub files: Vec<StoreItemFileDto>,
    pub dependencies: Vec<StoreItemDependencyDto>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemGalleryDto {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemDetailsDto {
    pub body: String,
    pub gallery: Vec<StoreItemGalleryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreItemRecordDto {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: i64,
    pub follows: i64,
    pub categories: Vec<String>,
    pub provider: String,
    pub project_type: String,
    pub is_installed: bool,
    pub installed_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreSearchResultDto {
    pub success: bool,
    pub hits: Vec<StoreItemRecordDto>,
    pub total_hits: usize,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDetailsResponseDto {
    pub success: bool,
    pub details: StoreItemDetailsDto,
    pub versions: Vec<StoreItemVersionDto>,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDownloadProgressDto {
    pub project_id: String,
    pub filename: String,
    pub progress: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInstallResultDto {
    pub success: bool,
    pub filename: String,
    pub installed_dependencies: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInstallRequestDto {
    pub provider: String,
    pub project_id: String,
    pub version_id: Option<String>,
    pub url: String,
    pub filename: String,
    pub project_type: String,
    pub loader: Option<String>,
    pub game_version: Option<String>,
}

#[tauri::command]
pub async fn search_store_catalog(
    state: State<'_, AppState>,
    provider: String,
    query: Option<String>,
    project_type: Option<String>,
    loader: Option<String>,
    game_version: Option<String>,
    category: Option<String>,
    sort_index: Option<String>,
    offset: Option<i32>,
) -> Result<StoreSearchResultDto, AppError> {
    let cfg = config::load_app_config();
    let q = query.unwrap_or_default();
    let pt = project_type.unwrap_or_else(|| "mod".to_string());
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();
    let c = category.unwrap_or_default();
    let si = sort_index.unwrap_or_else(|| "relevance".to_string());
    let off = offset.unwrap_or(0);

    let res = state
        .store
        .search(
            &provider,
            &q,
            &pt,
            &l,
            &gv,
            &c,
            &si,
            off,
            &cfg.current_instance,
        )
        .await
        .map_err(|e| AppError::Config(e))?;

    let hits = res
        .hits
        .into_iter()
        .map(|h| StoreItemRecordDto {
            project_id: h.project_id,
            slug: h.slug,
            title: h.title,
            author: h.author,
            description: h.description,
            icon_url: h.icon_url,
            downloads: h.downloads,
            follows: h.follows,
            categories: h.categories,
            provider: h.provider,
            project_type: h.project_type,
            is_installed: h.is_installed,
            installed_filename: h.installed_filename,
        })
        .collect();

    Ok(StoreSearchResultDto {
        success: res.success,
        hits,
        total_hits: res.total_hits,
        msg: res.msg,
    })
}

#[tauri::command]
pub async fn get_store_project_details(
    state: State<'_, AppState>,
    provider: String,
    project_id: String,
    loader: Option<String>,
    game_version: Option<String>,
) -> Result<StoreDetailsResponseDto, AppError> {
    let l = loader.unwrap_or_default();
    let gv = game_version.unwrap_or_default();

    let res = state
        .store
        .get_details(&provider, &project_id, &l, &gv)
        .await
        .map_err(|e| AppError::Config(e))?;

    let gallery = res
        .details
        .gallery
        .into_iter()
        .map(|g| StoreItemGalleryDto {
            url: g.url,
            title: g.title,
        })
        .collect();

    let versions = res
        .versions
        .into_iter()
        .map(|v| StoreItemVersionDto {
            id: v.id,
            version_number: v.version_number,
            name: v.name,
            date: v.date,
            changelog: v.changelog,
            files: v
                .files
                .into_iter()
                .map(|f| StoreItemFileDto {
                    filename: f.filename,
                    url: f.url,
                    primary: f.primary,
                    size: f.size,
                })
                .collect(),
            dependencies: v
                .dependencies
                .into_iter()
                .map(|d| StoreItemDependencyDto {
                    project_id: d.project_id,
                    version_id: d.version_id,
                    dependency_type: d.dependency_type,
                    file_name: d.file_name,
                })
                .collect(),
            game_versions: v.game_versions,
            loaders: v.loaders,
        })
        .collect();

    Ok(StoreDetailsResponseDto {
        success: res.success,
        details: StoreItemDetailsDto {
            body: res.details.body,
            gallery,
        },
        versions,
        msg: res.msg,
    })
}

#[tauri::command]
pub async fn install_store_item_stream(
    state: State<'_, AppState>,
    _app: tauri::AppHandle,
    payload: StoreInstallRequestDto,
    progress_channel: Channel<StoreDownloadProgressDto>,
) -> Result<StoreInstallResultDto, AppError> {
    let cfg = config::load_app_config();
    let mc_dir = Path::new(&cfg.current_instance);
    let target_dir = match payload.project_type.as_str() {
        "resourcepack" => mc_dir.join("resourcepacks"),
        "shader" => mc_dir.join("shaderpacks"),
        "modpack" => mc_dir.join("downloads").join("modpacks"),
        _ => mc_dir.join("mods"),
    };

    fs::create_dir_all(&target_dir)?;

    let clean_fname = Path::new(&payload.filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&payload.filename);
    let primary_dest = target_dir.join(clean_fname);
    let temp_dest = target_dir.join(format!("{}.dl_tmp", clean_fname));

    let _ = progress_channel.send(StoreDownloadProgressDto {
        project_id: payload.project_id.clone(),
        filename: clean_fname.to_string(),
        progress: 10.0,
        status: "Initiating HTTP payload stream...".to_string(),
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("KIPStudio/KIP_Hub/{} (contact@kip.studio)", config::APP_VERSION))
        .build()?;

    let resp = client.get(&payload.url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Launch(format!("Download failed with status code {}", resp.status())));
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = File::create(&temp_dest)?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        let progress = if total_size > 0 {
            10.0 + ((downloaded as f64 / total_size as f64) * 80.0)
        } else {
            50.0
        };

        let _ = progress_channel.send(StoreDownloadProgressDto {
            project_id: payload.project_id.clone(),
            filename: clean_fname.to_string(),
            progress,
            status: "Downloading binary package...".to_string(),
        });
    }

    file.sync_all()?;
    drop(file);

    // Verify ZIP magic header: [0x50, 0x4B, 0x03, 0x04]
    let is_valid_zip = if let Ok(meta) = fs::metadata(&temp_dest) {
        if meta.len() >= 100 {
            if let Ok(mut check_file) = File::open(&temp_dest) {
                let mut magic = [0u8; 4];
                check_file.read_exact(&mut magic).is_ok() && &magic == &[0x50, 0x4B, 0x03, 0x04]
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if !is_valid_zip {
        let _ = fs::remove_file(&temp_dest);
        return Err(AppError::Config("Downloaded archive failed ZIP magic integrity validation.".to_string()));
    }

    if primary_dest.exists() {
        let _ = fs::remove_file(&primary_dest);
    }
    fs::rename(&temp_dest, &primary_dest)?;

    let mut installed_dependencies = Vec::new();

    // Transitive dependencies auto-resolver for Modrinth mods
    if payload.provider == "modrinth" && payload.project_type == "mod" {
        let ver_url = if let Some(ref vid) = payload.version_id {
            if !vid.is_empty() {
                format!("https://api.modrinth.com/v2/version/{}", vid)
            } else {
                format!("https://api.modrinth.com/v2/project/{}/version", payload.project_id)
            }
        } else {
            format!("https://api.modrinth.com/v2/project/{}/version", payload.project_id)
        };

        if let Ok(v_resp) = client.get(&ver_url).send().await {
            if let Ok(v_data) = v_resp.json::<Value>().await {
                let target_ver_json = if v_data.is_array() {
                    v_data.as_array().and_then(|a| a.first()).cloned().unwrap_or(Value::Null)
                } else {
                    v_data
                };

                if let Some(deps) = target_ver_json["dependencies"].as_array() {
                    let mut visited = HashSet::new();
                    visited.insert(payload.project_id.clone());

                    for dep in deps {
                        if dep["dependency_type"].as_str() == Some("required") {
                            if let Some(dep_proj_id) = dep["project_id"].as_str() {
                                if !visited.contains(dep_proj_id) {
                                    visited.insert(dep_proj_id.to_string());
                                    let ldr = payload.loader.as_deref().unwrap_or("fabric");
                                    let gver = payload.game_version.as_deref().unwrap_or("26.3");

                                    if let Ok(dep_details) = state.store.get_details("modrinth", dep_proj_id, ldr, gver).await {
                                        if let Some(first_ver) = dep_details.versions.first() {
                                            if let Some(target_f) = first_ver.files.iter().find(|f| f.primary).or_else(|| first_ver.files.first()) {
                                                let dep_fname = Path::new(&target_f.filename)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or(&target_f.filename);
                                                let dep_dest = target_dir.join(dep_fname);

                                                if !dep_dest.exists() {
                                                    let _ = progress_channel.send(StoreDownloadProgressDto {
                                                        project_id: dep_proj_id.to_string(),
                                                        filename: dep_fname.to_string(),
                                                        progress: 95.0,
                                                        status: format!("Acquiring companion dependency {}...", dep_fname),
                                                    });

                                                    if let Ok(dep_dl_resp) = client.get(&target_f.url).send().await {
                                                        if let Ok(bytes) = dep_dl_resp.bytes().await {
                                                            if bytes.len() >= 100 && &bytes[0..4] == &[0x50, 0x4B, 0x03, 0x04] {
                                                                if fs::write(&dep_dest, bytes).is_ok() {
                                                                    installed_dependencies.push(dep_fname.to_string());
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
                        }
                    }
                }
            }
        }
    }

    let _ = progress_channel.send(StoreDownloadProgressDto {
        project_id: payload.project_id.clone(),
        filename: clean_fname.to_string(),
        progress: 100.0,
        status: "Deployment successfully finalized.".to_string(),
    });

    Ok(StoreInstallResultDto {
        success: true,
        filename: clean_fname.to_string(),
        installed_dependencies,
        message: format!("Successfully deployed {}", clean_fname),
    })
}

#[tauri::command]
pub fn uninstall_store_item(
    state: State<'_, AppState>,
    project_type: String,
    filename: String,
) -> Result<bool, AppError> {
    let cfg = config::load_app_config();
    state
        .store
        .uninstall_item(&cfg.current_instance, &project_type, &filename)
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
}