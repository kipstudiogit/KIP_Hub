use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredLanServerDto {
    pub motd: String,
    pub port: u16,
    pub address: String,
    pub last_seen_epoch: u64,
}

static LAN_LISTENER_ACTIVE: AtomicBool = AtomicBool::new(false);
static DISCOVERED_LAN_SERVERS: Mutex<Option<HashMap<String, DiscoveredLanServerDto>>> = Mutex::new(None);

fn read_varint<R: std::io::Read>(reader: &mut R) -> Result<i32, std::io::Error> {
    let mut value = 0i32;
    let mut position = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        value |= ((byte[0] & 0x7F) as i32) << position;
        if (byte[0] & 0x80) == 0 {
            break;
        }
        position += 7;
        if position >= 32 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "VarInt too big"));
        }
    }
    Ok(value)
}

fn write_varint(mut value: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        if (value & !0x7F) == 0 {
            bytes.push(value as u8);
            break;
        }
        bytes.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
    bytes
}

fn resolve_dns_srv_if_needed(host: &str) -> (String, u16) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("powershell");
        cmd.arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(format!(
                "(Resolve-DnsName -Type SRV -Name '_minecraft._tcp.{}' -ErrorAction SilentlyContinue | Select-Object -First 1 | ForEach-Object {{ \"$($_.NameTarget):$($_.Port)\" }})",
                host
            ));
        cmd.creation_flags(0x08000000);
        if let Ok(out) = cmd.output() {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some((target_host, port_str)) = res.split_once(':') {
                if let Ok(p) = port_str.trim().parse::<u16>() {
                    let clean_target = target_host.trim().trim_end_matches('.').to_string();
                    if !clean_target.is_empty() {
                        return (clean_target, p);
                    }
                }
            }
        }
    }

    (host.to_string(), 25565)
}

fn parse_lan_motd_and_port(msg: &str) -> Option<(String, u16)> {
    let motd_start = msg.find("[MOTD]")? + 6;
    let motd_end = msg.find("[/MOTD]")?;
    let ad_start = msg.find("[AD]")? + 4;
    let ad_end = msg.find("[/AD]")?;

    if motd_start <= motd_end && ad_start <= ad_end {
        let motd = msg[motd_start..motd_end].to_string();
        let port = msg[ad_start..ad_end].trim().parse::<u16>().ok()?;
        Some((motd, port))
    } else {
        None
    }
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

    let (host_input, explicit_port) = if let Some((h, p)) = trimmed.split_once(':') {
        (h.trim(), p.trim().parse::<u16>().ok())
    } else {
        (trimmed, None)
    };

    let (resolved_host, target_port) = match explicit_port {
        Some(p) => (host_input.to_string(), p),
        None => {
            let (srv_h, srv_p) = resolve_dns_srv_if_needed(host_input);
            (srv_h, srv_p)
        }
    };

    let target = format!("{}:{}", resolved_host, target_port);
    let start_time = Instant::now();

    let mut stream = match tokio::time::timeout(Duration::from_secs(4), TcpStream::connect(&target)).await {
        Ok(Ok(s)) => s,
        _ => {
            return Ok(ServerPingResultDto {
                online: false,
                ping: None,
                motd: None,
                players: None,
                icon: None,
            });
        }
    };

    let latency = start_time.elapsed().as_millis() as i64;
    let _ = stream.set_nodelay(true);

    let host_bytes = resolved_host.as_bytes();
    let mut handshake_body = Vec::new();
    handshake_body.extend_from_slice(&write_varint(0));
    handshake_body.extend_from_slice(&write_varint(767));
    handshake_body.extend_from_slice(&write_varint(host_bytes.len() as i32));
    handshake_body.extend_from_slice(host_bytes);
    handshake_body.extend_from_slice(&target_port.to_be_bytes());
    handshake_body.extend_from_slice(&write_varint(1));

    let mut handshake_packet = Vec::new();
    handshake_packet.extend_from_slice(&write_varint(handshake_body.len() as i32));
    handshake_packet.extend_from_slice(&handshake_body);

    let request_packet = vec![0x01, 0x00];

    if stream.write_all(&handshake_packet).await.is_err() || stream.write_all(&request_packet).await.is_err() {
        return Ok(ServerPingResultDto {
            online: false,
            ping: None,
            motd: None,
            players: None,
            icon: None,
        });
    }

    let mut header_buf = [0u8; 8];
    let n = match tokio::time::timeout(Duration::from_millis(2500), stream.read(&mut header_buf)).await {
        Ok(Ok(bytes_read)) if bytes_read > 0 => bytes_read,
        _ => {
            return Ok(ServerPingResultDto {
                online: true,
                ping: Some(latency),
                motd: Some("Minecraft Server (Responsive)".to_string()),
                players: Some("Online".to_string()),
                icon: None,
            });
        }
    };

    let mut cursor = Cursor::new(&header_buf[..n]);
    let packet_len = match read_varint(&mut cursor) {
        Ok(len) => len as usize,
        Err(_) => return Ok(ServerPingResultDto {
            online: true,
            ping: Some(latency),
            motd: Some("Protected Server Gateway".to_string()),
            players: Some("Online".to_string()),
            icon: None,
        }),
    };

    let _ = read_varint(&mut cursor);
    let str_len = match read_varint(&mut cursor) {
        Ok(len) => len as usize,
        Err(_) => return Ok(ServerPingResultDto {
            online: true,
            ping: Some(latency),
            motd: Some("Minecraft Server".to_string()),
            players: Some("Online".to_string()),
            icon: None,
        }),
    };

    let header_consumed = cursor.position() as usize;
    let mut payload = Vec::with_capacity(str_len.min(packet_len) + 64);
    payload.extend_from_slice(&header_buf[header_consumed..n]);

    let target_total = str_len.min(65536);
    let read_start = Instant::now();

    while payload.len() < target_total && read_start.elapsed() < Duration::from_millis(3000) {
        let mut temp = [0u8; 8192];
        match tokio::time::timeout(Duration::from_millis(600), stream.read(&mut temp)).await {
            Ok(Ok(bytes)) if bytes > 0 => {
                payload.extend_from_slice(&temp[..bytes]);
            }
            _ => break,
        }
    }

    let json_slice = if payload.len() >= str_len {
        &payload[..str_len]
    } else {
        &payload[..]
    };

    if let Ok(parsed) = serde_json::from_slice::<Value>(json_slice) {
        let mut motd = "Minecraft Server".to_string();
        if let Some(desc) = parsed["description"]["text"].as_str() {
            motd = desc.to_string();
        } else if let Some(desc) = parsed["description"].as_str() {
            motd = desc.to_string();
        } else if let Some(extra) = parsed["description"]["extra"].as_array() {
            let joined: String = extra.iter().filter_map(|e| e["text"].as_str()).collect();
            if !joined.is_empty() {
                motd = joined;
            }
        }

        let online = parsed["players"]["online"].as_i64().unwrap_or(0);
        let max = parsed["players"]["max"].as_i64().unwrap_or(0);
        let players = format!("{}/{}", online, max);
        let icon = parsed["favicon"].as_str().map(|s| s.to_string());

        return Ok(ServerPingResultDto {
            online: true,
            ping: Some(latency),
            motd: Some(motd),
            players: Some(players),
            icon,
        });
    }

    Ok(ServerPingResultDto {
        online: true,
        ping: Some(latency),
        motd: Some("Minecraft Server (Online)".to_string()),
        players: Some("Online".to_string()),
        icon: None,
    })
}

#[tauri::command]
pub fn start_lan_discovery() -> bool {
    if LAN_LISTENER_ACTIVE.swap(true, Ordering::SeqCst) {
        return true;
    }

    tauri::async_runtime::spawn(async move {
        let socket = match UdpSocket::bind("0.0.0.0:4445").await {
            Ok(s) => s,
            Err(_) => {
                LAN_LISTENER_ACTIVE.store(false, Ordering::SeqCst);
                return;
            }
        };

        let multicast_addr = Ipv4Addr::new(224, 0, 2, 60);
        if socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED).is_err() {
            LAN_LISTENER_ACTIVE.store(false, Ordering::SeqCst);
            return;
        }

        let mut buf = [0u8; 1024];

        while LAN_LISTENER_ACTIVE.load(Ordering::Relaxed) {
            match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if let Some((motd, port)) = parse_lan_motd_and_port(&msg) {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        let key = format!("{}:{}", addr.ip(), port);
                        let mut guard = DISCOVERED_LAN_SERVERS.lock();
                        let map = guard.get_or_insert_with(HashMap::new);

                        map.insert(
                            key,
                            DiscoveredLanServerDto {
                                motd,
                                port,
                                address: addr.ip().to_string(),
                                last_seen_epoch: now,
                            },
                        );
                    }
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    });

    true
}

#[tauri::command]
pub fn get_discovered_lan_servers() -> Vec<DiscoveredLanServerDto> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut guard = DISCOVERED_LAN_SERVERS.lock();
    if let Some(map) = guard.as_mut() {
        map.retain(|_, v| now.saturating_sub(v.last_seen_epoch) < 15);
        return map.values().cloned().collect();
    }
    Vec::new()
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
        .map_err(AppError::Config)?;

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
        .map_err(AppError::Config)?;

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