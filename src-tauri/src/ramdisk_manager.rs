use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use parking_lot::Mutex;
use sysinfo::System;
use walkdir::WalkDir;

pub struct RamDiskManager {
    is_active: Arc<AtomicBool>,
    mount_path: Arc<Mutex<Option<PathBuf>>>,
    physical_path: Arc<Mutex<Option<PathBuf>>>,
    allocated_bytes: Arc<AtomicU64>,
}

impl RamDiskManager {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            mount_path: Arc::new(Mutex::new(None)),
            physical_path: Arc::new(Mutex::new(None)),
            allocated_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_available_system_ram_mb() -> u64 {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.available_memory() / (1024 * 1024)
    }

    fn resolve_ram_path(instance_name: &str) -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            let shm = PathBuf::from("/dev/shm");
            if shm.exists() {
                return shm.join(format!("kip_ram_{}", instance_name));
            }
        }

        #[cfg(target_os = "macos")]
        {
            let tmp = PathBuf::from("/tmp");
            return tmp.join(format!("kip_ram_{}", instance_name));
        }

        let base = std::env::temp_dir();
        base.join("KIP_RAM_STAGING").join(instance_name)
    }

    pub fn mount_and_stage(&self, physical_instance_dir: &str) -> Result<(PathBuf, u64, f64), String> {
        let phys_path = PathBuf::from(physical_instance_dir);
        if !phys_path.exists() {
            return Err("Instance directory does not exist.".to_string());
        }

        let instance_name = phys_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let ram_path = Self::resolve_ram_path(&instance_name);
        fs::create_dir_all(&ram_path).map_err(|e| e.to_string())?;

        let start_time = std::time::Instant::now();
        let mut copied_bytes: u64 = 0;

        let folders_to_stage = ["mods", "config", "defaultconfigs", "kubejs"];
        for folder in &folders_to_stage {
            let src_folder = phys_path.join(folder);
            let dst_folder = ram_path.join(folder);

            if src_folder.exists() {
                fs::create_dir_all(&dst_folder).map_err(|e| e.to_string())?;
                for entry in WalkDir::new(&src_folder).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Ok(rel) = path.strip_prefix(&src_folder) {
                        let target = dst_folder.join(rel);
                        if path.is_dir() {
                            let _ = fs::create_dir_all(&target);
                        } else if path.is_file() {
                            if let Ok(meta) = path.metadata() {
                                copied_bytes += meta.len();
                            }
                            let _ = fs::copy(path, &target);
                        }
                    }
                }
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        let speed_mb_s = ((copied_bytes as f64 / (1024.0 * 1024.0)) / elapsed * 10.0).round() / 10.0;

        self.is_active.store(true, Ordering::SeqCst);
        self.allocated_bytes.store(copied_bytes, Ordering::SeqCst);
        *self.mount_path.lock() = Some(ram_path.clone());
        *self.physical_path.lock() = Some(phys_path);

        Ok((ram_path, copied_bytes / (1024 * 1024), speed_mb_s))
    }

    pub fn sync_back_to_disk(&self) -> Result<(u64, usize), String> {
        let ram_guard = self.mount_path.lock();
        let phys_guard = self.physical_path.lock();

        let (ram_path, phys_path) = match (ram_guard.as_ref(), phys_guard.as_ref()) {
            (Some(r), Some(p)) => (r, p),
            _ => return Ok((0, 0)),
        };

        let mut synced_bytes: u64 = 0;
        let mut files_updated: usize = 0;
        let folders_to_sync = ["config", "defaultconfigs", "kubejs", "options.txt"];

        for item in &folders_to_sync {
            let src = ram_path.join(item);
            let dst = phys_path.join(item);

            if src.is_file() {
                if Self::should_sync_file(&src, &dst) {
                    if let Ok(meta) = src.metadata() {
                        synced_bytes += meta.len();
                        files_updated += 1;
                    }
                    let _ = fs::copy(&src, &dst);
                }
            } else if src.is_dir() {
                fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
                for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Ok(rel) = path.strip_prefix(&src) {
                        let target = dst.join(rel);
                        if path.is_dir() {
                            let _ = fs::create_dir_all(&target);
                        } else if path.is_file() && Self::should_sync_file(path, &target) {
                            if let Ok(meta) = path.metadata() {
                                synced_bytes += meta.len();
                                files_updated += 1;
                            }
                            let _ = fs::copy(path, &target);
                        }
                    }
                }
            }
        }

        Ok((synced_bytes, files_updated))
    }

    fn should_sync_file(src: &Path, dst: &Path) -> bool {
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext == "jar" && dst.exists() {
            return false;
        }

        if !dst.exists() {
            return true;
        }

        let src_mod = src.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        let dst_mod = dst.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);

        src_mod > dst_mod
    }

    pub fn unmount(&self) -> Result<(), String> {
        let _ = self.sync_back_to_disk();

        let mut ram_guard = self.mount_path.lock();
        if let Some(ref path) = *ram_guard {
            if path.exists() {
                let _ = fs::remove_dir_all(path);
            }
        }

        *ram_guard = None;
        *self.physical_path.lock() = None;
        self.is_active.store(false, Ordering::SeqCst);
        self.allocated_bytes.store(0, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_ramdisk_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    pub fn get_mount_path(&self) -> Option<PathBuf> {
        self.mount_path.lock().clone()
    }
}