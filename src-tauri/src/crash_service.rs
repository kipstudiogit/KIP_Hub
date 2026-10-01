use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub struct CrashService;

impl CrashService {
    pub fn start_crash_monitor<F>(crash_dir_str: String, stop_signal: Arc<AtomicBool>, mut on_crash: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        std::thread::spawn(move || {
            let crash_path = Path::new(&crash_dir_str);
            let _ = std::fs::create_dir_all(crash_path);

            let (tx, rx) = channel();
            let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
                Ok(w) => w,
                Err(_) => return,
            };

            if watcher.watch(crash_path, RecursiveMode::NonRecursive).is_err() {
                return;
            }

            while !stop_signal.load(Ordering::Relaxed) {
                if let Ok(Ok(event)) = rx.recv_timeout(Duration::from_millis(500)) {
                    if let EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Some(ext) = path.extension() {
                                if ext == "txt" {
                                    std::thread::sleep(Duration::from_secs(2));
                                    if let Ok(file) = File::open(&path) {
                                        let mut buffer = String::new();
                                        let mut handle = file.take(4000);
                                        if handle.read_to_string(&mut buffer).is_ok() {
                                            on_crash(buffer);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}