use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use crate::config;
use crate::system_utils::SystemUtils;

pub struct InstanceManager {
    client: reqwest::Client,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn get_required_java_major(&self, loader: &str, version: &str, version_data: Option<&Value>) -> u32 {
        if loader.to_lowercase() == "quilt" {
            return 21;
        }

        if let Some(data) = version_data {
            if let Some(major) = data["javaVersion"]["majorVersion"].as_u64() {
                return major as u32;
            }
        }

        if version.starts_with("1.21") || version.starts_with("1.20.5") || version.starts_with("1.20.6") {
            21
        } else if version.starts_with("1.18") || version.starts_with("1.19") || version.starts_with("1.20") {
            17
        } else if version.starts_with("1.17") {
            16
        } else {
            8
        }
    }

    fn check_java_binary(&self, path: &str) -> Option<u32> {
        let output = Command::new(path).arg("-version").output().ok()?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ).to_lowercase();

        let quote_parts: Vec<&str> = text.split('"').collect();
        let ver_str = if quote_parts.len() > 1 {
            quote_parts[1]
        } else {
            return None;
        };

        let dot_parts: Vec<&str> = ver_str.split('.').collect();
        if dot_parts.is_empty() {
            return None;
        }

        if let Ok(first) = dot_parts[0].parse::<u32>() {
            if first == 1 && dot_parts.len() > 1 {
                return dot_parts[1].parse::<u32>().ok();
            }
            return Some(first);
        }
        None
    }

    fn find_compatible_java_binary(&self, mc_dir: &Path, required_major: u32) -> Option<String> {
        let mut candidates = Vec::new();

        let runtime_dir = mc_dir.join("runtime");
        if runtime_dir.exists() {
            let win_runtimes = [
                "java-runtime-delta/windows/java-runtime-delta/bin/javaw.exe",
                "java-runtime-gamma/windows/java-runtime-gamma/bin/javaw.exe",
                "java-runtime-beta/windows/java-runtime-beta/bin/javaw.exe",
                "java-runtime-alpha/windows/java-runtime-alpha/bin/javaw.exe",
                "jre-legacy/windows/jre-legacy/bin/javaw.exe",
            ];
            for r in win_runtimes {
                candidates.push(runtime_dir.join(r));
            }

            let unix_runtimes = [
                "java-runtime-delta/bin/java",
                "java-runtime-gamma/bin/java",
                "java-runtime-beta/bin/java",
                "jre-legacy/bin/java",
            ];
            for r in unix_runtimes {
                candidates.push(runtime_dir.join(r));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Some(local_app_data) = dirs::data_local_dir() {
                let ms_store_pkg = local_app_data.join("Packages");
                if let Ok(entries) = fs::read_dir(ms_store_pkg) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("Microsoft.4297127D64C6C") {
                            let runtime_sub = entry.path().join("LocalCache").join("Local").join("runtime");
                            if runtime_sub.exists() {
                                let sub_win = [
                                    "java-runtime-delta/windows/java-runtime-delta/bin/javaw.exe",
                                    "java-runtime-gamma/windows/java-runtime-gamma/bin/javaw.exe",
                                    "java-runtime-beta/windows/java-runtime-beta/bin/javaw.exe",
                                    "jre-legacy/windows/jre-legacy/bin/javaw.exe",
                                ];
                                for sw in sub_win {
                                    candidates.push(runtime_sub.join(sw));
                                }
                            }
                        }
                    }
                }
            }

            let win_dirs = [
                "C:\\Program Files (x86)\\Minecraft Launcher\\runtime\\java-runtime-delta\\windows\\java-runtime-delta\\bin\\javaw.exe",
                "C:\\Program Files (x86)\\Minecraft Launcher\\runtime\\java-runtime-gamma\\windows\\java-runtime-gamma\\bin\\javaw.exe",
                "C:\\Program Files (x86)\\Minecraft Launcher\\runtime\\java-runtime-beta\\windows\\java-runtime-beta\\bin\\javaw.exe",
                "C:\\Program Files\\Eclipse Adoptium",
                "C:\\Program Files\\Microsoft",
                "C:\\Program Files\\Java",
                "C:\\Program Files\\BellSoft",
                "C:\\Program Files\\Amazon Corretto",
                "C:\\Program Files\\Zulu",
            ];

            for d in win_dirs {
                let p = Path::new(d);
                if p.is_file() {
                    candidates.push(p.to_path_buf());
                } else if p.is_dir() {
                    if let Ok(entries) = fs::read_dir(p) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let sub_javaw = entry.path().join("bin").join("javaw.exe");
                            if sub_javaw.exists() {
                                candidates.push(sub_javaw);
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let jvm_dir = Path::new("/usr/lib/jvm");
            if let Ok(entries) = fs::read_dir(jvm_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let sub = entry.path().join("bin").join("java");
                    if sub.exists() {
                        candidates.push(sub);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let jvm_dir = Path::new("/Library/Java/JavaVirtualMachines");
            if let Ok(entries) = fs::read_dir(jvm_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let sub = entry.path().join("Contents").join("Home").join("bin").join("java");
                    if sub.exists() {
                        candidates.push(sub);
                    }
                }
            }
        }

        for c in candidates {
            if c.exists() {
                let path_str = c.to_string_lossy().to_string();
                if let Some(major) = self.check_java_binary(&path_str) {
                    if major == required_major || (required_major == 17 && major == 21) {
                        return Some(path_str);
                    }
                }
            }
        }

        None
    }

    fn resolve_compatible_java(&self, mc_dir: &Path, loader: &str, version: &str, version_data: &Value) -> Result<String, String> {
        let app_config = config::load_app_config();
        if !app_config.custom_java_path.is_empty() && Path::new(&app_config.custom_java_path).exists() {
            return Ok(app_config.custom_java_path);
        }

        let required_major = self.get_required_java_major(loader, version, Some(version_data));
        let system_major = self.check_java_binary("java").unwrap_or(0);

        if loader.to_lowercase() == "quilt" && system_major > 21 {
            if let Some(found_java) = self.find_compatible_java_binary(mc_dir, 21).or_else(|| self.find_compatible_java_binary(mc_dir, 17)) {
                return Ok(found_java);
            }
        }

        if system_major == required_major || (required_major == 17 && system_major == 21) {
            return Ok("java".to_string());
        }

        if let Some(found_java) = self.find_compatible_java_binary(mc_dir, required_major) {
            return Ok(found_java);
        }

        Ok("java".to_string())
    }

    async fn ensure_version_downloaded(&self, version: &str, mc_dir: &Path, app: &AppHandle) -> Result<PathBuf, String> {
        let versions_dir = mc_dir.join("versions");
        let version_dir = versions_dir.join(version);
        let version_json_path = version_dir.join(format!("{}.json", version));

        if !version_json_path.exists() {
            let _ = app.emit("updateLaunchStatus", format!("Downloading manifest for {}...", version));
            let manifest_res = self.client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let manifest: Value = manifest_res.json().await.map_err(|e| e.to_string())?;
            let mut version_url = None;

            if let Some(versions) = manifest["versions"].as_array() {
                for v in versions {
                    if v["id"].as_str() == Some(version) {
                        version_url = v["url"].as_str().map(|s| s.to_string());
                        break;
                    }
                }
            }

            let url = version_url.ok_or_else(|| format!("Version {} not found in official manifest.", version))?;
            let v_res = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
            let v_bytes = v_res.bytes().await.map_err(|e| e.to_string())?;

            fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
            fs::write(&version_json_path, v_bytes).map_err(|e| e.to_string())?;
        }

        let json_content = fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
        let version_data: Value = serde_json::from_str(&json_content).map_err(|e| e.to_string())?;

        let client_jar_path = version_dir.join(format!("{}.jar", version));
        if !client_jar_path.exists() {
            if let Some(client_url) = version_data["downloads"]["client"]["url"].as_str() {
                let _ = app.emit("updateLaunchStatus", format!("Downloading Minecraft {} client...", version));
                let resp = self.client.get(client_url).send().await.map_err(|e| e.to_string())?;
                let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                fs::write(&client_jar_path, bytes).map_err(|e| e.to_string())?;
            }
        }

        self.download_libraries_from_manifest(&version_data, mc_dir, app).await;

        Ok(version_json_path)
    }

    async fn ensure_loader_profile(
        &self,
        loader: &str,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let versions_dir = mc_dir.join("versions");
        fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;

        let _ = self.ensure_version_downloaded(mc_version, mc_dir, app).await?;

        match loader.to_lowercase().as_str() {
            "fabric" => {
                let actual_loader_ver = if loader_version.is_empty() {
                    let meta_url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
                    let resp = self.client.get(&meta_url).send().await.map_err(|e| e.to_string())?;
                    let arr: Value = resp.json().await.map_err(|e| e.to_string())?;
                    arr.as_array()
                        .and_then(|a| a.first())
                        .and_then(|v| v["loader"]["version"].as_str())
                        .ok_or_else(|| "Failed to resolve latest Fabric loader version".to_string())?
                        .to_string()
                } else {
                    loader_version.to_string()
                };

                let profile_name = format!("fabric-loader-{}-{}", actual_loader_ver, mc_version);
                let profile_dir = versions_dir.join(&profile_name);
                let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

                if !profile_json_path.exists() {
                    let _ = app.emit("updateLaunchStatus", format!("Installing Fabric {}...", actual_loader_ver));
                    let profile_url = format!(
                        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                        mc_version, actual_loader_ver
                    );
                    let resp = self.client.get(&profile_url).send().await.map_err(|e| e.to_string())?;
                    let profile_bytes = resp.bytes().await.map_err(|e| e.to_string())?;

                    fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
                    fs::write(&profile_json_path, &profile_bytes).map_err(|e| e.to_string())?;
                }

                if let Ok(content) = fs::read_to_string(&profile_json_path) {
                    if let Ok(data) = serde_json::from_str::<Value>(&content) {
                        self.download_libraries_from_manifest(&data, mc_dir, app).await;
                    }
                }

                Ok(profile_name)
            }
            "quilt" => {
                let actual_loader_ver = if loader_version.is_empty() {
                    let meta_url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
                    let mut resolved = None;
                    if let Ok(resp) = self.client.get(&meta_url).send().await {
                        if let Ok(arr) = resp.json::<Value>().await {
                            resolved = arr.as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v["loader"]["version"].as_str())
                                .map(|s| s.to_string());
                        }
                    }

                    if resolved.is_none() {
                        let global_url = "https://meta.quiltmc.org/v3/versions/loader";
                        let resp = self.client.get(global_url).send().await.map_err(|e| e.to_string())?;
                        let arr: Value = resp.json().await.map_err(|e| e.to_string())?;
                        resolved = arr.as_array()
                            .and_then(|a| a.first())
                            .and_then(|v| v["version"].as_str())
                            .map(|s| s.to_string());
                    }

                    resolved.ok_or_else(|| "Failed to resolve Quilt loader version".to_string())?
                } else {
                    loader_version.to_string()
                };

                let profile_name = format!("quilt-loader-{}-{}", actual_loader_ver, mc_version);
                let profile_dir = versions_dir.join(&profile_name);
                let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

                if !profile_json_path.exists() {
                    let _ = app.emit("updateLaunchStatus", format!("Installing Quilt {}...", actual_loader_ver));
                    let profile_url = format!(
                        "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
                        mc_version, actual_loader_ver
                    );

                    let resp = self.client.get(&profile_url).send().await.map_err(|e| e.to_string())?;
                    let profile_bytes = resp.bytes().await.map_err(|e| e.to_string())?;

                    fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
                    fs::write(&profile_json_path, &profile_bytes).map_err(|e| e.to_string())?;
                }

                if let Ok(content) = fs::read_to_string(&profile_json_path) {
                    if let Ok(data) = serde_json::from_str::<Value>(&content) {
                        self.download_libraries_from_manifest(&data, mc_dir, app).await;
                    }
                }

                Ok(profile_name)
            }
            "forge" => {
                let actual_forge_ver = if loader_version.is_empty() {
                    let promo_url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
                    let resp = self.client.get(promo_url).send().await.map_err(|e| e.to_string())?;
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

                let forge_client_jar = mc_dir.join("libraries").join("net").join("minecraftforge").join("forge")
                    .join(format!("{}-{}", mc_version, actual_forge_ver))
                    .join(format!("forge-{}-{}-client.jar", mc_version, actual_forge_ver));

                if !profile_json_path.exists() || !forge_client_jar.exists() {
                    let _ = app.emit("updateLaunchStatus", format!("Installing Forge {}...", actual_forge_ver));
                    let installer_url = format!(
                        "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                        mc_version, actual_forge_ver, mc_version, actual_forge_ver
                    );

                    let resp = self.client.get(&installer_url).send().await.map_err(|e| e.to_string())?;
                    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

                    let installer_path = mc_dir.join("forge-installer-temp.jar");
                    fs::write(&installer_path, bytes).map_err(|e| e.to_string())?;

                    let dummy_val = json!({ "javaVersion": { "majorVersion": 17 } });
                    let java_bin = self.resolve_compatible_java(mc_dir, loader, mc_version, &dummy_val)?;

                    let mut child = Command::new(&java_bin)
                        .arg("-jar")
                        .arg(&installer_path)
                        .arg("--installClient")
                        .arg(mc_dir.to_string_lossy().to_string())
                        .stdout(Stdio::piped())
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
            "neoforge" => {
                let actual_neo_ver = if loader_version.is_empty() {
                    let meta_url = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
                    let resp = self.client.get(meta_url).send().await.map_err(|e| e.to_string())?;
                    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

                    data["versions"].as_array()
                        .and_then(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .filter(|s| s.starts_with(mc_version) || s.starts_with(&mc_version.replace("1.", "")))
                                .last()
                        })
                        .ok_or_else(|| format!("No NeoForge version found for Minecraft {}", mc_version))?
                        .to_string()
                } else {
                    loader_version.to_string()
                };

                let profile_name = format!("neoforge-{}", actual_neo_ver);
                let profile_dir = versions_dir.join(&profile_name);
                let profile_json_path = profile_dir.join(format!("{}.json", profile_name));

                if !profile_json_path.exists() {
                    let _ = app.emit("updateLaunchStatus", format!("Installing NeoForge {}...", actual_neo_ver));
                    let installer_url = format!(
                        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
                        actual_neo_ver, actual_neo_ver
                    );

                    let resp = self.client.get(&installer_url).send().await.map_err(|e| e.to_string())?;
                    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

                    let installer_path = mc_dir.join("neoforge-installer-temp.jar");
                    fs::write(&installer_path, bytes).map_err(|e| e.to_string())?;

                    let dummy_val = json!({ "javaVersion": { "majorVersion": 21 } });
                    let java_bin = self.resolve_compatible_java(mc_dir, loader, mc_version, &dummy_val)?;

                    let mut child = Command::new(&java_bin)
                        .arg("-jar")
                        .arg(&installer_path)
                        .arg("--install-client")
                        .arg(mc_dir.to_string_lossy().to_string())
                        .stdout(Stdio::piped())
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
            _ => Ok(mc_version.to_string()),
        }
    }

    async fn download_libraries_from_manifest(&self, manifest: &Value, mc_dir: &Path, app: &AppHandle) {
        if let Some(libs) = manifest["libraries"].as_array() {
            let total_libs = libs.len();
            let mut processed = 0;

            for lib in libs {
                processed += 1;
                let mut download_url = None;
                let mut rel_path = None;

                if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                    download_url = artifact["url"].as_str().map(|s| s.to_string());
                    rel_path = artifact["path"].as_str().map(PathBuf::from);
                } else if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() >= 3 {
                        let group = parts[0].replace('.', "/");
                        let artifact = parts[1];
                        let version = parts[2];
                        let classifier = if parts.len() > 3 { format!("-{}", parts[3]) } else { String::new() };
                        let jar_name = format!("{}-{}{}.jar", artifact, version, classifier);
                        let sub_path = PathBuf::from(group).join(artifact).join(version).join(&jar_name);

                        let base = lib.get("url")
                            .and_then(|u| u.as_str())
                            .unwrap_or("https://libraries.minecraft.net/");
                        let base_clean = base.trim_end_matches('/');
                        let formatted_sub = sub_path.to_string_lossy().replace('\\', "/");
                        download_url = Some(format!("{}/{}", base_clean, formatted_sub));
                        rel_path = Some(sub_path);
                    }
                }

                if let (Some(url_str), Some(path_buf)) = (download_url, rel_path) {
                    let full_path = mc_dir.join("libraries").join(path_buf);
                    if !full_path.exists() {
                        if let Some(parent) = full_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let percent = if total_libs > 0 {
                            (processed as f64 / total_libs as f64) * 100.0
                        } else {
                            100.0
                        };
                        let _ = app.emit("updateLaunchStatus", format!("Resolving library {} of {}...", processed, total_libs));
                        let _ = app.emit("updateLaunchProgress", percent);
                        if let Ok(res) = self.client.get(&url_str).send().await {
                            if let Ok(bytes) = res.bytes().await {
                                let _ = fs::write(&full_path, bytes);
                            }
                        }
                    }
                }
            }
        }
    }

    fn resolve_library_path(&self, mc_dir: &Path, lib: &Value) -> Option<PathBuf> {
        if let Some(artifact_path) = lib["downloads"]["artifact"]["path"].as_str() {
            let path = mc_dir.join("libraries").join(artifact_path);
            if path.exists() {
                return Some(path);
            }
        }

        if let Some(name) = lib["name"].as_str() {
            let parts: Vec<&str> = name.split(':').collect();
            if parts.len() >= 3 {
                let group = parts[0].replace('.', "/");
                let artifact = parts[1];
                let version = parts[2];
                let classifier = if parts.len() > 3 { format!("-{}", parts[3]) } else { String::new() };
                let jar_name = format!("{}-{}{}.jar", artifact, version, classifier);
                let path = mc_dir.join("libraries").join(group).join(artifact).join(version).join(jar_name);
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }

    fn is_rule_allowed(&self, rules: &[Value], is_demo: bool) -> bool {
        let mut allow = false;
        for rule in rules {
            let action = rule["action"].as_str().unwrap_or("");
            let os_match = rule["os"]["name"].as_str().map_or(true, |os| {
                match os {
                    "windows" => cfg!(target_os = "windows"),
                    "osx" => cfg!(target_os = "macos"),
                    "linux" => cfg!(target_os = "linux"),
                    _ => false,
                }
            });

            let mut features_match = true;
            if let Some(features) = rule["features"].as_object() {
                for (feat, expected) in features {
                    let expected_val = expected.as_bool().unwrap_or(false);
                    match feat.as_str() {
                        "is_demo_user" => {
                            if is_demo != expected_val {
                                features_match = false;
                                break;
                            }
                        }
                        "is_quick_play_singleplayer"
                        | "is_quick_play_multiplayer"
                        | "is_quick_play_realms" => {
                            if expected_val {
                                features_match = false;
                                break;
                            }
                        }
                        _ => {
                            if expected_val {
                                features_match = false;
                                break;
                            }
                        }
                    }
                }
            }

            if os_match && features_match {
                allow = action == "allow";
            }
        }
        allow
    }

    fn merge_manifests(&self, child: &Value, parent: &Value) -> Value {
        let mut merged = parent.clone();

        if let Some(id) = child["id"].as_str() {
            merged["id"] = json!(id);
        }
        if let Some(main_class) = child["mainClass"].as_str() {
            merged["mainClass"] = json!(main_class);
        }

        if let (Some(p_libs), Some(c_libs)) = (merged["libraries"].as_array_mut(), child["libraries"].as_array()) {
            for c_lib in c_libs {
                p_libs.push(c_lib.clone());
            }
        } else if child["libraries"].is_array() {
            merged["libraries"] = child["libraries"].clone();
        }

        if let Some(c_args) = child.get("arguments") {
            if let Some(c_jvm) = c_args.get("jvm").and_then(|j| j.as_array()) {
                if let Some(p_jvm) = merged["arguments"]["jvm"].as_array_mut() {
                    for arg in c_jvm {
                        p_jvm.push(arg.clone());
                    }
                } else {
                    merged["arguments"]["jvm"] = json!(c_jvm);
                }
            }
            if let Some(c_game) = c_args.get("game").and_then(|g| g.as_array()) {
                if let Some(p_game) = merged["arguments"]["game"].as_array_mut() {
                    for arg in c_game {
                        p_game.push(arg.clone());
                    }
                } else {
                    merged["arguments"]["game"] = json!(c_game);
                }
            }
        }

        merged
    }

    fn build_classpath(&self, mc_dir: &Path, version_data: &Value, launch_version: &str, vanilla_version: &str, is_demo: bool) -> String {
        let mut cp_entries: Vec<String> = Vec::new();

        if let Some(libs) = version_data["libraries"].as_array() {
            for lib in libs {
                let allow = if let Some(rules) = lib["rules"].as_array() {
                    self.is_rule_allowed(rules, is_demo)
                } else {
                    true
                };

                if allow {
                    if let Some(path) = self.resolve_library_path(mc_dir, lib) {
                        let path_str = path.to_string_lossy().to_string();
                        if !cp_entries.contains(&path_str) {
                            cp_entries.push(path_str);
                        }
                    }
                }
            }
        }

        let custom_jar = mc_dir.join("versions").join(launch_version).join(format!("{}.jar", launch_version));
        let vanilla_jar = mc_dir.join("versions").join(vanilla_version).join(format!("{}.jar", vanilla_version));

        if custom_jar.exists() {
            let p = custom_jar.to_string_lossy().to_string();
            if !cp_entries.contains(&p) {
                cp_entries.push(p);
            }
        } else if vanilla_jar.exists() {
            let p = vanilla_jar.to_string_lossy().to_string();
            if !cp_entries.contains(&p) {
                cp_entries.push(p);
            }
        }

        #[cfg(target_os = "windows")]
        let sep = ";";
        #[cfg(not(target_os = "windows"))]
        let sep = ":";

        cp_entries.join(sep)
    }

    fn replace_placeholders(&self, text: &str, vars: &HashMap<&str, &str>) -> String {
        let mut result = text.to_string();
        for (k, v) in vars {
            result = result.replace(&format!("${{{}}}", k), v);
        }
        result
    }

    pub async fn launch_game(
        &self,
        version: &str,
        loader: &str,
        loader_version: &str,
        mc_dir_str: &str,
        account: &Value,
        app: &AppHandle,
    ) -> Result<(String, u32), String> {
        let mc_dir = Path::new(mc_dir_str);
        if !mc_dir.exists() {
            let _ = fs::create_dir_all(mc_dir);
        }

        let launch_version = if loader.is_empty() || loader == "vanilla" {
            let _ = self.ensure_version_downloaded(version, mc_dir, app).await?;
            version.to_string()
        } else {
            self.ensure_loader_profile(loader, version, loader_version, mc_dir, app).await?
        };

        let versions_dir = mc_dir.join("versions");
        let version_json_path = versions_dir.join(&launch_version).join(format!("{}.json", launch_version));

        if !version_json_path.exists() {
            return Err(format!("Version profile JSON not found for '{}'.", launch_version));
        }

        let json_content = fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
        let mut version_data: Value = serde_json::from_str(&json_content).map_err(|e| e.to_string())?;

        if let Some(parent_ver) = version_data.get("inheritsFrom").and_then(|v| v.as_str()) {
            let parent_json_path = versions_dir.join(parent_ver).join(format!("{}.json", parent_ver));
            if parent_json_path.exists() {
                if let Ok(p_content) = fs::read_to_string(&parent_json_path) {
                    if let Ok(p_data) = serde_json::from_str::<Value>(&p_content) {
                        version_data = self.merge_manifests(&version_data, &p_data);
                    }
                }
            }
        }

        let is_demo = account["access_token"].as_str().map_or(false, |tok| tok == "0");
        let main_class = version_data["mainClass"].as_str().ok_or("MainClass not found in version JSON.")?;
        let classpath = self.build_classpath(mc_dir, &version_data, &launch_version, version, is_demo);

        let java_path = self.resolve_compatible_java(mc_dir, loader, version, &version_data)?;

        let mut cmd = Command::new(&java_path);
        cmd.current_dir(mc_dir);

        cmd.env_remove("_JAVA_OPTIONS");
        cmd.env_remove("JAVA_TOOL_OPTIONS");
        cmd.env_remove("JAVA_OPTIONS");

        let app_config = config::load_app_config();
        let (_, default_jvm_args) = SystemUtils::generate_jvm_args_for_ram(app_config.ram_allocation);
        let jvm_args_str = if !app_config.custom_jvm_args.is_empty() {
            &app_config.custom_jvm_args
        } else {
            &default_jvm_args
        };

        for arg in jvm_args_str.split_whitespace().filter(|a| !a.contains("UseCompactObjectHeaders")) {
            cmd.arg(arg);
        }

        let natives_dir = mc_dir.join("versions").join(version).join("natives");
        let _ = fs::create_dir_all(&natives_dir);

        let natives_lossy = natives_dir.to_string_lossy().to_string();
        let mc_dir_lossy = mc_dir.to_string_lossy().to_string();
        let assets_dir = mc_dir.join("assets").to_string_lossy().to_string();
        let libraries_dir = mc_dir.join("libraries").to_string_lossy().to_string();
        let assets_idx = version_data["assets"].as_str().unwrap_or(version);

        #[cfg(target_os = "windows")]
        let sep = ";";
        #[cfg(not(target_os = "windows"))]
        let sep = ":";

        let mut placeholders = HashMap::new();
        let auth_name = account["name"].as_str().unwrap_or("Player");
        let auth_uuid = account["uuid"].as_str().unwrap_or("00000000000000000000000000000000");
        let auth_token = account["access_token"].as_str().unwrap_or("0");

        placeholders.insert("auth_player_name", auth_name);
        placeholders.insert("version_name", launch_version.as_str());
        placeholders.insert("game_directory", mc_dir_lossy.as_str());
        placeholders.insert("assets_root", assets_dir.as_str());
        placeholders.insert("assets_index_name", assets_idx);
        placeholders.insert("auth_uuid", auth_uuid);
        placeholders.insert("auth_access_token", auth_token);
        placeholders.insert("user_type", "msa");
        placeholders.insert("version_type", "release");
        placeholders.insert("natives_directory", natives_lossy.as_str());
        placeholders.insert("library_directory", libraries_dir.as_str());
        placeholders.insert("classpath_separator", sep);
        placeholders.insert("launcher_name", "KIP_Hub");
        placeholders.insert("launcher_version", config::APP_VERSION);
        placeholders.insert("classpath", classpath.as_str());

        if loader.to_lowercase() == "quilt" {
            cmd.arg(format!("-Dquilt.gameDir={}", mc_dir_lossy));
            cmd.arg(format!("-Dfabric.gameDir={}", mc_dir_lossy));
        }

        if let Some(jvm_args) = version_data["arguments"]["jvm"].as_array() {
            for arg_val in jvm_args {
                if let Some(arg_str) = arg_val.as_str() {
                    cmd.arg(self.replace_placeholders(arg_str, &placeholders));
                } else if let Some(rules) = arg_val["rules"].as_array() {
                    if self.is_rule_allowed(rules, is_demo) {
                        if let Some(v_str) = arg_val["value"].as_str() {
                            cmd.arg(self.replace_placeholders(v_str, &placeholders));
                        } else if let Some(v_arr) = arg_val["value"].as_array() {
                            for item in v_arr {
                                if let Some(s) = item.as_str() {
                                    cmd.arg(self.replace_placeholders(s, &placeholders));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            cmd.arg(format!("-Djava.library.path={}", natives_lossy));
            cmd.arg("-cp").arg(&classpath);
        }

        cmd.arg(main_class);

        let mut has_game_dir = false;

        if let Some(game_args) = version_data["arguments"]["game"].as_array() {
            for arg_val in game_args {
                if let Some(arg_str) = arg_val.as_str() {
                    if arg_str == "--gameDir" {
                        has_game_dir = true;
                    }
                    cmd.arg(self.replace_placeholders(arg_str, &placeholders));
                } else if let Some(rules) = arg_val["rules"].as_array() {
                    if self.is_rule_allowed(rules, is_demo) {
                        if let Some(v_str) = arg_val["value"].as_str() {
                            if v_str == "--gameDir" {
                                has_game_dir = true;
                            }
                            cmd.arg(self.replace_placeholders(v_str, &placeholders));
                        } else if let Some(v_arr) = arg_val["value"].as_array() {
                            for item in v_arr {
                                if let Some(s) = item.as_str() {
                                    if s == "--gameDir" {
                                        has_game_dir = true;
                                    }
                                    cmd.arg(self.replace_placeholders(s, &placeholders));
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(legacy_args) = version_data["minecraftArguments"].as_str() {
            for arg in legacy_args.split_whitespace() {
                if arg == "--gameDir" {
                    has_game_dir = true;
                }
                cmd.arg(self.replace_placeholders(arg, &placeholders));
            }
        }

        if !has_game_dir {
            cmd.arg("--gameDir").arg(&mc_dir_lossy);
        }

        let res_parts: Vec<&str> = app_config.game_resolution.split('x').collect();
        if res_parts.len() == 2 {
            cmd.arg("--width").arg(res_parts[0].trim());
            cmd.arg("--height").arg(res_parts[1].trim());
        }

        if app_config.game_fullscreen {
            cmd.arg("--fullscreen");
        }

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        match cmd.spawn() {
            Ok(child) => Ok(("Game launched successfully.".to_string(), child.id())),
            Err(e) => Err(format!("Failed to start game process: {}", e)),
        }
    }

    pub async fn create_local_server(&self, core_type: &str, version: &str, mc_dir_str: &str) -> Result<String, String> {
        let server_dir = Path::new(mc_dir_str).join("local_servers").join(format!("{}_{}", core_type, version));
        fs::create_dir_all(&server_dir).map_err(|e| e.to_string())?;

        let jar_dest = server_dir.join("server.jar");

        let download_url = match core_type.to_lowercase().as_str() {
            "paper" => {
                let api_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}", version);
                let res = self.client.get(&api_url).send().await.map_err(|e| e.to_string())?;
                let data: Value = res.json().await.map_err(|e| e.to_string())?;
                let builds = data["builds"].as_array().ok_or("No builds found")?;
                let latest_build = builds.last().and_then(|v| v.as_i64()).ok_or("Invalid build ID")?;
                format!("{}/builds/{}/downloads/paper-{}-{}.jar", api_url, latest_build, version, latest_build)
            }
            "fabric" => {
                format!("https://meta.fabricmc.net/v2/versions/loader/{}/0.15.7/1.0.1/server/jar", version)
            }
            _ => return Err("Unsupported core type.".to_string()),
        };

        let response = self.client.get(&download_url).send().await.map_err(|e| e.to_string())?;
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        fs::write(&jar_dest, bytes).map_err(|e| e.to_string())?;

        let eula_path = server_dir.join("eula.txt");
        fs::write(eula_path, "eula=true\n").map_err(|e| e.to_string())?;

        let (_, jvm_args) = SystemUtils::generate_jvm_args();

        #[cfg(target_os = "windows")]
        {
            let bat_path = server_dir.join("start.bat");
            let content = format!("@echo off\njava {} -jar server.jar nogui\npause\n", jvm_args);
            fs::write(bat_path, content).map_err(|e| e.to_string())?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            let sh_path = server_dir.join("start.sh");
            let content = format!("#!/bin/bash\njava {} -jar server.jar nogui\n", jvm_args);
            fs::write(&sh_path, content).map_err(|e| e.to_string())?;
        }

        Ok(server_dir.to_string_lossy().to_string())
    }

    pub fn deploy_docker_server(&self, core_type: &str, version: &str, port: &str, mc_dir_str: &str) -> Result<String, String> {
        let docker_check = Command::new("docker").arg("info").output();
        if docker_check.is_err() || !docker_check.unwrap().status.success() {
            return Err("Docker daemon is not running or not installed.".to_string());
        }

        let server_dir = Path::new(mc_dir_str).join("local_servers").join(format!("docker_{}_{}", core_type, version));
        let data_dir = server_dir.join("data");
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        let compose_content = format!(
            "version: \"3.8\"\nservices:\n  mc_server:\n    image: itzg/minecraft-server\n    container_name: kip_mc_{}_{}\n    ports:\n      - \"{}:25565\"\n    environment:\n      EULA: \"TRUE\"\n      TYPE: \"{}\"\n      VERSION: \"{}\"\n      MEMORY: \"4G\"\n    volumes:\n      - ./data:/data\n    restart: unless-stopped\n",
            core_type, version, port, core_type.to_uppercase(), version
        );

        let compose_path = server_dir.join("docker-compose.yml");
        fs::write(compose_path, compose_content).map_err(|e| e.to_string())?;

        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("up").arg("-d").current_dir(&server_dir);

        match cmd.spawn() {
            Ok(_) => Ok(format!("Container deployed to port {}", port)),
            Err(e) => Err(format!("Docker compose failed: {}", e)),
        }
    }
}