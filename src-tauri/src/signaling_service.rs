use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
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
        let addr: SocketAddr = addr_str.parse()?;
        let listener = TcpListener::bind(&addr).await?;

        while let Ok((stream, _)) = listener.accept().await {
            let channels = Arc::clone(&self.channels);
            tokio::spawn(async move {
                let _ = Self::handle_connection(channels, stream).await;
            });
        }

        Ok(())
    }

    fn is_valid_message_type(msg_type: &str) -> bool {
        matches!(
            msg_type,
            "ping"
                | "offer"
                | "answer"
                | "ice-candidate"
                | "mute-state"
                | "deafen-state"
                | "party-invite"
                | "party-accept"
                | "party-decline"
        )
    }

    async fn handle_connection(channels: ChannelsMap, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let requested_uri = Arc::new(parking_lot::Mutex::new(String::new()));
        let uri_capture = Arc::clone(&requested_uri);

        let ws_stream = accept_hdr_async(stream, move |req: &Request, res: Response| {
            *uri_capture.lock() = req.uri().to_string();
            Ok(res)
        }).await?;

        let uri_str = requested_uri.lock().clone();
        let parsed_url = Url::parse(&format!("ws://localhost{}", uri_str))?;

        let mut channel_name = String::new();
        let mut user_name = "Guest".to_string();

        for (k, v) in parsed_url.query_pairs() {
            if k == "channel" {
                let trimmed = v.trim();
                if !trimmed.is_empty() && trimmed.len() <= 64 {
                    channel_name = trimmed.to_string();
                }
            } else if k == "user" {
                let trimmed = v.trim();
                if !trimmed.is_empty() && trimmed.len() <= 32 {
                    user_name = trimmed.to_string();
                }
            }
        }

        if channel_name.is_empty() {
            return Ok(());
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

            for (_, peer) in channel_peers.iter() {
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
                tx,
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
                        continue;
                    }

                    data["userId"] = json!(user_id);
                    data["userName"] = json!(user_name);

                    let target = data["target"].as_str().map(|s| s.to_string());
                    let payload_str = data.to_string();

                    let chs = channels.lock().await;
                    if let Some(channel_peers) = chs.get(&channel_name) {
                        if let Some(target_id) = target {
                            if let Some(peer) = channel_peers.get(&target_id) {
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

                for (_, peer) in channel_peers.iter() {
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