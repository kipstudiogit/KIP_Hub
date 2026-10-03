use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use tokio::sync::Semaphore;
use zip::ZipArchive;

pub struct LibraryManager;

impl LibraryManager {
    pub fn is_architecture_compatible(lib_name: &str) -> bool {
        let name = lib_name.to_lowercase();
        #[cfg(target_arch = "x86_64")]
        {
            if name.contains("arm64")
                || name.contains("aarch64")
                || name.ends_with("-x86")
                || name.ends_with(":x86")
                || name.contains(":natives-windows-x86:")
                || name.ends_with(":natives-windows-x86")
                || name.contains("-x86.jar")
            {
                return false;
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if (name.contains("x86_64") || name.contains("x64"))
                && !name.contains("arm64")
                && !name.contains("aarch64")
            {
                return false;
            }
        }
        true
    }

    pub fn get_library_key(lib_name: &str) -> String {
        let parts: Vec<&str> = lib_name.split(':').collect();
        if parts.len() >= 4 {
            format!("{}:{}:{}", parts[0], parts[1], parts[3])
        } else if parts.len() >= 2 {
            format!("{}:{}", parts[0], parts[1])
        } else {
            lib_name.to_string()
        }
    }

    pub fn resolve_library_paths(mc_dir: &Path, lib: &Value) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let os_key = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };

        let lib_name_str = lib.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if !Self::is_architecture_compatible(lib_name_str) {
            return results;
        }

        if let Some(artifact_path) = lib.get("downloads").and_then(|d| d.get("artifact")).and_then(|a| a.get("path")).and_then(|p| p.as_str()) {
            let path = mc_dir.join("libraries").join(artifact_path);
            if path.exists() && path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                results.push(path);
            }
        }

        if let Some(native_classifier) = lib.get("natives").and_then(|n| n.get(os_key)).and_then(|s| s.as_str()) {
            if let Some(classifiers) = lib.get("downloads").and_then(|d| d.get("classifiers")) {
                if let Some(path_str) = classifiers.get(native_classifier).and_then(|a| a.get("path")).and_then(|p| p.as_str()) {
                    let path = mc_dir.join("libraries").join(path_str);
                    if path.exists() && path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                        results.push(path);
                    }
                }
            }
        }

        if results.is_empty() && !lib_name_str.is_empty() {
            let parts: Vec<&str> = lib_name_str.split(':').collect();
            if parts.len() >= 3 {
                let group = parts[0].replace('.', "/");
                let artifact = parts[1];
                let version = parts[2];
                let classifier = if parts.len() > 3 { format!("-{}", parts[3]) } else { String::new() };
                let jar_name = format!("{}-{}{}.jar", artifact, version, classifier);
                let path = mc_dir.join("libraries").join(&group).join(artifact).join(version).join(&jar_name);
                if path.exists() && path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                    results.push(path);
                }

                let universal_path = mc_dir.join("libraries").join(&group).join(artifact).join(version).join(format!("{}-{}-universal.jar", artifact, version));
                if universal_path.exists() && universal_path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                    results.push(universal_path);
                }

                let client_path = mc_dir.join("libraries").join(&group).join(artifact).join(version).join(format!("{}-{}-client.jar", artifact, version));
                if client_path.exists() && client_path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                    results.push(client_path);
                }
            }
        }
        results
    }

    pub fn extract_natives(mc_dir: &Path, version_data: &Value, natives_dir: &Path) {
        let _ = fs::create_dir_all(natives_dir);

        if let Some(libs) = version_data["libraries"].as_array() {
            for lib in libs {
                let name = lib["name"].as_str().unwrap_or("");
                if !Self::is_architecture_compatible(name) {
                    continue;
                }

                let is_native = name.contains("natives")
                    || lib.get("natives").is_some()
                    || lib.get("downloads").and_then(|d| d.get("classifiers")).is_some();

                if is_native {
                    for path in Self::resolve_library_paths(mc_dir, lib) {
                        if let Ok(f) = File::open(&path) {
                            if let Ok(mut zip) = ZipArchive::new(f) {
                                for i in 0..zip.len() {
                                    if let Ok(mut zf) = zip.by_index(i) {
                                        let fname = zf.name().to_string();
                                        if !fname.starts_with("META-INF") && (fname.ends_with(".dll") || fname.ends_with(".so") || fname.ends_with(".dylib")) {
                                            if let Some(base_name) = Path::new(&fname).file_name() {
                                                let dest = natives_dir.join(base_name);
                                                if !dest.exists() {
                                                    if let Ok(mut out) = File::create(&dest) {
                                                        let _ = std::io::copy(&mut zf, &mut out);
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

    pub async fn download_libraries(client: &reqwest::Client, manifest: &Value, mc_dir: &Path) {
        let Some(libs) = manifest["libraries"].as_array() else {
            return;
        };

        let os_key = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };

        let mut targets = Vec::new();

        for lib in libs {
            let lib_name_str = lib.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if !Self::is_architecture_compatible(lib_name_str) {
                continue;
            }

            if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                if let (Some(url), Some(path)) = (artifact["url"].as_str(), artifact["path"].as_str()) {
                    let path_buf = PathBuf::from(path);
                    let full_path = mc_dir.join("libraries").join(&path_buf);
                    if !full_path.exists() || full_path.metadata().map(|m| m.len() < 100).unwrap_or(true) {
                        targets.push((url.to_string(), full_path, lib_name_str.to_string(), path_buf));
                    }
                }
            }

            if let Some(native_classifier) = lib.get("natives").and_then(|n| n.get(os_key)).and_then(|s| s.as_str()) {
                if let Some(classifiers) = lib.get("downloads").and_then(|d| d.get("classifiers")) {
                    if let Some(native_artifact) = classifiers.get(native_classifier) {
                        if let (Some(url), Some(path)) = (native_artifact["url"].as_str(), native_artifact["path"].as_str()) {
                            let path_buf = PathBuf::from(path);
                            let full_path = mc_dir.join("libraries").join(&path_buf);
                            if !full_path.exists() || full_path.metadata().map(|m| m.len() < 100).unwrap_or(true) {
                                targets.push((url.to_string(), full_path, lib_name_str.to_string(), path_buf));
                            }
                        }
                    }
                }
            }

            if lib.get("downloads").is_none() && !lib_name_str.is_empty() {
                let parts: Vec<&str> = lib_name_str.split(':').collect();
                if parts.len() >= 3 {
                    let group = parts[0].replace('.', "/");
                    let artifact = parts[1];
                    let version = parts[2];
                    let classifier = if parts.len() > 3 { format!("-{}", parts[3]) } else { String::new() };
                    let jar_name = format!("{}-{}{}.jar", artifact, version, classifier);
                    let sub_path = PathBuf::from(&group).join(artifact).join(version).join(&jar_name);

                    let base = lib.get("url").and_then(|u| u.as_str()).unwrap_or("https://libraries.minecraft.net/");
                    let base_clean = base.trim_end_matches('/');
                    let formatted_sub = sub_path.to_string_lossy().replace('\\', "/");
                    let download_url = format!("{}/{}", base_clean, formatted_sub);
                    let full_path = mc_dir.join("libraries").join(&sub_path);

                    if !full_path.exists() || full_path.metadata().map(|m| m.len() < 100).unwrap_or(true) {
                        targets.push((download_url, full_path, lib_name_str.to_string(), sub_path));
                    }
                }
            }
        }

        if targets.is_empty() {
            return;
        }

        let semaphore = Arc::new(Semaphore::new(16));
        let mut tasks = Vec::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("KIPStudio/KIP_Hub/1.6.0 (contact@kip.studio)"),
        );

        let dl_client = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::limited(10))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| client.clone());

        for (url_str, full_path, lib_name, path_buf) in targets {
            let cl = dl_client.clone();
            let sem = Arc::clone(&semaphore);

            tasks.push(tokio::spawn(async move {
                let _permit = sem.acquire().await;
                if let Some(parent) = full_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                let mut downloaded = false;
                if let Ok(res) = cl.get(&url_str).send().await {
                    if res.status().is_success() {
                        if let Ok(bytes) = res.bytes().await {
                            if bytes.len() > 100 {
                                let _ = fs::write(&full_path, bytes);
                                downloaded = true;
                            }
                        }
                    }
                }

                if !downloaded {
                    let formatted_sub = path_buf.to_string_lossy().replace('\\', "/");
                    let mut fallback_urls = Vec::new();

                    let name_lower = lib_name.to_lowercase();
                    if name_lower.contains("neoforged") || name_lower.contains("neoforge") {
                        fallback_urls.push(format!("https://maven.neoforged.net/releases/{}", formatted_sub));
                    } else if name_lower.contains("minecraftforge") || name_lower.contains("forge") {
                        fallback_urls.push(format!("https://maven.minecraftforge.net/{}", formatted_sub));
                    } else if name_lower.contains("fabricmc") || name_lower.contains("fabric") {
                        fallback_urls.push(format!("https://maven.fabricmc.net/{}", formatted_sub));
                    } else if name_lower.contains("quiltmc") || name_lower.contains("quilt") {
                        fallback_urls.push(format!("https://maven.quiltmc.org/repository/release/{}", formatted_sub));
                    }

                    fallback_urls.push(format!("https://libraries.minecraft.net/{}", formatted_sub));
                    fallback_urls.push(format!("https://repo1.maven.org/maven2/{}", formatted_sub));

                    for fb_url in fallback_urls {
                        if let Ok(res) = cl.get(&fb_url).send().await {
                            if res.status().is_success() {
                                if let Ok(bytes) = res.bytes().await {
                                    if bytes.len() > 100 {
                                        let _ = fs::write(&full_path, bytes);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }));
        }

        let _ = futures_util::future::join_all(tasks).await;
    }
}