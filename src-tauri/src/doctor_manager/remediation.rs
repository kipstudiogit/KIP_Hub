use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};

use crate::api_manager::ApiManager;

pub struct RemediationEngine;

impl RemediationEngine {
    fn sanitize_target_path(base_dir: &Path, file_path_str: &str) -> Option<PathBuf> {
        let cleaned = file_path_str.trim();
        if cleaned.is_empty() {
            return None;
        }

        let target_relative = if let Some(stripped) = cleaned.strip_prefix("../config/") {
            Path::new("config").join(stripped)
        } else if let Some(stripped) = cleaned.strip_prefix("../") {
            PathBuf::from(stripped)
        } else {
            Path::new("mods").join(cleaned)
        };

        let mut verified_path = PathBuf::new();
        for component in target_relative.components() {
            match component {
                std::path::Component::Normal(segment) => verified_path.push(segment),
                _ => return None,
            }
        }

        let instance_root = base_dir.parent()?;
        let full_path = instance_root.join(verified_path);
        Some(full_path)
    }

    pub async fn apply(
        mods_dir: &str,
        _api_manager: &ApiManager,
        issues: &[Value],
        mc_version: &str,
        loader: &str,
    ) -> Value {
        let m_dir = Path::new(mods_dir);
        let mut deleted = 0;
        let mut downloaded = 0;
        let mut upgraded = 0;
        let mut disabled = 0;
        let mut errors = Vec::new();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("KIPStudio/KIP_Hub/1.7.0 (contact@kip.studio)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        for issue in issues {
            let action = issue["action"].as_str().unwrap_or("");
            let target_file = issue["target_file"].as_str().unwrap_or("");
            let target_slug = issue["target_slug"].as_str().unwrap_or("");

            if action == "DELETE" && !target_file.is_empty() {
                if let Some(target_path) = Self::sanitize_target_path(m_dir, target_file) {
                    if target_path.exists() && target_path.is_file() {
                        match fs::remove_file(&target_path) {
                            Ok(_) => deleted += 1,
                            Err(e) => errors.push(format!("Failed to delete '{}': {}", target_file, e)),
                        }
                    }
                }
            } else if action == "DISABLE" && !target_file.is_empty() {
                if !target_file.ends_with(".disabled") {
                    if let Some(target_path) = Self::sanitize_target_path(m_dir, target_file) {
                        if target_path.exists() && target_path.is_file() {
                            let new_name = format!("{}.disabled", target_path.to_string_lossy());
                            match fs::rename(&target_path, &new_name) {
                                Ok(_) => disabled += 1,
                                Err(e) => errors.push(format!("Failed to disable '{}': {}", target_file, e)),
                            }
                        }
                    }
                }
            } else if (action == "DOWNLOAD" || action == "UPGRADE") && !target_slug.is_empty() {
                let search_url = format!(
                    "https://api.modrinth.com/v2/project/{}/version?loaders=[%22{}%22]&game_versions=[%22{}%22]",
                    target_slug, loader, mc_version
                );

                let mut target_file_url = None;
                let mut target_filename = None;

                if let Ok(res) = client.get(&search_url).send().await {
                    if res.status().is_success() {
                        if let Ok(versions) = res.json::<Value>().await {
                            if let Some(v_arr) = versions.as_array() {
                                for ver in v_arr {
                                    if let Some(files) = ver["files"].as_array() {
                                        let valid_file = files.iter().find(|f| {
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

                                        if let Some(f) = valid_file {
                                            target_file_url = f["url"].as_str().map(|s| s.to_string());
                                            target_filename = f["filename"].as_str().map(|s| s.to_string());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if target_file_url.is_none() {
                    let fallback_search = format!("https://api.modrinth.com/v2/project/{}/version", target_slug);
                    if let Ok(res) = client.get(&fallback_search).send().await {
                        if res.status().is_success() {
                            if let Ok(versions) = res.json::<Value>().await {
                                if let Some(v_arr) = versions.as_array() {
                                    for v in v_arr {
                                        let matches_loader = loader.is_empty() || v["loaders"].as_array().map_or(false, |l| l.iter().any(|val| val == loader));
                                        let matches_mc = mc_version.is_empty() || v["game_versions"].as_array().map_or(false, |g| g.iter().any(|val| val == mc_version));

                                        if matches_loader && matches_mc {
                                            if let Some(files) = v["files"].as_array() {
                                                let valid_file = files.iter().find(|f| {
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

                                                if let Some(f) = valid_file {
                                                    target_file_url = f["url"].as_str().map(|s| s.to_string());
                                                    target_filename = f["filename"].as_str().map(|s| s.to_string());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let mut resolved = false;

                if let (Some(dl_url), Some(fname)) = (target_file_url, target_filename) {
                    let clean_fname = Path::new(&fname).file_name().and_then(|n| n.to_str()).unwrap_or(&fname);
                    let dest = m_dir.join(clean_fname);
                    let temp_dest = m_dir.join(format!("{}.dl_tmp", clean_fname));

                    if let Ok(dl_res) = client.get(&dl_url).send().await {
                        if dl_res.status().is_success() {
                            if let Ok(bytes) = dl_res.bytes().await {
                                if bytes.len() > 1000 && &bytes[0..4] == &[0x50, 0x4B, 0x03, 0x04] {
                                    if let Ok(mut out) = File::create(&temp_dest) {
                                        if out.write_all(&bytes).is_ok() && out.sync_all().is_ok() {
                                            drop(out);
                                            if fs::rename(&temp_dest, &dest).is_ok() {
                                                if action == "UPGRADE" {
                                                    upgraded += 1;
                                                    if !target_file.is_empty() && target_file != clean_fname {
                                                        if let Some(old_p) = Self::sanitize_target_path(m_dir, target_file) {
                                                            if old_p.exists() && old_p != dest {
                                                                let _ = fs::remove_file(old_p);
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    downloaded += 1;
                                                }
                                                resolved = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let _ = fs::remove_file(&temp_dest);
                }

                if !resolved {
                    errors.push(format!("Could not acquire verified binary for '{}' matching Minecraft {} ({}).", target_slug, mc_version, loader));
                }
            }
        }

        json!({
            "success": errors.is_empty(),
            "deleted": deleted,
            "downloaded": downloaded,
            "upgraded": upgraded,
            "disabled": disabled,
            "errors": errors
        })
    }
}