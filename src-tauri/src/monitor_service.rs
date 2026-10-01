use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, ProcessRefreshKind, System, Pid};
use crate::database::DatabaseManager;

pub struct MonitorService {
    db: Arc<DatabaseManager>,
    pub is_mc_running: Arc<AtomicBool>,
    pub console_streaming: Arc<AtomicBool>,
    pub active_game_pid: Arc<AtomicU32>,
}

impl MonitorService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            is_mc_running: Arc::new(AtomicBool::new(false)),
            console_streaming: Arc::new(AtomicBool::new(false)),
            active_game_pid: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn set_game_pid(&self, pid: u32) {
        self.active_game_pid.store(pid, Ordering::Relaxed);
    }

    pub fn start_process_monitor<F>(&self, stop_signal: Arc<AtomicBool>, current_instance: String, mut on_status_change: F)
    where
        F: FnMut(bool, &str) + Send + 'static,
    {
        let is_running_flag = Arc::clone(&self.is_mc_running);
        let active_pid_flag = Arc::clone(&self.active_game_pid);
        let db = Arc::clone(&self.db);

        std::thread::spawn(move || {
            let mut sys = System::new_all();

            while !stop_signal.load(Ordering::Relaxed) {
                sys.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::everything(),
                );

                let mut running = false;
                let tracked_pid = active_pid_flag.load(Ordering::Relaxed);

                if tracked_pid > 0 {
                    let pid_obj = Pid::from_u32(tracked_pid);
                    if let Some(p) = sys.process(pid_obj) {
                        let p_name = p.name().to_string_lossy().to_ascii_lowercase();
                        if p_name.contains("java") || p_name.contains("minecraft") {
                            running = true;
                        }
                    } else {
                        active_pid_flag.store(0, Ordering::Relaxed);
                    }
                }

                if !running {
                    for (_pid, process) in sys.processes() {
                        let name = process.name().to_string_lossy().to_ascii_lowercase();
                        if name == "javaw.exe" || name == "java.exe" || name == "javaw" || name == "java" {
                            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").to_ascii_lowercase();
                            let exe = process.exe().map(|p| p.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
                            
                            if cmd.contains("minecraft")
                                || cmd.contains("net.fabricmc")
                                || cmd.contains("fabric-loader")
                                || cmd.contains("net.minecraftforge")
                                || cmd.contains("net.neoforged")
                                || cmd.contains("quiltmc")
                                || cmd.contains("knotclient")
                                || cmd.contains("cpw.mods")
                                || cmd.contains("launchwrapper")
                                || cmd.contains("mojangtricks")
                                || exe.contains(".minecraft")
                            {
                                running = true;
                                break;
                            }
                        }
                    }
                }

                is_running_flag.store(running, Ordering::Relaxed);

                if running {
                    db.add_play_time(&current_instance, 3);
                }

                let status_text = if running { "Minecraft running" } else { "Minecraft stopped" };
                on_status_change(running, status_text);

                std::thread::sleep(Duration::from_secs(3));
            }
        });
    }

    pub fn start_console_stream<F>(&self, stop_signal: Arc<AtomicBool>, log_path_str: String, mut on_new_line: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        let streaming_flag = Arc::clone(&self.console_streaming);

        std::thread::spawn(move || {
            let log_path = Path::new(&log_path_str);
            let mut file_reader: Option<BufReader<File>> = None;
            let mut last_size: u64 = 0;

            while !stop_signal.load(Ordering::Relaxed) {
                if streaming_flag.load(Ordering::Relaxed) && log_path.exists() {
                    if file_reader.is_none() {
                        if let Ok(mut file) = File::open(log_path) {
                            if let Ok(meta) = file.metadata() {
                                last_size = meta.len();
                                let _ = file.seek(SeekFrom::End(0));
                                file_reader = Some(BufReader::new(file));
                            }
                        }
                    }

                    if let Some(reader) = file_reader.as_mut() {
                        if let Ok(meta) = fs::metadata(log_path) {
                            if meta.len() < last_size {
                                if let Ok(mut f) = File::open(log_path) {
                                    let _ = f.seek(SeekFrom::Start(0));
                                    *reader = BufReader::new(f);
                                }
                            }
                            last_size = meta.len();
                        }

                        let mut line = String::new();
                        match reader.read_line(&mut line) {
                            Ok(bytes) if bytes > 0 => {
                                on_new_line(line);
                            }
                            _ => {
                                std::thread::sleep(Duration::from_millis(100));
                            }
                        }
                    } else {
                        std::thread::sleep(Duration::from_millis(500));
                    }
                } else {
                    file_reader = None;
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        });
    }
}