use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use parking_lot::Mutex;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

pub struct ProcessSupervisor;

impl ProcessSupervisor {
    pub fn build_classpath(
        mc_dir: &Path,
        version_data: &Value,
        launch_version: &str,
        vanilla_version: &str,
        is_demo: bool,
    ) -> String {
        let mut cp_entries: Vec<String> = Vec::new();

        if let Some(libs) = version_data["libraries"].as_array() {
            for lib in libs {
                let allow = if let Some(rules) = lib["rules"].as_array() {
                    Self::is_rule_allowed(rules, is_demo)
                } else {
                    true
                };

                if allow {
                    for path in crate::launch::libraries::LibraryManager::resolve_library_paths(mc_dir, lib) {
                        let path_str = path.to_string_lossy().to_string();
                        if !cp_entries.contains(&path_str) {
                            cp_entries.push(path_str);
                        }
                    }
                }
            }
        }

        let is_neoforge = launch_version.to_lowercase().contains("neoforge")
            || version_data["mainClass"].as_str().map_or(false, |m| m.contains("neoforge"));

        let is_modern_forge = (launch_version.to_lowercase().contains("forge")
            && !launch_version.to_lowercase().contains("fabric")
            && !launch_version.to_lowercase().contains("quilt"))
            && (vanilla_version.starts_with("1.20")
                || vanilla_version.starts_with("1.21")
                || vanilla_version.starts_with("2"));

        let should_skip_main_jar = is_neoforge || is_modern_forge;

        if !should_skip_main_jar {
            let base_mc_version = version_data.get("jar")
                .and_then(|v| v.as_str())
                .or_else(|| version_data.get("inheritsFrom").and_then(|v| v.as_str()))
                .unwrap_or(vanilla_version);

            let clean_base_version = base_mc_version.split('-').next().unwrap_or(base_mc_version);
            let mut candidate_jars = Vec::new();
            candidate_jars.push(mc_dir.join("versions").join(vanilla_version).join(format!("{}.jar", vanilla_version)));
            candidate_jars.push(mc_dir.join("versions").join(clean_base_version).join(format!("{}.jar", clean_base_version)));
            candidate_jars.push(mc_dir.join("versions").join(base_mc_version).join(format!("{}.jar", base_mc_version)));
            if launch_version != clean_base_version && launch_version != vanilla_version {
                candidate_jars.push(mc_dir.join("versions").join(launch_version).join(format!("{}.jar", launch_version)));
            }

            for c_jar in candidate_jars {
                if c_jar.exists() && c_jar.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
                    let p = c_jar.to_string_lossy().to_string();
                    if !cp_entries.contains(&p) {
                        cp_entries.push(p);
                        break;
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        let sep = ";";
        #[cfg(not(target_os = "windows"))]
        let sep = ":";

        cp_entries.join(sep)
    }

    pub fn is_rule_allowed(rules: &[Value], is_demo: bool) -> bool {
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

    pub fn replace_placeholders(text: &str, vars: &HashMap<&str, &str>) -> String {
        let mut result = text.to_string();
        for (k, v) in vars {
            result = result.replace(&format!("${{{}}}", k), v);
        }
        result
    }

    pub fn filter_jvm_arg(arg: &str, java_major: u32) -> bool {
        if java_major < 24 && arg.contains("UseCompactObjectHeaders") {
            return false;
        }

        if !cfg!(target_os = "linux") && (arg.contains("CRaC") || arg.contains("CRaCCheckpoint") || arg.contains("CRaCRestore")) {
            return false;
        }

        true
    }

    pub fn execute(
        mut cmd: Command,
        app: &AppHandle,
    ) -> Result<u32, String> {
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn game process: {}", e))?;
        let child_pid = child.id();

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let recent_lines = Arc::new(Mutex::new(Vec::<String>::new()));
        let recent_lines_out = Arc::clone(&recent_lines);
        let app_out = app.clone();

        std::thread::spawn(move || {
            if let Some(out) = stdout {
                let reader = BufReader::new(out);
                for line in reader.lines().filter_map(|l| l.ok()) {
                    let _ = app_out.emit("appendConsoleLine", line.clone());
                    let mut buf = recent_lines_out.lock();
                    buf.push(line);
                    if buf.len() > 100 {
                        buf.remove(0);
                    }
                }
            }
        });

        let recent_lines_err = Arc::clone(&recent_lines);
        let app_err = app.clone();

        std::thread::spawn(move || {
            if let Some(err) = stderr {
                let reader = BufReader::new(err);
                for line in reader.lines().filter_map(|l| l.ok()) {
                    let _ = app_err.emit("appendConsoleLine", format!("[ERROR] {}", line));
                    let mut buf = recent_lines_err.lock();
                    buf.push(format!("[ERROR] {}", line));
                    if buf.len() > 100 {
                        buf.remove(0);
                    }
                }
            }
        });

        let app_exit = app.clone();
        let recent_lines_exit = Arc::clone(&recent_lines);

        std::thread::spawn(move || {
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        let logs = recent_lines_exit.lock().join("\n");
                        let snippet = format!(
                            "Process terminated with exit code: {:?}\n\nCrash Log Snippet:\n{}",
                            status.code(),
                            if logs.trim().is_empty() { "No JVM console output received." } else { &logs }
                        );
                        let _ = app_exit.emit("showCrashAlert", snippet);
                    }
                    let _ = app_exit.emit(
                        "updateDaemonStatus",
                        serde_json::json!({ "running": false, "status": "Minecraft stopped" }),
                    );
                }
                Err(_) => {
                    let _ = app_exit.emit(
                        "updateDaemonStatus",
                        serde_json::json!({ "running": false, "status": "Minecraft stopped" }),
                    );
                }
            }
        });

        Ok(child_pid)
    }
}