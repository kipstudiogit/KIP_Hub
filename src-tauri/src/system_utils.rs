use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use parking_lot::Mutex;
use sysinfo::{ProcessesToUpdate, System};
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const DETACHED_PROCESS: u32 = 0x00000008;

static CACHED_JAVA_VERSION: Mutex<Option<String>> = Mutex::new(None);

pub struct SystemUtils;

impl SystemUtils {
    pub fn silent_command(program: &str) -> Command {
        let mut cmd = Command::new(program);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
        }
        cmd
    }

    pub fn optimize_fps(mc_dir: &str) -> i32 {
        let options_path = Path::new(mc_dir).join("options.txt");
        if !options_path.exists() {
            return -1;
        }

        let mut sys = System::new_all();
        sys.refresh_memory();
        let ram_gb = sys.total_memory() / (1024 * 1024 * 1024);

        let mut target_opts = std::collections::HashMap::new();
        if ram_gb >= 12 {
            target_opts.insert("renderDistance", "16");
            target_opts.insert("simulationDistance", "10");
            target_opts.insert("graphicsMode", "2");
            target_opts.insert("entityShadows", "true");
            target_opts.insert("particles", "0");
        } else if ram_gb >= 6 {
            target_opts.insert("renderDistance", "12");
            target_opts.insert("simulationDistance", "8");
            target_opts.insert("graphicsMode", "1");
            target_opts.insert("entityShadows", "true");
            target_opts.insert("particles", "1");
        } else {
            target_opts.insert("renderDistance", "8");
            target_opts.insert("simulationDistance", "5");
            target_opts.insert("graphicsMode", "0");
            target_opts.insert("entityShadows", "false");
            target_opts.insert("particles", "2");
        }

        let mut applied = 0;
        let mut handled_keys = std::collections::HashSet::new();
        let mut new_lines = Vec::new();

        if let Ok(content) = fs::read_to_string(&options_path) {
            for line in content.lines() {
                let s = line.trim();
                if let Some((k, v)) = s.split_once(':') {
                    if let Some(&new_v) = target_opts.get(k) {
                        handled_keys.insert(k.to_string());
                        if v != new_v {
                            new_lines.push(format!("{}:{}", k, new_v));
                            applied += 1;
                        } else {
                            new_lines.push(line.to_string());
                        }
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    new_lines.push(line.to_string());
                }
            }

            for (k, v) in &target_opts {
                if !handled_keys.contains(*k) {
                    new_lines.push(format!("{}:{}", k, v));
                    applied += 1;
                }
            }
        } else {
            return -1;
        }

        let tmp_path = options_path.with_extension("tmp");
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(&tmp_path) {
            for line in new_lines {
                let _ = writeln!(file, "{}", line);
            }
            let _ = file.sync_all();
            drop(file);

            let _ = fs::remove_file(&options_path);
            if fs::rename(&tmp_path, &options_path).is_ok() {
                applied
            } else {
                -1
            }
        } else {
            -1
        }
    }

    pub fn kill_zombie_processes() -> i32 {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let mut killed_count = 0;

        for (_pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_ascii_lowercase();
            if name == "java.exe" || name == "javaw.exe" || name == "java" {
                let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").to_ascii_lowercase();
                if cmd.contains("minecraft") || cmd.contains("net.fabricmc") || cmd.contains("net.minecraftforge") {
                    if process.kill() {
                        killed_count += 1;
                    }
                }
            }
        }
        killed_count
    }

    pub fn flush_dns_cache() {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Self::silent_command("ipconfig");
            cmd.arg("/flushdns");
            let _ = cmd.output();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("dscacheutil").arg("-flushcache").output();
            let _ = Command::new("killall").arg("-HUP").arg("mDNSResponder").output();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("resolvectl").arg("flush-caches").output();
        }
    }

    pub fn get_java_version() -> String {
        if let Some(ref ver) = *CACHED_JAVA_VERSION.lock() {
            return ver.clone();
        }

        let mut cmd = Self::silent_command("java");
        cmd.arg("-version");

        if let Ok(output) = cmd.output() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
            if stderr.contains("version") {
                let parts: Vec<&str> = stderr.split('"').collect();
                if parts.len() > 1 {
                    let parsed = parts[1].to_string();
                    *CACHED_JAVA_VERSION.lock() = Some(parsed.clone());
                    return parsed;
                }
            }
        }

        let fallback = "8.0.0".to_string();
        *CACHED_JAVA_VERSION.lock() = Some(fallback.clone());
        fallback
    }

    pub fn generate_jvm_args_for_ram(ram_allocation: i32) -> (i32, String) {
        let target_gb = if ram_allocation > 0 {
            ram_allocation
        } else {
            let mut sys = System::new_all();
            sys.refresh_memory();
            let total_gb = (sys.total_memory() / (1024 * 1024 * 1024)) as i32;
            if total_gb >= 16 {
                6
            } else if total_gb >= 8 {
                4
            } else {
                2
            }
        };

        let args = format!("-Xmx{}G -Xms256M -XX:+UseG1GC", target_gb);
        (target_gb, args)
    }

    pub fn generate_jvm_args() -> (i32, String) {
        Self::generate_jvm_args_for_ram(0)
    }

    pub fn get_dir_size_mb(path: &str) -> f64 {
        let mut total_size = 0;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
        (total_size as f64) / (1024.0 * 1024.0)
    }

    pub fn get_saves_count(saves_dir: &str) -> i32 {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(saves_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}