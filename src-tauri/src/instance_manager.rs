use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::config;
use crate::java_manager::JavaManager;
use crate::launch::libraries::LibraryManager;
use crate::launch::loaders::LoaderManager;
use crate::launch::process::ProcessSupervisor;
use crate::launch::ManifestMerger;
use crate::system_utils::SystemUtils;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct InstanceManager {
    client: reqwest::Client,
    java: Arc<JavaManager>,
}

impl InstanceManager {
    pub fn new(java: Arc<JavaManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(10))
                .connect_timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            java,
        }
    }

    pub async fn ensure_version_downloaded(&self, version: &str, mc_dir: &Path, app: &AppHandle) -> Result<PathBuf, String> {
        let clean_ver = version.trim();
        let versions_dir = mc_dir.join("versions");
        let version_dir = versions_dir.join(clean_ver);
        let version_json_path = version_dir.join(format!("{}.json", clean_ver));

        let mut manifest_vdata: Option<Value> = None;

        if !version_json_path.exists() {
            let _ = app.emit("updateLaunchStatus", format!("Downloading manifest for {}...", clean_ver));
            let manifest_res = self.client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let manifest: Value = manifest_res.json().await.map_err(|e| e.to_string())?;
            let mut version_url = None;

            if let Some(versions) = manifest["versions"].as_array() {
                for v in versions {
                    if v["id"].as_str() == Some(clean_ver) {
                        version_url = v["url"].as_str().map(|s| s.to_string());
                        break;
                    }
                }
            }

            let url = version_url.ok_or_else(|| format!("Version {} not found in official manifest.", clean_ver))?;
            let v_res = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
            let v_bytes = v_res.bytes().await.map_err(|e| e.to_string())?;

            fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
            fs::write(&version_json_path, &v_bytes).map_err(|e| e.to_string())?;
            manifest_vdata = serde_json::from_slice(&v_bytes).ok();
        }

        let version_data: Value = if let Some(d) = manifest_vdata {
            d
        } else {
            let json_content = fs::read_to_string(&version_json_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&json_content).map_err(|e| e.to_string())?
        };

        let client_jar_path = version_dir.join(format!("{}.jar", clean_ver));
        let client_jar_valid = client_jar_path.exists() && client_jar_path.metadata().map(|m| m.len() > 1000).unwrap_or(false);

        if !client_jar_valid {
            let mut client_url_opt = version_data.get("downloads")
                .and_then(|d| d.get("client"))
                .and_then(|c| c.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            if client_url_opt.is_none() {
                if let Ok(manifest_res) = self.client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").send().await {
                    if let Ok(manifest) = manifest_res.json::<Value>().await {
                        if let Some(versions) = manifest["versions"].as_array() {
                            for v in versions {
                                if v["id"].as_str() == Some(clean_ver) {
                                    if let Some(v_url) = v["url"].as_str() {
                                        if let Ok(v_res) = self.client.get(v_url).send().await {
                                            if let Ok(official_vdata) = v_res.json::<Value>().await {
                                                client_url_opt = official_vdata.get("downloads")
                                                    .and_then(|d| d.get("client"))
                                                    .and_then(|c| c.get("url"))
                                                    .and_then(|u| u.as_str())
                                                    .map(|s| s.to_string());
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if let Some(client_url) = client_url_opt {
                let _ = app.emit("updateLaunchStatus", format!("Downloading Minecraft {} client...", clean_ver));
                let resp = self.client.get(&client_url).send().await.map_err(|e| e.to_string())?;
                if resp.status().is_success() {
                    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                    fs::write(&client_jar_path, bytes).map_err(|e| e.to_string())?;
                }
            }
        }

        if let Some(asset_index) = version_data.get("assetIndex") {
            if let (Some(id), Some(url)) = (asset_index["id"].as_str(), asset_index["url"].as_str()) {
                let index_path = mc_dir.join("assets").join("indexes").join(format!("{}.json", id));
                if !index_path.exists() || index_path.metadata().map(|m| m.len() < 10).unwrap_or(true) {
                    if let Some(parent) = index_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(res) = self.client.get(url).send().await {
                        if res.status().is_success() {
                            if let Ok(bytes) = res.bytes().await {
                                let _ = fs::write(&index_path, bytes);
                            }
                        }
                    }
                }
            }
        }

        LibraryManager::download_libraries(&self.client, &version_data, mc_dir).await;

        Ok(version_json_path)
    }

    pub async fn ensure_loader_profile(
        &self,
        loader: &str,
        mc_version: &str,
        loader_version: &str,
        mc_dir: &Path,
        app: &AppHandle,
    ) -> Result<String, String> {
        let clean_mc_ver = mc_version.trim();
        let versions_dir = mc_dir.join("versions");
        fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;

        let _ = self.ensure_version_downloaded(clean_mc_ver, mc_dir, app).await?;

        match loader.to_lowercase().as_str() {
            "fabric" => LoaderManager::ensure_fabric(&self.client, clean_mc_ver, loader_version, mc_dir, app).await,
            "quilt" => LoaderManager::ensure_quilt(&self.client, clean_mc_ver, loader_version, mc_dir, app).await,
            "forge" => LoaderManager::ensure_forge(&self.client, &self.java, clean_mc_ver, loader_version, mc_dir, app).await,
            "neoforge" => LoaderManager::ensure_neoforge(&self.client, &self.java, clean_mc_ver, loader_version, mc_dir, app).await,
            _ => Ok(clean_mc_ver.to_string()),
        }
    }

    pub async fn launch_game(
        &self,
        version: &str,
        loader: &str,
        loader_version: &str,
        mc_dir_str: &str,
        account: &Value,
        extra_jvm_args: Option<&str>,
        app: &AppHandle,
    ) -> Result<(String, u32), String> {
        let mc_dir = Path::new(mc_dir_str);
        if !mc_dir.exists() {
            let _ = fs::create_dir_all(mc_dir);
        }

        let clean_version = version.trim();
        let _ = self.ensure_version_downloaded(clean_version, mc_dir, app).await?;

        let launch_version = if loader.is_empty() || loader == "vanilla" {
            clean_version.to_string()
        } else {
            self.ensure_loader_profile(loader, clean_version, loader_version, mc_dir, app).await?
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
            if !parent_json_path.exists() {
                let _ = self.ensure_version_downloaded(parent_ver, mc_dir, app).await;
            }
            if let Ok(p_content) = fs::read_to_string(&parent_json_path) {
                if let Ok(p_data) = serde_json::from_str::<Value>(&p_content) {
                    version_data = ManifestMerger::merge(&version_data, &p_data);
                }
            }
        }

        let base_mc_ver = version_data.get("jar")
            .and_then(|v| v.as_str())
            .or_else(|| version_data.get("inheritsFrom").and_then(|v| v.as_str()))
            .unwrap_or(clean_version);
        let final_base_ver = base_mc_ver.trim();
        let base_jar_path = mc_dir.join("versions").join(final_base_ver).join(format!("{}.jar", final_base_ver));

        if !base_jar_path.exists() || base_jar_path.metadata().map(|m| m.len() < 1000).unwrap_or(true) {
            let _ = self.ensure_version_downloaded(final_base_ver, mc_dir, app).await?;
        }

        LibraryManager::download_libraries(&self.client, &version_data, mc_dir).await;

        let is_demo = account["access_token"].as_str().map_or(false, |tok| tok == "0");
        let main_class = version_data["mainClass"].as_str().ok_or("MainClass not found in version JSON.")?;
        let classpath = ProcessSupervisor::build_classpath(mc_dir, &version_data, &launch_version, final_base_ver, is_demo);

        let required_major = self.java.get_required_java_major(loader, final_base_ver, Some(&version_data));
        let java_path = match self.java.resolve_compatible_java(mc_dir, loader, final_base_ver, &version_data) {
            Ok(p) => p,
            Err(_) => self.java.ensure_java_runtime(mc_dir, required_major, app).await?,
        };

        let detected_java_major = self.java.check_java_binary(&java_path).unwrap_or(required_major);

        let natives_dir = mc_dir.join("versions").join(final_base_ver).join("natives");
        LibraryManager::extract_natives(mc_dir, &version_data, &natives_dir);

        let mut cmd = Command::new(&java_path);
        cmd.current_dir(mc_dir);

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd.env_remove("_JAVA_OPTIONS");
        cmd.env_remove("JAVA_TOOL_OPTIONS");
        cmd.env_remove("JAVA_OPTIONS");

        let app_config = config::load_app_config();
        let (_, default_jvm_args) = SystemUtils::generate_jvm_args_for_ram(app_config.ram_allocation);
        let mut jvm_args_str = if !app_config.custom_jvm_args.is_empty() {
            app_config.custom_jvm_args.clone()
        } else {
            default_jvm_args
        };

        if let Some(extra) = extra_jvm_args {
            jvm_args_str.push(' ');
            jvm_args_str.push_str(extra);
        }

        for arg in jvm_args_str.split_whitespace() {
            if ProcessSupervisor::filter_jvm_arg(arg, detected_java_major) {
                cmd.arg(arg);
            }
        }

        let natives_lossy = natives_dir.to_string_lossy().to_string();
        let mc_dir_lossy = mc_dir.to_string_lossy().to_string();
        let assets_dir = mc_dir.join("assets").to_string_lossy().to_string();
        let libraries_dir = mc_dir.join("libraries").to_string_lossy().to_string();
        let assets_idx = version_data["assets"].as_str().unwrap_or(final_base_ver);

        #[cfg(target_os = "windows")]
        let sep = ";";
        #[cfg(not(target_os = "windows"))]
        let sep = ":";

        let res_parts: Vec<&str> = app_config.game_resolution.split('x').collect();
        let (res_w, res_h) = if res_parts.len() == 2 {
            (res_parts[0].trim(), res_parts[1].trim())
        } else {
            ("854", "480")
        };

        let raw_uuid = account["uuid"].as_str().unwrap_or("00000000-0000-0000-0000-000000000000");
        let formatted_uuid = if raw_uuid.len() == 32 && !raw_uuid.contains('-') {
            format!("{}-{}-{}-{}-{}", &raw_uuid[0..8], &raw_uuid[8..12], &raw_uuid[12..16], &raw_uuid[16..20], &raw_uuid[20..32])
        } else {
            raw_uuid.to_string()
        };

        let auth_name = account["name"].as_str().unwrap_or("Player");
        let auth_token = account["access_token"].as_str().unwrap_or("0");
        let user_type = if auth_token == "0" { "mojang" } else { "msa" };

        let mut placeholders = HashMap::new();
        placeholders.insert("auth_player_name", auth_name);
        placeholders.insert("version_name", launch_version.as_str());
        placeholders.insert("game_directory", mc_dir_lossy.as_str());
        placeholders.insert("game_assets", assets_dir.as_str());
        placeholders.insert("assets_root", assets_dir.as_str());
        placeholders.insert("assets_index_name", assets_idx);
        placeholders.insert("auth_uuid", formatted_uuid.as_str());
        placeholders.insert("auth_access_token", auth_token);
        placeholders.insert("clientid", "0");
        placeholders.insert("auth_xuid", "0");
        placeholders.insert("user_properties", "{}");
        placeholders.insert("user_type", user_type);
        placeholders.insert("version_type", "release");
        placeholders.insert("natives_directory", natives_lossy.as_str());
        placeholders.insert("library_directory", libraries_dir.as_str());
        placeholders.insert("classpath_separator", sep);
        placeholders.insert("launcher_name", "KIP_Hub");
        placeholders.insert("launcher_version", config::APP_VERSION);
        placeholders.insert("classpath", classpath.as_str());
        placeholders.insert("resolution_width", res_w);
        placeholders.insert("resolution_height", res_h);
        placeholders.insert("mc_version", final_base_ver);
        placeholders.insert("primary_jar", final_base_ver);
        placeholders.insert("installer_path", "");

        if loader.to_lowercase() == "quilt" || loader.to_lowercase() == "fabric" {
            cmd.arg(format!("-Dfabric.gameDir={}", mc_dir_lossy));
            cmd.arg(format!("-Dquilt.gameDir={}", mc_dir_lossy));
        }

        let mut has_cp_passed = false;

        if let Some(jvm_args) = version_data["arguments"]["jvm"].as_array() {
            for arg_val in jvm_args {
                if let Some(arg_str) = arg_val.as_str() {
                    if arg_str == "-cp" || arg_str == "-classpath" {
                        has_cp_passed = true;
                    }
                    let parsed_arg = ProcessSupervisor::replace_placeholders(arg_str, &placeholders);
                    if ProcessSupervisor::filter_jvm_arg(&parsed_arg, detected_java_major) {
                        cmd.arg(parsed_arg);
                    }
                } else if let Some(rules) = arg_val["rules"].as_array() {
                    if ProcessSupervisor::is_rule_allowed(rules, is_demo) {
                        if let Some(v_str) = arg_val["value"].as_str() {
                            if v_str == "-cp" || v_str == "-classpath" {
                                has_cp_passed = true;
                            }
                            let parsed_arg = ProcessSupervisor::replace_placeholders(v_str, &placeholders);
                            if ProcessSupervisor::filter_jvm_arg(&parsed_arg, detected_java_major) {
                                cmd.arg(parsed_arg);
                            }
                        } else if let Some(v_arr) = arg_val["value"].as_array() {
                            for item in v_arr {
                                if let Some(s) = item.as_str() {
                                    if s == "-cp" || s == "-classpath" {
                                        has_cp_passed = true;
                                    }
                                    let parsed_arg = ProcessSupervisor::replace_placeholders(s, &placeholders);
                                    if ProcessSupervisor::filter_jvm_arg(&parsed_arg, detected_java_major) {
                                        cmd.arg(parsed_arg);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            cmd.arg(format!("-Djava.library.path={}", natives_lossy));
            cmd.arg("-cp").arg(&classpath);
            has_cp_passed = true;
        }

        if !has_cp_passed {
            cmd.arg("-cp").arg(&classpath);
        }

        cmd.arg(main_class);

        let mut has_game_dir = false;
        let mut has_resolution_passed = false;

        if let Some(game_args) = version_data["arguments"]["game"].as_array() {
            for arg_val in game_args {
                if let Some(arg_str) = arg_val.as_str() {
                    if arg_str == "--gameDir" {
                        has_game_dir = true;
                    }
                    if arg_str == "--width" {
                        has_resolution_passed = true;
                    }
                    cmd.arg(ProcessSupervisor::replace_placeholders(arg_str, &placeholders));
                } else if let Some(rules) = arg_val["rules"].as_array() {
                    if ProcessSupervisor::is_rule_allowed(rules, is_demo) {
                        if let Some(v_str) = arg_val["value"].as_str() {
                            if v_str == "--gameDir" {
                                has_game_dir = true;
                            }
                            if v_str == "--width" {
                                has_resolution_passed = true;
                            }
                            cmd.arg(ProcessSupervisor::replace_placeholders(v_str, &placeholders));
                        } else if let Some(v_arr) = arg_val["value"].as_array() {
                            for item in v_arr {
                                if let Some(s) = item.as_str() {
                                    if s == "--gameDir" {
                                        has_game_dir = true;
                                    }
                                    if s == "--width" {
                                        has_resolution_passed = true;
                                    }
                                    cmd.arg(ProcessSupervisor::replace_placeholders(s, &placeholders));
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
                cmd.arg(ProcessSupervisor::replace_placeholders(arg, &placeholders));
            }
        }

        if !has_game_dir {
            cmd.arg("--gameDir").arg(&mc_dir_lossy);
        }

        if !has_resolution_passed {
            cmd.arg("--width").arg(res_w);
            cmd.arg("--height").arg(res_h);
        }

        if app_config.game_fullscreen {
            cmd.arg("--fullscreen");
        }

        let pid = ProcessSupervisor::execute(cmd, app)?;
        Ok(("Game launched successfully.".to_string(), pid))
    }

    pub fn deploy_docker_server(&self, core_type: &str, version: &str, port: &str, mc_dir_str: &str) -> Result<String, String> {
        let mut check_cmd = Command::new("docker");
        check_cmd.arg("info");

        #[cfg(target_os = "windows")]
        {
            check_cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let docker_check = check_cmd.output();
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

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        match cmd.spawn() {
            Ok(_) => Ok(format!("Container deployed to port {}", port)),
            Err(e) => Err(format!("Docker compose failed: {}", e)),
        }
    }
}