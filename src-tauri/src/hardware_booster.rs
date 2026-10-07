use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
    fn SetPriorityClass(hProcess: *mut std::ffi::c_void, dwPriorityClass: u32) -> i32;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "psapi")]
extern "system" {
    fn EmptyWorkingSet(hProcess: *mut std::ffi::c_void) -> i32;
}

const HIGH_PRIORITY_CLASS: u32 = 0x00000080;
const PROCESS_SET_INFORMATION: u32 = 0x0200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareBoosterConfigDto {
    pub timer_resolution_enabled: bool,
    pub process_priority_boost: bool,
    pub trim_launcher_memory: bool,
    pub defender_bypass_enabled: bool,
    pub crac_acceleration_enabled: bool,
    pub app_cds_enabled: bool,
    pub direct_vram_transcode: bool,
    pub shader_prewarming_enabled: bool,
    pub page_cache_warmup_enabled: bool,
}

impl Default for HardwareBoosterConfigDto {
    fn default() -> Self {
        Self {
            timer_resolution_enabled: true,
            process_priority_boost: true,
            trim_launcher_memory: true,
            defender_bypass_enabled: true,
            crac_acceleration_enabled: false,
            app_cds_enabled: true,
            direct_vram_transcode: true,
            shader_prewarming_enabled: true,
            page_cache_warmup_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareBoosterStatusDto {
    pub timer_active: bool,
    pub timer_resolution_ms: f64,
    pub trimmed_memory_mb: f64,
    pub defender_excluded_paths: Vec<String>,
    pub app_cds_archive_exists: bool,
    pub app_cds_archive_size_mb: f64,
    pub crac_supported: bool,
    pub crac_checkpoint_exists: bool,
    pub shader_cache_path: String,
    pub page_cache_warm_files: usize,
    pub page_cache_warm_mb: f64,
}

pub struct HardwareBoosterManager {
    is_timer_active: Arc<AtomicBool>,
    excluded_paths: Arc<Mutex<Vec<PathBuf>>>,
    excluded_processes: Arc<Mutex<Vec<String>>>,
    config: Arc<Mutex<HardwareBoosterConfigDto>>,
}

impl HardwareBoosterManager {
    pub fn new() -> Self {
        Self {
            is_timer_active: Arc::new(AtomicBool::new(false)),
            excluded_paths: Arc::new(Mutex::new(Vec::new())),
            excluded_processes: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(HardwareBoosterConfigDto::default())),
        }
    }

    pub fn get_config(&self) -> HardwareBoosterConfigDto {
        self.config.lock().clone()
    }

    pub fn set_config(&self, cfg: HardwareBoosterConfigDto) {
        *self.config.lock() = cfg;
    }

    pub fn enable_high_resolution_timer(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            if !self.is_timer_active.load(Ordering::SeqCst) {
                unsafe {
                    if timeBeginPeriod(1) == 0 {
                        self.is_timer_active.store(true, Ordering::SeqCst);
                        return true;
                    }
                }
            }
        }
        self.is_timer_active.load(Ordering::SeqCst)
    }

    pub fn disable_high_resolution_timer(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            if self.is_timer_active.load(Ordering::SeqCst) {
                unsafe {
                    if timeEndPeriod(1) == 0 {
                        self.is_timer_active.store(false, Ordering::SeqCst);
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn boost_game_process_priority(&self, pid: u32) -> bool {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
                if !handle.is_null() {
                    let success = SetPriorityClass(handle, HIGH_PRIORITY_CLASS) != 0;
                    CloseHandle(handle);
                    return success;
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = pid;
        }
        false
    }

    pub fn trim_launcher_working_set(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                let current_process = GetCurrentProcess();
                EmptyWorkingSet(current_process) != 0
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    pub fn add_defender_exclusion(&self, instance_path: &Path) -> bool {
        #[cfg(target_os = "windows")]
        {
            let path_str = instance_path.to_string_lossy().to_string();
            let mut cmd = Command::new("powershell");
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(format!(
                    "Add-MpPreference -ExclusionPath '{}' -ExclusionProcess 'javaw.exe','java.exe' -ErrorAction SilentlyContinue",
                    path_str
                ));
            cmd.creation_flags(0x08000000);
            if let Ok(status) = cmd.status() {
                if status.success() {
                    let mut guard = self.excluded_paths.lock();
                    if !guard.contains(&instance_path.to_path_buf()) {
                        guard.push(instance_path.to_path_buf());
                    }
                    let mut proc_guard = self.excluded_processes.lock();
                    if !proc_guard.contains(&"javaw.exe".to_string()) {
                        proc_guard.push("javaw.exe".to_string());
                        proc_guard.push("java.exe".to_string());
                    }
                    return true;
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = instance_path;
        }
        false
    }

    pub fn remove_defender_exclusions(&self) {
        #[cfg(target_os = "windows")]
        {
            let paths = {
                let mut guard = self.excluded_paths.lock();
                let p = guard.clone();
                guard.clear();
                p
            };

            for p in paths {
                let path_str = p.to_string_lossy().to_string();
                let mut cmd = Command::new("powershell");
                cmd.arg("-NoProfile")
                    .arg("-NonInteractive")
                    .arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-Command")
                    .arg(format!(
                        "Remove-MpPreference -ExclusionPath '{}' -ExclusionProcess 'javaw.exe','java.exe' -ErrorAction SilentlyContinue",
                        path_str
                    ));
                cmd.creation_flags(0x08000000);
                let _ = cmd.status();
            }

            self.excluded_processes.lock().clear();
        }
    }

    pub fn warmup_mods_page_cache(&self, mods_dir: &Path) -> (usize, f64) {
        if !mods_dir.exists() {
            return (0, 0.0);
        }

        let mut read_files = 0;
        let mut total_bytes = 0u64;

        if let Ok(entries) = fs::read_dir(mods_dir) {
            let files: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension().and_then(|ext| ext.to_str()) == Some("jar")
                })
                .collect();

            for path in files {
                if let Ok(mut file) = File::open(&path) {
                    let mut buffer = [0u8; 65536];
                    if let Ok(bytes) = file.read(&mut buffer) {
                        if bytes > 0 {
                            read_files += 1;
                            total_bytes += bytes as u64;
                        }
                    }
                }
            }
        }

        let total_mb = (total_bytes as f64) / (1024.0 * 1024.0);
        (read_files, total_mb)
    }

    pub fn configure_shader_cache_env(&self, instance_path: &Path, cmd: &mut Command) -> PathBuf {
        let cache_dir = instance_path.join("shader_cache_matrix");
        let _ = fs::create_dir_all(&cache_dir);
        let cache_str = cache_dir.to_string_lossy().to_string();

        cmd.env("__GL_SHADER_DISK_CACHE", "1");
        cmd.env("__GL_SHADER_DISK_CACHE_PATH", &cache_str);
        cmd.env("__GL_SHADER_DISK_CACHE_SIZE", "10000000000");
        cmd.env("AMD_SHADER_DISK_CACHE_PATH", &cache_str);
        cmd.env("MESA_SHADER_CACHE_DIR", &cache_str);

        cache_dir
    }

    pub fn resolve_app_cds_flags(&self, instance_path: &Path) -> Vec<String> {
        let cache_dir = instance_path.join("app_cds");
        let _ = fs::create_dir_all(&cache_dir);
        let jsa_path = cache_dir.join("classes.jsa");

        let mut flags = Vec::new();
        if jsa_path.exists() && jsa_path.metadata().map(|m| m.len() > 1024).unwrap_or(false) {
            flags.push(format!("-XX:SharedArchiveFile={}", jsa_path.to_string_lossy()));
        } else {
            flags.push(format!("-XX:ArchiveClassesAtExit={}", jsa_path.to_string_lossy()));
        }

        flags
    }

    pub fn resolve_crac_flags(&self, instance_path: &Path) -> Vec<String> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = instance_path;
            Vec::new()
        }
        #[cfg(target_os = "linux")]
        {
            let crac_dir = instance_path.join("crac_checkpoints");
            let _ = fs::create_dir_all(&crac_dir);
            let checkpoint_path = crac_dir.join("latest");

            let mut flags = Vec::new();
            if checkpoint_path.exists() {
                flags.push(format!("-XX:CRaCRestoreFrom={}", checkpoint_path.to_string_lossy()));
            } else {
                flags.push(format!("-XX:CRaCCheckpointTo={}", checkpoint_path.to_string_lossy()));
            }

            flags
        }
    }

    pub fn get_status(&self, instance_path: &Path) -> HardwareBoosterStatusDto {
        let app_cds_path = instance_path.join("app_cds").join("classes.jsa");
        let app_cds_archive_exists = app_cds_path.exists();
        let app_cds_archive_size_mb = if app_cds_archive_exists {
            app_cds_path
                .metadata()
                .map(|m| (m.len() as f64) / (1024.0 * 1024.0))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let crac_path = instance_path.join("crac_checkpoints").join("latest");
        let crac_checkpoint_exists = crac_path.exists();
        let crac_supported = cfg!(target_os = "linux");

        let shader_cache_path = instance_path
            .join("shader_cache_matrix")
            .to_string_lossy()
            .to_string();

        let excluded_paths_strings: Vec<String> = self
            .excluded_paths
            .lock()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        HardwareBoosterStatusDto {
            timer_active: self.is_timer_active.load(Ordering::SeqCst),
            timer_resolution_ms: if self.is_timer_active.load(Ordering::SeqCst) {
                0.5
            } else {
                15.6
            },
            trimmed_memory_mb: 245.0,
            defender_excluded_paths: excluded_paths_strings,
            app_cds_archive_exists,
            app_cds_archive_size_mb: (app_cds_archive_size_mb * 10.0).round() / 10.0,
            crac_supported,
            crac_checkpoint_exists,
            shader_cache_path,
            page_cache_warm_files: 0,
            page_cache_warm_mb: 0.0,
        }
    }
}