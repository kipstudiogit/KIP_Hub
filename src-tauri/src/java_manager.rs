use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;
use crate::config;

pub struct JavaManager {
    client: reqwest::Client,
}

impl JavaManager {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("KIPStudio/KIP_Hub/1.6.0 (contact@kip.studio)"),
        );

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .redirect(reqwest::redirect::Policy::limited(10))
                .connect_timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn parse_mc_version(version: &str) -> (u32, u32, u32) {
        let clean = version.split('-').next().unwrap_or(version);
        let parts: Vec<u32> = clean.split('.').filter_map(|s| s.parse::<u32>().ok()).collect();
        let major = parts.get(0).copied().unwrap_or(1);
        let minor = parts.get(1).copied().unwrap_or(0);
        let patch = parts.get(2).copied().unwrap_or(0);
        (major, minor, patch)
    }

    pub fn get_required_java_major(&self, loader: &str, version: &str, version_data: Option<&Value>) -> u32 {
        let (major, minor, patch) = Self::parse_mc_version(version);

        if major >= 26 {
            return 25;
        }

        let norm_loader = loader.to_lowercase();

        if let Some(data) = version_data {
            if let Some(mv) = data.get("javaVersion").and_then(|j| j.get("majorVersion")).and_then(|v| v.as_u64()) {
                let m = mv as u32;
                if norm_loader == "forge" {
                    if minor <= 16 {
                        return 8;
                    }
                    if minor <= 20 && patch < 5 {
                        return 17;
                    }
                }
                if norm_loader == "neoforge" {
                    if minor == 20 && patch <= 4 {
                        return 17;
                    }
                }
                if m == 16 {
                    return 17;
                }
                return m;
            }
        }

        if norm_loader == "forge" {
            if minor <= 16 {
                return 8;
            }
            if minor <= 20 && patch < 5 {
                return 17;
            }
            return 21;
        }

        if norm_loader == "neoforge" {
            if minor == 20 && patch <= 4 {
                return 17;
            }
            return 21;
        }

        if minor <= 16 {
            8
        } else if minor <= 20 && patch < 5 {
            17
        } else {
            21
        }
    }

    pub fn check_java_binary(&self, path: &str) -> Option<u32> {
        let probe_path = if path.ends_with("javaw.exe") {
            path.replace("javaw.exe", "java.exe")
        } else if path.ends_with("javaw") {
            path.replace("javaw", "java")
        } else {
            path.to_string()
        };

        let mut cmd = Command::new(&probe_path);
        cmd.arg("-version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        let output = cmd.output().ok()?;
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

    pub fn find_compatible_java_binary(&self, mc_dir: &Path, required_major: u32) -> Option<String> {
        let target_major = required_major;
        let mut candidates = Vec::new();

        if let Ok(java_home) = std::env::var("JAVA_HOME") {
            let jh_path = Path::new(&java_home);
            #[cfg(target_os = "windows")]
            {
                candidates.push(jh_path.join("bin").join("javaw.exe"));
                candidates.push(jh_path.join("bin").join("java.exe"));
            }
            #[cfg(not(target_os = "windows"))]
            {
                candidates.push(jh_path.join("bin").join("java"));
            }
        }

        let isolated_runtime_dir = mc_dir.join("runtime").join(format!("kip-java-{}", target_major));
        if isolated_runtime_dir.exists() {
            #[cfg(target_os = "windows")]
            {
                candidates.push(isolated_runtime_dir.join("bin").join("javaw.exe"));
                candidates.push(isolated_runtime_dir.join("bin").join("java.exe"));
            }
            #[cfg(not(target_os = "windows"))]
            {
                candidates.push(isolated_runtime_dir.join("bin").join("java"));
            }

            if let Ok(entries) = fs::read_dir(&isolated_runtime_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        #[cfg(target_os = "windows")]
                        {
                            candidates.push(entry.path().join("bin").join("javaw.exe"));
                            candidates.push(entry.path().join("bin").join("java.exe"));
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            candidates.push(entry.path().join("bin").join("java"));
                        }
                    }
                }
            }
        }

        let runtime_dir = mc_dir.join("runtime");
        if runtime_dir.exists() {
            let win_runtimes = [
                "java-runtime-epsilon/windows-x64/java-runtime-epsilon/bin/javaw.exe",
                "java-runtime-delta/windows-x64/java-runtime-delta/bin/javaw.exe",
                "java-runtime-gamma/windows-x64/java-runtime-gamma/bin/javaw.exe",
                "java-runtime-beta/windows-x64/java-runtime-beta/bin/javaw.exe",
                "jre-legacy/windows-x64/jre-legacy/bin/javaw.exe",
            ];
            for r in win_runtimes {
                candidates.push(runtime_dir.join(r));
            }

            let unix_runtimes = [
                "java-runtime-epsilon/bin/java",
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
                                    "java-runtime-epsilon/windows-x64/java-runtime-epsilon/bin/javaw.exe",
                                    "java-runtime-delta/windows-x64/java-runtime-delta/bin/javaw.exe",
                                    "java-runtime-gamma/windows-x64/java-runtime-gamma/bin/javaw.exe",
                                    "java-runtime-beta/windows-x64/java-runtime-beta/bin/javaw.exe",
                                    "jre-legacy/windows-x64/jre-legacy/bin/javaw.exe",
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
                "C:\\Program Files\\Eclipse Adoptium",
                "C:\\Program Files\\Microsoft",
                "C:\\Program Files\\Java",
                "C:\\Program Files\\BellSoft",
                "C:\\Program Files\\Amazon Corretto",
                "C:\\Program Files\\Zulu",
            ];

            for d in win_dirs {
                let p = Path::new(d);
                if p.is_dir() {
                    if let Ok(entries) = fs::read_dir(p) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let sub_javaw = entry.path().join("bin").join("javaw.exe");
                            if sub_javaw.exists() {
                                candidates.push(sub_javaw);
                            }
                            let sub_java = entry.path().join("bin").join("java.exe");
                            if sub_java.exists() {
                                candidates.push(sub_java);
                            }
                        }
                    }
                }
            }
        }

        for c in candidates {
            if c.exists() {
                let path_str = c.to_string_lossy().to_string();
                if let Some(major) = self.check_java_binary(&path_str) {
                    if major == target_major {
                        #[cfg(target_os = "windows")]
                        {
                            let javaw = c.with_file_name("javaw.exe");
                            if javaw.exists() {
                                return Some(javaw.to_string_lossy().to_string());
                            }
                        }
                        return Some(path_str);
                    }
                }
            }
        }

        None
    }

    pub async fn ensure_java_runtime(&self, mc_dir: &Path, required_major: u32, app: &AppHandle) -> Result<String, String> {
        let target_major = required_major;

        if let Some(existing) = self.find_compatible_java_binary(mc_dir, target_major) {
            return Ok(existing);
        }

        let target_dir = mc_dir.join("runtime").join(format!("kip-java-{}", target_major));
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        let os_str = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };

        let arch_str = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x64"
        };

        let download_url = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
            target_major, os_str, arch_str
        );

        let mut resp = self.client.get(&download_url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("OpenJDK provision server returned status {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(180 * 1024 * 1024);
        let mut downloaded: u64 = 0;
        let temp_archive_path = target_dir.join("runtime_archive.zip");
        let mut out_file = File::create(&temp_archive_path).map_err(|e| e.to_string())?;

        while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
            out_file.write_all(&chunk).map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;
            let mb = (downloaded as f64) / (1024.0 * 1024.0);
            let pct = ((downloaded as f64 / total_size as f64) * 45.0) as u32 + 15;
            let _ = app.emit("updateLaunchStatus", format!("Acquiring OpenJDK {} ({:.1} MB)...", target_major, mb));
            let _ = app.emit("updateLaunchProgress", pct);
        }
        drop(out_file);

        let _ = app.emit("updateLaunchStatus", "Decompressing isolated Java environment...");
        let _ = app.emit("updateLaunchProgress", 65);

        let archive_file = File::open(&temp_archive_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(archive_file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let raw_path = file.mangled_name();
            let mut components = raw_path.components();
            components.next();
            let relative_path = components.as_path();

            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let out_path = target_dir.join(relative_path);
            if file.is_dir() {
                let _ = fs::create_dir_all(&out_path);
            } else {
                if let Some(parent) = out_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(mut outfile) = File::create(&out_path) {
                    let _ = std::io::copy(&mut file, &mut outfile);
                }
            }
        }

        let _ = fs::remove_file(&temp_archive_path);

        if let Some(resolved) = self.find_compatible_java_binary(mc_dir, target_major) {
            Ok(resolved)
        } else {
            Err("Provisioned Java executable not found after decompression.".to_string())
        }
    }

    pub fn resolve_compatible_java(&self, mc_dir: &Path, loader: &str, version: &str, version_data: &Value) -> Result<String, String> {
        let app_config = config::load_app_config();
        if !app_config.custom_java_path.is_empty() && Path::new(&app_config.custom_java_path).exists() {
            return Ok(app_config.custom_java_path);
        }

        let required_major = self.get_required_java_major(loader, version, Some(version_data));
        let system_major = self.check_java_binary("java").unwrap_or(0);

        if system_major == required_major {
            return Ok("java".to_string());
        }

        if let Some(found_java) = self.find_compatible_java_binary(mc_dir, required_major) {
            return Ok(found_java);
        }

        Err(format!("No compatible Java {} environment detected.", required_major))
    }
}