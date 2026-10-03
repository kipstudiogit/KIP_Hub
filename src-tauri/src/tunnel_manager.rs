use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use regex::Regex;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct TunnelManager {
    child_process: Arc<Mutex<Option<Child>>>,
    pub running: Arc<AtomicBool>,
    pub active_endpoint: Arc<Mutex<Option<String>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            child_process: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            active_endpoint: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start<F>(&self, port: &str, mut callback: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        self.stop();

        let child_arc = Arc::clone(&self.child_process);
        let running_flag = Arc::clone(&self.running);
        let endpoint_arc = Arc::clone(&self.active_endpoint);
        let port_owned = port.to_string();

        running_flag.store(true, Ordering::Relaxed);

        std::thread::spawn(move || {
            let mut cmd = Command::new("ssh");
            cmd.arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg("-o")
                .arg("ServerAliveInterval=30")
                .arg("-p")
                .arg("443")
                .arg(format!("-R0:localhost:{}", port_owned))
                .arg("tcp@a.pinggy.io")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            #[cfg(target_os = "windows")]
            {
                cmd.creation_flags(0x08000000);
            }

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    running_flag.store(false, Ordering::Relaxed);
                    *endpoint_arc.lock() = None;
                    callback(format!("Error: {}", e));
                    return;
                }
            };

            let stdout = child.stdout.take();
            *child_arc.lock() = Some(child);

            let re = Regex::new(r"(?:tcp://|Forwarding TCP connections from )([a-zA-Z0-9\.\-]+:\d+)").unwrap();

            if let Some(out) = stdout {
                let reader = BufReader::new(out);
                for line_res in reader.lines() {
                    if !running_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Ok(line) = line_res {
                        if let Some(caps) = re.captures(&line) {
                            if let Some(matched) = caps.get(1) {
                                let ep = matched.as_str().to_string();
                                *endpoint_arc.lock() = Some(ep.clone());
                                callback(ep);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        *self.active_endpoint.lock() = None;
        let mut child_guard = self.child_process.lock();
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}