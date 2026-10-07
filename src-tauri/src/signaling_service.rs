use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use uuid::Uuid;

type PeerSender = mpsc::UnboundedSender<String>;

#[derive(Clone)]
struct PeerInfo {
    id: String,
    name: String,
    tx: PeerSender,
}

type ChannelsMap = Arc<Mutex<HashMap<String, HashMap<String, PeerInfo>>>>;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingMessageDto {
    pub r#type: String,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub target: Option<String>,
    pub sdp: Option<Value>,
    pub candidate: Option<Value>,
    pub muted: Option<bool>,
    pub deafened: Option<bool>,
    pub tunnel_url: Option<String>,
    pub mods: Option<Vec<String>>,
}

pub struct SignalingService {
    channels: ChannelsMap,
}

impl SignalingService {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run_server(&self, addr_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = if addr_str.starts_with("0.0.0.0") {
            "127.0.0.1:8765".parse()?
        } else {
            addr_str.parse()?
        };

        let listener = TcpListener::bind(&addr).await?;

        while let Ok((stream, _)) = listener.accept().await {
            let channels = Arc::clone(&self.channels);
            tokio::spawn(async move {
                let _ = Self::handle_connection(channels, stream).await;
            });
        }

        Ok(())
    }

    fn is_origin_allowed(req: &Request) -> bool {
        if let Some(origin_header) = req.headers().get("Origin").and_then(|v| v.to_str().ok()) {
            let lower = origin_header.to_lowercase();
            lower.starts_with("tauri://")
                || lower.starts_with("http://tauri.localhost")
                || lower.starts_with("https://tauri.localhost")
                || lower.starts_with("http://localhost")
                || lower.starts_with("https://localhost")
                || lower.starts_with("http://127.0.0.1")
                || lower.starts_with("https://127.0.0.1")
                || lower.starts_with("https://kip-backend.noisyfutlor98.workers.dev")
        } else {
            true
        }
    }

    fn is_valid_message_type(msg_type: &str) -> bool {
        matches!(
            msg_type,
            "ping"
                | "pong"
                | "offer"
                | "answer"
                | "ice-candidate"
                | "mute-state"
                | "deafen-state"
                | "party-invite"
                | "party-accept"
                | "party-decline"
                | "lobby-sync"
        )
    }

    async fn handle_connection(channels: ChannelsMap, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let requested_uri = Arc::new(parking_lot::Mutex::new(String::new()));
        let uri_capture = Arc::clone(&requested_uri);

        let ws_stream = match accept_hdr_async(stream, move |req: &Request, res: Response| {
            if !Self::is_origin_allowed(req) {
                let mut forbidden_res = ErrorResponse::new(Some("Forbidden origin".to_string()));
                *forbidden_res.status_mut() = StatusCode::FORBIDDEN;
                return Err(forbidden_res);
            }
            *uri_capture.lock() = req.uri().to_string();
            Ok(res)
        }).await {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let uri_str = requested_uri.lock().clone();
        let target_parse = if uri_str.starts_with('/') {
            format!("ws://127.0.0.1{}", uri_str)
        } else {
            format!("ws://127.0.0.1/{}", uri_str)
        };

        let parsed_url = Url::parse(&target_parse)?;

        let mut channel_name = String::new();
        let mut user_name = "Guest".to_string();

        for (k, v) in parsed_url.query_pairs() {
            if k == "channel" {
                let trimmed = v.trim().to_lowercase();
                if !trimmed.is_empty() && trimmed.len() <= 64 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                    channel_name = trimmed;
                }
            } else if k == "user" {
                let trimmed = v.trim();
                if !trimmed.is_empty() && trimmed.len() <= 32 {
                    user_name = trimmed.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == ' ').collect();
                }
            }
        }

        if channel_name.is_empty() {
            channel_name = "global".to_string();
        }

        let user_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        {
            let mut chs = channels.lock().await;
            let channel_peers = chs.entry(channel_name.clone()).or_insert_with(HashMap::new);

            let notify_existing_peers = json!({
                "type": "user-joined",
                "userId": user_id,
                "userName": user_name,
                "initiator": true
            }).to_string();

            for peer in channel_peers.values() {
                let _ = peer.tx.send(notify_existing_peers.clone());

                let notify_new_client = json!({
                    "type": "user-joined",
                    "userId": peer.id,
                    "userName": peer.name,
                    "initiator": false
                }).to_string();
                let _ = tx.send(notify_new_client);
            }

            channel_peers.insert(user_id.clone(), PeerInfo {
                id: user_id.clone(),
                name: user_name.clone(),
                tx: tx.clone(),
            });
        }

        let user_id_clone = user_id.clone();
        tokio::spawn(async move {
            while let Some(msg_str) = rx.recv().await {
                if ws_sender.send(Message::Text(msg_str.into())).await.is_err() {
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                if text.len() > 65536 {
                    continue;
                }

                if let Ok(mut data) = serde_json::from_str::<Value>(&text) {
                    let msg_type = match data["type"].as_str() {
                        Some(t) => t,
                        None => continue,
                    };

                    if !Self::is_valid_message_type(msg_type) {
                        continue;
                    }

                    if msg_type == "ping" {
                        let _ = tx.send(json!({ "type": "pong" }).to_string());
                        continue;
                    }

                    data["userId"] = json!(user_id);
                    data["userName"] = json!(user_name);

                    let target = data["target"].as_str().map(|s| s.to_string());
                    let payload_str = data.to_string();

                    let chs = channels.lock().await;
                    if let Some(channel_peers) = chs.get(&channel_name) {
                        if let Some(ref target_id) = target {
                            let found_peer = channel_peers.get(target_id).or_else(|| {
                                channel_peers.values().find(|p| p.name.eq_ignore_ascii_case(target_id))
                            });

                            if let Some(peer) = found_peer {
                                let _ = peer.tx.send(payload_str);
                            }
                        } else {
                            for (p_id, peer) in channel_peers.iter() {
                                if p_id != &user_id {
                                    let _ = peer.tx.send(payload_str.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut chs = channels.lock().await;
            if let Some(channel_peers) = chs.get_mut(&channel_name) {
                channel_peers.remove(&user_id_clone);
                let leave_msg = json!({
                    "type": "user-left",
                    "userId": user_id_clone
                }).to_string();

                for peer in channel_peers.values() {
                    let _ = peer.tx.send(leave_msg.clone());
                }
            }

            if let Some(channel_peers) = chs.get(&channel_name) {
                if channel_peers.is_empty() {
                    chs.remove(&channel_name);
                }
            }
        }

        Ok(())
    }
}