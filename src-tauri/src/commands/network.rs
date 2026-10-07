use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPingResultDto {
    pub online: bool,
    pub ping: Option<i64>,
    pub motd: Option<String>,
    pub players: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PteroServerDto {
    pub id: String,
    pub name: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkActionResultDto {
    pub success: bool,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyInvitePreparedDto {
    pub mods: Vec<String>,
    pub tunnel_url: String,
}

#[tauri::command]
pub async fn ping_server(ip: String) -> Result<ServerPingResultDto, AppError> {
    let trimmed = ip.trim();
    if trimmed.is_empty() {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    let host = parts[0].trim();
    if host.is_empty() {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let port = if parts.len() > 1 {
        parts[1].trim().parse::<u16>().unwrap_or(25565)
    } else {
        25565
    };

    if port == 0 {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let target = format!("{}:{}", host, port);
    let start_time = std::time::Instant::now();

    match tokio::time::timeout(std::time::Duration::from_secs(3), TcpStream::connect(&target)).await {
        Ok(Ok(mut stream)) => {
            let latency = start_time.elapsed().as_millis() as i64;
            let mut handshake = Vec::new();
            handshake.push(0x00);
            handshake.extend_from_slice(&[0xfd, 0x05]);
            let host_bytes = host.as_bytes();
            handshake.push(host_bytes.len() as u8);
            handshake.extend_from_slice(host_bytes);
            handshake.extend_from_slice(&port.to_be_bytes());
            handshake.push(0x01);

            let mut packet = Vec::new();
            packet.push(handshake.len() as u8);
            packet.extend_from_slice(&handshake);
            packet.push(0x01);
            packet.push(0x00);

            let _ = stream.write_all(&packet).await;

            let mut raw_buf = Vec::new();
            let mut temp = [0u8; 4096];
            let read_start = std::time::Instant::now();
            let mut motd = "Minecraft Server".to_string();
            let mut players = "Online".to_string();
            let mut icon: Option<String> = None;
            let mut parse_success = false;

            while read_start.elapsed() < std::time::Duration::from_secs(3) {
                if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_millis(400), stream.read(&mut temp)).await {
                    if n == 0 {
                        break;
                    }
                    raw_buf.extend_from_slice(&temp[..n]);
                    let raw_str = String::from_utf8_lossy(&raw_buf);
                    if let Some(s) = raw_str.find('{') {
                        if let Some(e) = raw_str.rfind('}') {
                            if s < e {
                                if let Ok(parsed) = serde_json::from_str::<Value>(&raw_str[s..=e]) {
                                    if let Some(desc) = parsed["description"]["text"].as_str() {
                                        motd = desc.to_string();
                                    } else if let Some(desc) = parsed["description"].as_str() {
                                        motd = desc.to_string();
                                    }
                                    let online = parsed["players"]["online"].as_i64().unwrap_or(0);
                                    let max = parsed["players"]["max"].as_i64().unwrap_or(0);
                                    players = format!("{}/{}", online, max);
                                    if let Some(fav) = parsed["favicon"].as_str() {
                                        icon = Some(fav.to_string());
                                    }
                                    parse_success = true;
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }

            if parse_success {
                Ok(ServerPingResultDto {
                    online: true,
                    ping: Some(latency),
                    motd: Some(motd),
                    players: Some(players),
                    icon,
                })
            } else {
                Ok(ServerPingResultDto {
                    online: false,
                    ping: None,
                    motd: None,
                    players: None,
                    icon: None,
                })
            }
        }
        _ => Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        }),
    }
}

#[tauri::command]
pub fn start_tunnel(state: State<'_, AppState>, app: AppHandle, port: String) -> bool {
    state.tunnel.start(&port, move |msg| {
        let _ = app.emit("updateTunnelStatus", msg);
    });
    true
}

#[tauri::command]
pub fn stop_tunnel(state: State<'_, AppState>) -> bool {
    state.tunnel.stop();
    true
}

#[tauri::command]
pub async fn deploy_docker_server(
    state: State<'_, AppState>,
    core: String,
    version: String,
    port: String,
) -> Result<NetworkActionResultDto, AppError> {
    let instance = state.instance.clone();
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        match instance.deploy_docker_server(&core, &version, &port, &cfg.current_instance) {
            Ok(msg) => Ok(NetworkActionResultDto { success: true, msg }),
            Err(e) => Ok(NetworkActionResultDto { success: false, msg: e }),
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn ptero_connect(
    state: State<'_, AppState>,
    url: String,
    key: String,
) -> Result<Vec<PteroServerDto>, AppError> {
    let clean_url = url.trim().trim_end_matches('/').to_string();
    if clean_url.is_empty() || key.trim().is_empty() {
        return Err(AppError::Config("Panel URL and API key cannot be empty.".to_string()));
    }

    let mut cfg = config::load_app_config();
    cfg.ptero_url = clean_url.clone();
    config::set_secret("ptero_key", key.trim());
    config::save_app_config(&cfg);

    let raw_servers = state
        .api
        .get_ptero_server_status(&clean_url, key.trim())
        .await
        .map_err(|e| AppError::Config(e))?;

    let mut result = Vec::new();
    if let Some(arr) = raw_servers.as_array() {
        for s in arr {
            result.push(PteroServerDto {
                id: s["id"].as_str().unwrap_or("").to_string(),
                name: s["name"].as_str().unwrap_or("Server").to_string(),
                state: s["state"].as_str().unwrap_or("offline").to_string(),
            });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn ptero_action(
    state: State<'_, AppState>,
    action: String,
    server_id: String,
) -> Result<NetworkActionResultDto, AppError> {
    let cfg = config::load_app_config();
    let key = config::get_secret("ptero_key");
    let ok = state
        .api
        .send_ptero_power_action(&cfg.ptero_url, &server_id, &action, &key)
        .await
        .map_err(|e| AppError::Config(e))?;

    Ok(NetworkActionResultDto {
        success: ok,
        msg: if ok {
            format!("Signal '{}' dispatched to server.", action)
        } else {
            "Action rejected by remote daemon.".to_string()
        },
    })
}

#[tauri::command]
pub fn party_invite_prepare(_state: State<'_, AppState>) -> Result<PartyInvitePreparedDto, AppError> {
    let cfg = config::load_app_config();
    let mods_dir = Path::new(&cfg.current_instance).join("mods");
    let mut mods = Vec::new();

    if let Ok(entries) = fs::read_dir(mods_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                mods.push(path.file_stem().unwrap_or_default().to_string_lossy().to_string());
            }
        }
    }

    Ok(PartyInvitePreparedDto {
        mods,
        tunnel_url: "tcp://a.pinggy.io:443".to_string(),
    })
}