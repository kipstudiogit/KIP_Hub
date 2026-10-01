use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use discord_rich_presence::activity::{Activity, Assets};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use crate::config;
use crate::database::DatabaseManager;

pub struct RpcService;

impl RpcService {
    pub fn start_rpc(
        client_id: &str,
        db: Arc<DatabaseManager>,
        is_mc_running: Arc<AtomicBool>,
        stop_signal: Arc<AtomicBool>,
    ) {
        let cid = client_id.to_string();

        std::thread::spawn(move || {
            let mut client = DiscordIpcClient::new(&cid);
            let mut connected = false;

            while !stop_signal.load(Ordering::Relaxed) {
                let app_config = config::load_app_config();

                if !app_config.rpc {
                    if connected {
                        let _ = client.close();
                        connected = false;
                    }
                    std::thread::sleep(Duration::from_secs(5));
                    continue;
                }

                if !connected {
                    if client.connect().is_ok() {
                        connected = true;
                    } else {
                        std::thread::sleep(Duration::from_secs(15));
                        continue;
                    }
                }

                let running = is_mc_running.load(Ordering::Relaxed);
                let state = if running { "Playing Minecraft" } else { "In Hub" };
                let pt = db.get_play_time(&app_config.current_instance);

                let details = if pt > 0 {
                    format!("Playtime: {}h {}m", pt / 3600, (pt % 3600) / 60)
                } else {
                    format!("Protected by {}", config::APP_NAME)
                };

                let large_text = format!("{} {}", config::APP_NAME, config::APP_VERSION);
                let assets = Assets::new().large_image("logo").large_text(&large_text);
                let activity = Activity::new()
                    .state(state)
                    .details(&details)
                    .assets(assets);

                if client.set_activity(activity).is_err() {
                    let _ = client.close();
                    connected = false;
                }

                std::thread::sleep(Duration::from_secs(15));
            }

            if connected {
                let _ = client.close();
            }
        });
    }
}