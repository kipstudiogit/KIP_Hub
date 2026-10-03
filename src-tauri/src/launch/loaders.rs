use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::java_manager::JavaManager;
use crate::launch::libraries::LibraryManager;

pub struct LoaderManager;

impl LoaderManager {
    fn extract_semver_weight(ver: &str) -> (u32, u32, u32, u32) {
        let clean = ver.trim_start_matches('v');
        let main_part = clean.split('+').next().unwrap_or(clean);
        let mut subparts = main_part.split('-');
        let num_str = subparts.next().unwrap_or("0.0.0");
        let tag = subparts.next().unwrap_or("");

        let nums: Vec<u32> = num_str.split('.').filter_map(|s| s.parse::<u32>().ok()).collect();
        let maj = nums.get(0).copied().unwrap_or(0);
        let min = nums.get(1).copied().unwrap_or(0);
        let pat = nums.get(2).copied().unwrap_or(0);

        let pre = if tag.is_empty() {
            999999
        } else if let Some(stripped) = tag.strip_prefix("beta.") {
            stripped.parse::<u32>().unwrap_or(100) + 1000
        } else if tag.starts_with("beta") {
            1000
        } else if tag.starts_with("alpha") {
            100
        } else {
            500
        };

        (maj, min, pat, pre)
    }

    pub async fn ensure_fabric(
        client: &reqwest::Client,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let versions_dir = mc_dir.join("versions");
        let actual_loader_ver = if loader_version.is_empty() {
            let meta_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
            let resp = client.get(&meta_url).send().await.map_err(|e| e.to_string())?;
            let arr: Value = resp.json().await.map_err(|e| e.to_string())?;
            let mut candidates: Vec<String> = Vec::new();
            if let Some(list) = arr.as_array() {
                for item in list {
                    if let Some(v_str) = item["loader"]["version"].as_str() {
                        candidates.push(v_str.to_string());
                    }
                }
            }

            candidates.sort_by(|a, b| {
                Self::extract_semver_weight(b).cmp(&Self::extract_semver_weight(a))
            });

            candidates.first().cloned().ok_or_else(|| "Failed to resolve Fabric loader version".to_string())?
        } else {
            loader_version.to_string()
        };

        let profile_name = format!("fabric-loader-{}-{}", actual_loader_ver, mc_version);
        let profile_dir = versions_dir.join(&profile_name);
        let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

        if !profile_json_path.exists() || profile_json_path.metadata().map(|m| m.len() < 10).unwrap_or(true) {
            let _ = app.emit("updateLaunchStatus", format!("Installing Fabric {}...", actual_loader_ver));
            let profile_url = format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                mc_version, actual_loader_ver
            );
            let resp = client.get(&profile_url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let profile_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
                fs::write(&profile_json_path, &profile_bytes).map_err(|e| e.to_string())?;
            }
        }

        if let Ok(content) = fs::read_to_string(&profile_json_path) {
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                LibraryManager::download_libraries(client, &data, mc_dir).await;
            }
        }

        Ok(profile_name)
    }

    pub async fn ensure_quilt(
        client: &reqwest::Client,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let versions_dir = mc_dir.join("versions");
        let actual_loader_ver = if loader_version.is_empty() {
            let meta_url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
            let mut candidates: Vec<String> = Vec::new();

            if let Ok(resp) = client.get(&meta_url).send().await {
                if let Ok(arr) = resp.json::<Value>().await {
                    if let Some(list) = arr.as_array() {
                        for item in list {
                            if let Some(v_str) = item["loader"]["version"].as_str() {
                                candidates.push(v_str.to_string());
                            }
                        }
                    }
                }
            }

            if candidates.is_empty() {
                let global_url = "https://meta.quiltmc.org/v3/versions/loader";
                let resp = client.get(global_url).send().await.map_err(|e| e.to_string())?;
                let arr: Value = resp.json().await.map_err(|e| e.to_string())?;
                if let Some(list) = arr.as_array() {
                    for item in list {
                        if let Some(v_str) = item["version"].as_str() {
                            candidates.push(v_str.to_string());
                        }
                    }
                }
            }

            candidates.sort_by(|a, b| {
                Self::extract_semver_weight(b).cmp(&Self::extract_semver_weight(a))
            });

            candidates.first().cloned().ok_or_else(|| "Failed to resolve Quilt loader version".to_string())?
        } else {
            loader_version.to_string()
        };

        let profile_name = format!("quilt-loader-{}-{}", actual_loader_ver, mc_version);
        let profile_dir = versions_dir.join(&profile_name);
        let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

        let should_install = if !profile_json_path.exists() || profile_json_path.metadata().map(|m| m.len() < 10).unwrap_or(true) {
            true
        } else {
            actual_loader_ver.starts_with("0.20.") || actual_loader_ver.starts_with("0.19.")
        };

        if should_install {
            let _ = app.emit("updateLaunchStatus", format!("Installing Quilt {}...", actual_loader_ver));
            let profile_url = format!(
                "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
                mc_version, actual_loader_ver
            );

            let resp = client.get(&profile_url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let profile_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
                fs::write(&profile_json_path, &profile_bytes).map_err(|e| e.to_string())?;
            }
        }

        if let Ok(content) = fs::read_to_string(&profile_json_path) {
            if let Ok(data) = serde_json::from_str::<Value>(&content) {
                LibraryManager::download_libraries(client, &data, mc_dir).await;
            }
        }

        Ok(profile_name)
    }

    pub async fn ensure_forge(
        client: &reqwest::Client,
        java: &Arc<JavaManager>,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let versions_dir = mc_dir.join("versions");
        let actual_forge_ver = if loader_version.is_empty() {
            let promo_url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
            let resp = client.get(promo_url).send().await.map_err(|e| e.to_string())?;
            let data: Value = resp.json().await.map_err(|e| e.to_string())?;

            data["promos"][format!("{}-recommended", mc_version)]
                .as_str()
                .or_else(|| data["promos"][format!("{}-latest", mc_version)].as_str())
                .ok_or_else(|| format!("No Forge version found for Minecraft {}", mc_version))?
                .to_string()
        } else {
            loader_version.to_string()
        };

        let profile_name = format!("{}-forge-{}", mc_version, actual_forge_ver);
        let profile_dir = versions_dir.join(&profile_name);
        let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

        let legacy_forge_dir = versions_dir.join(format!("{}-forge{}-{}", mc_version, mc_version, actual_forge_ver));
        let legacy_json_path = legacy_forge_dir.join(format!("{}-forge{}-{}.json", mc_version, mc_version, actual_forge_ver));

        let already_installed = (profile_json_path.exists() && profile_json_path.metadata().map(|m| m.len() > 10).unwrap_or(false))
            || (legacy_json_path.exists() && legacy_json_path.metadata().map(|m| m.len() > 10).unwrap_or(false));

        if !already_installed {
            let _ = app.emit("updateLaunchStatus", format!("Installing Forge {}...", actual_forge_ver));
            let installer_url = format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                mc_version, actual_forge_ver, mc_version, actual_forge_ver
            );

            let resp = client.get(&installer_url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                let installer_path = mc_dir.join("forge-installer-temp.jar");
                fs::write(&installer_path, bytes).map_err(|e| e.to_string())?;

                let req_major = java.get_required_java_major("forge", mc_version, None);
                let dummy_val = json!({ "javaVersion": { "majorVersion": req_major } });
                let java_bin = match java.resolve_compatible_java(mc_dir, "forge", mc_version, &dummy_val) {
                    Ok(b) => b,
                    Err(_) => java.ensure_java_runtime(mc_dir, req_major, app).await?,
                };

                let mut child = Command::new(&java_bin)
                    .arg("-jar")
                    .arg(&installer_path)
                    .arg("--installClient")
                    .arg(mc_dir.to_string_lossy().to_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn Forge installer: {}", e))?;

                let mut error_log = String::new();
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        error_log.push_str(&line);
                        error_log.push('\n');
                    }
                }

                let status = child.wait().map_err(|e| format!("Forge installer process error: {}", e))?;
                let _ = fs::remove_file(&installer_path);

                if !status.success() {
                    return Err(format!("Forge installer returned failure status code {:?}. Log: {}", status.code(), error_log));
                }
            }
        }

        if let Ok(entries) = fs::read_dir(&versions_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("forge") && name.contains(mc_version) {
                    return Ok(name);
                }
            }
        }

        Ok(profile_name)
    }

    pub async fn ensure_neoforge(
        client: &reqwest::Client,
        java: &Arc<JavaManager>,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let versions_dir = mc_dir.join("versions");

        let actual_neo_ver = if loader_version.is_empty() {
            let meta_url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
            let resp = client.get(meta_url).send().await.map_err(|e| e.to_string())?;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            let target_prefix = if mc_version == "1.20.1" {
                "47.1.".to_string()
            } else if mc_version == "1.20.2" {
                "20.2.".to_string()
            } else if mc_version == "1.20.3" {
                "20.3.".to_string()
            } else if mc_version == "1.20.4" {
                "20.4.".to_string()
            } else if mc_version == "1.20.5" || mc_version == "1.20.6" {
                "20.6.".to_string()
            } else if mc_version == "1.21" || mc_version == "1.21.0" {
                "21.0.".to_string()
            } else if mc_version == "1.21.1" {
                "21.1.".to_string()
            } else {
                let sub = mc_version.strip_prefix("1.").unwrap_or(mc_version);
                format!("{}.", sub)
            };

            let mut matched_versions = Vec::new();
            for line in body.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("<version>") && trimmed.ends_with("</version>") {
                    let v = trimmed.trim_start_matches("<version>").trim_end_matches("</version>");
                    if v.starts_with(&target_prefix) {
                        matched_versions.push(v.to_string());
                    }
                }
            }

            matched_versions.sort_by(|a, b| {
                Self::extract_semver_weight(b).cmp(&Self::extract_semver_weight(a))
            });

            matched_versions.first().cloned().ok_or_else(|| {
                format!("No compatible NeoForge version found for Minecraft {}", mc_version)
            })?
        } else {
            loader_version.to_string()
        };

        let profile_name = format!("neoforge-{}", actual_neo_ver);
        let profile_dir = versions_dir.join(&profile_name);
        let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

        if !profile_json_path.exists() || profile_json_path.metadata().map(|m| m.len() < 10).unwrap_or(true) {
            let _ = app.emit("updateLaunchStatus", format!("Installing NeoForge {}...", actual_neo_ver));
            let installer_url = format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
                actual_neo_ver, actual_neo_ver
            );

            let resp = client.get(&installer_url).send().await.map_err(|e| e.to_string())?;
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                let installer_path = mc_dir.join("neoforge-installer-temp.jar");
                fs::write(&installer_path, bytes).map_err(|e| e.to_string())?;

                let req_major = java.get_required_java_major("neoforge", mc_version, None);
                let dummy_val = json!({ "javaVersion": { "majorVersion": req_major } });
                let java_bin = match java.resolve_compatible_java(mc_dir, "neoforge", mc_version, &dummy_val) {
                    Ok(b) => b,
                    Err(_) => java.ensure_java_runtime(mc_dir, req_major, app).await?,
                };

                let mut child = Command::new(&java_bin)
                    .arg("-jar")
                    .arg(&installer_path)
                    .arg("--install-client")
                    .arg(mc_dir.to_string_lossy().to_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to spawn NeoForge installer: {}", e))?;

                let mut error_log = String::new();
                if let Some(stderr) = child.stderr.take() {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        error_log.push_str(&line);
                        error_log.push('\n');
                    }
                }

                let status = child.wait().map_err(|e| format!("NeoForge installer process error: {}", e))?;
                let _ = fs::remove_file(&installer_path);

                if !status.success() {
                    return Err(format!("NeoForge installer returned failure status code {:?}. Log: {}", status.code(), error_log));
                }
            }
        }

        if let Ok(entries) = fs::read_dir(&versions_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("neoforge") && (name.contains(&actual_neo_ver) || name.contains(mc_version)) {
                    return Ok(name);
                }
            }
        }

        Ok(profile_name)
    }
}