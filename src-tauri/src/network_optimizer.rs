use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sysinfo::Networks;
use tauri::ipc::Channel;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawSocket;

#[cfg(not(target_os = "windows"))]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "windows")]
#[link(name = "ws2_32")]
extern "system" {
    fn setsockopt(s: usize, level: i32, optname: i32, optval: *const i8, optlen: i32) -> i32;
}

#[cfg(not(target_os = "windows"))]
extern "C" {
    fn setsockopt(sockfd: i32, level: i32, optname: i32, optval: *const std::ffi::c_void, optlen: u32) -> i32;
}

const SOL_SOCKET_VAL: i32 = if cfg!(target_os = "windows") { 0xffff } else { 1 };
const SO_SNDBUF_VAL: i32 = if cfg!(target_os = "windows") { 0x1001 } else { 7 };
const SO_RCVBUF_VAL: i32 = if cfg!(target_os = "windows") { 0x1002 } else { 8 };
const IPPROTO_IP_VAL: i32 = 0;
const IP_TOS_VAL: i32 = if cfg!(target_os = "windows") { 3 } else { 1 };
const IPTOS_LOWDELAY_VAL: i32 = 0x10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingMasterConfigDto {
    pub routing_mode: String,
    pub upstream_socks5_host: String,
    pub upstream_socks5_port: u16,
    pub anti_bufferbloat: bool,
    pub dscp_qos_enabled: bool,
}

impl Default for PingMasterConfigDto {
    fn default() -> Self {
        Self {
            routing_mode: "adaptive".to_string(),
            upstream_socks5_host: "127.0.0.1".to_string(),
            upstream_socks5_port: 7890,
            anti_bufferbloat: true,
            dscp_qos_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnDetectionDto {
    pub vpn_active: bool,
    pub active_adapter_name: Option<String>,
    pub adapter_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingMasterMetricDto {
    pub local_proxy_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub current_ping_ms: i64,
    pub jitter_ms: f64,
    pub min_ping_ms: i64,
    pub max_ping_ms: i64,
    pub packets_optimized: u64,
    pub bytes_transferred: u64,
    pub tcp_nodelay_active: bool,
    pub bufferbloat_reduced: bool,
    pub active_route: String,
    pub vpn_detected: bool,
    pub active_vpn_adapter: Option<String>,
    pub routing_mode: String,
    pub hit_reg_quality: String,
    pub packet_loss_percent: f64,
}

pub struct NetworkOptimizer {
    running: Arc<AtomicBool>,
    active_local_port: Arc<Mutex<Option<u16>>>,
    target_endpoint: Arc<Mutex<Option<String>>>,
    config: Arc<Mutex<PingMasterConfigDto>>,
    packets_transferred: Arc<AtomicU64>,
    bytes_transferred: Arc<AtomicU64>,
    rolling_ping_ms: Arc<Mutex<Vec<i64>>>,
}

impl NetworkOptimizer {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            active_local_port: Arc::new(Mutex::new(None)),
            target_endpoint: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(PingMasterConfigDto::default())),
            packets_transferred: Arc::new(AtomicU64::new(0)),
            bytes_transferred: Arc::new(AtomicU64::new(0)),
            rolling_ping_ms: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_config(&self) -> PingMasterConfigDto {
        self.config.lock().clone()
    }

    pub fn set_config(&self, cfg: PingMasterConfigDto) {
        *self.config.lock() = cfg;
    }

    pub fn detect_vpn_environment() -> VpnDetectionDto {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let vpn_keywords = [
            "tun", "tap", "wintun", "wireguard", "openvpn", "tailscale",
            "warp", "nord", "proton", "sing-box", "clash", "v2ray", "xray",
        ];

        let mut vpn_active = false;
        let mut active_adapter = None;
        let mut list = Vec::new();

        for (name, _) in networks.iter() {
            let lower = name.to_lowercase();
            list.push(name.clone());

            if vpn_keywords.iter().any(|k| lower.contains(k)) {
                vpn_active = true;
                if active_adapter.is_none() {
                    active_adapter = Some(name.clone());
                }
            }
        }

        VpnDetectionDto {
            vpn_active,
            active_adapter_name: active_adapter,
            adapter_list: list,
        }
    }

    fn read_varint<R: std::io::Read>(reader: &mut R) -> Result<(i32, usize), std::io::Error> {
        let mut value = 0i32;
        let mut position = 0;
        let mut bytes_read = 0;

        loop {
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;
            bytes_read += 1;

            value |= ((byte[0] & 0x7F) as i32) << position;

            if (byte[0] & 0x80) == 0 {
                break;
            }

            position += 7;
            if position >= 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarInt too big",
                ));
            }
        }

        Ok((value, bytes_read))
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

    fn rewrite_minecraft_handshake(
        raw_buffer: &[u8],
        target_host: &str,
        target_port: u16,
    ) -> Option<Vec<u8>> {
        let mut cursor = Cursor::new(raw_buffer);

        let (packet_len, len_bytes_count) = Self::read_varint(&mut cursor).ok()?;
        let packet_start = len_bytes_count;
        let packet_end = packet_start + packet_len as usize;

        if raw_buffer.len() < packet_start {
            return None;
        }

        let (packet_id, _) = Self::read_varint(&mut cursor).ok()?;
        if packet_id != 0 {
            return None;
        }

        let (proto_version, _) = Self::read_varint(&mut cursor).ok()?;
        let (host_str_len, _) = Self::read_varint(&mut cursor).ok()?;
        if host_str_len < 0 || host_str_len > 255 {
            return None;
        }

        let current_pos = cursor.position() as usize;
        let after_host_pos = current_pos + host_str_len as usize;
        if raw_buffer.len() < after_host_pos + 2 {
            return None;
        }

        cursor.set_position(after_host_pos as u64);

        let mut port_bytes = [0u8; 2];
        std::io::Read::read_exact(&mut cursor, &mut port_bytes).ok()?;

        let (next_state, _) = Self::read_varint(&mut cursor).ok()?;

        let mut packet_body = Vec::new();
        packet_body.extend_from_slice(&Self::write_varint(0));
        packet_body.extend_from_slice(&Self::write_varint(proto_version));
        packet_body.extend_from_slice(&Self::write_varint(target_host.len() as i32));
        packet_body.extend_from_slice(target_host.as_bytes());
        packet_body.extend_from_slice(&target_port.to_be_bytes());
        packet_body.extend_from_slice(&Self::write_varint(next_state));

        let mut new_packet = Vec::new();
        new_packet.extend_from_slice(&Self::write_varint(packet_body.len() as i32));
        new_packet.extend_from_slice(&packet_body);

        if raw_buffer.len() > packet_end {
            new_packet.extend_from_slice(&raw_buffer[packet_end..]);
        }

        Some(new_packet)
    }

    fn tune_socket(stream: &TcpStream, anti_bufferbloat: bool, dscp_qos: bool) {
        let _ = stream.set_nodelay(true);

        #[cfg(target_os = "windows")]
        unsafe {
            let sock = stream.as_raw_socket() as usize;
            if anti_bufferbloat {
                let buf_size: i32 = 8192;
                setsockopt(sock, SOL_SOCKET_VAL, SO_SNDBUF_VAL, &buf_size as *const _ as *const i8, 4);
                setsockopt(sock, SOL_SOCKET_VAL, SO_RCVBUF_VAL, &buf_size as *const _ as *const i8, 4);
            }
            if dscp_qos {
                let tos: i32 = IPTOS_LOWDELAY_VAL;
                setsockopt(sock, IPPROTO_IP_VAL, IP_TOS_VAL, &tos as *const _ as *const i8, 4);
            }
        }

        #[cfg(not(target_os = "windows"))]
        unsafe {
            let sock = stream.as_raw_fd();
            if anti_bufferbloat {
                let buf_size: i32 = 8192;
                setsockopt(sock, SOL_SOCKET_VAL, SO_SNDBUF_VAL, &buf_size as *const _ as *const std::ffi::c_void, 4);
                setsockopt(sock, SOL_SOCKET_VAL, SO_RCVBUF_VAL, &buf_size as *const _ as *const std::ffi::c_void, 4);
            }
            if dscp_qos {
                let tos: i32 = IPTOS_LOWDELAY_VAL;
                setsockopt(sock, IPPROTO_IP_VAL, IP_TOS_VAL, &tos as *const _ as *const std::ffi::c_void, 4);
            }
        }
    }

    async fn connect_socks5(
        proxy_host: &str,
        proxy_port: u16,
        target_host: &str,
        target_port: u16,
    ) -> Result<TcpStream, String> {
        let mut stream = TcpStream::connect(format!("{}:{}", proxy_host, proxy_port))
            .await
            .map_err(|e| e.to_string())?;

        stream.write_all(&[0x05, 0x01, 0x00]).await.map_err(|e| e.to_string())?;

        let mut auth_response = [0u8; 2];
        stream.read_exact(&mut auth_response).await.map_err(|e| e.to_string())?;

        if auth_response[0] != 0x05 || auth_response[1] != 0x00 {
            return Err("SOCKS5 proxy authentication rejected".to_string());
        }

        let host_bytes = target_host.as_bytes();
        let mut req = Vec::with_capacity(7 + host_bytes.len());
        req.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, host_bytes.len() as u8]);
        req.extend_from_slice(host_bytes);
        req.extend_from_slice(&target_port.to_be_bytes());

        stream.write_all(&req).await.map_err(|e| e.to_string())?;

        let mut resp_header = [0u8; 4];
        stream.read_exact(&mut resp_header).await.map_err(|e| e.to_string())?;

        if resp_header[1] != 0x00 {
            return Err(format!("SOCKS5 target unreachable (code: {})", resp_header[1]));
        }

        match resp_header[3] {
            1 => {
                let mut discard = [0u8; 6];
                stream.read_exact(&mut discard).await.map_err(|e| e.to_string())?;
            }
            3 => {
                let mut len = [0u8; 1];
                stream.read_exact(&mut len).await.map_err(|e| e.to_string())?;
                let mut discard = vec![0u8; len[0] as usize + 2];
                stream.read_exact(&mut discard).await.map_err(|e| e.to_string())?;
            }
            4 => {
                let mut discard = [0u8; 18];
                stream.read_exact(&mut discard).await.map_err(|e| e.to_string())?;
            }
            _ => return Err("Invalid SOCKS5 address type".to_string()),
        }

        Ok(stream)
    }

    async fn connect_upstream(
        target_host: &str,
        target_port: u16,
        config: &PingMasterConfigDto,
    ) -> Result<(TcpStream, String), String> {
        if config.routing_mode == "socks5" {
            let stream = Self::connect_socks5(
                &config.upstream_socks5_host,
                config.upstream_socks5_port,
                target_host,
                target_port,
            ).await?;
            return Ok((stream, format!("SOCKS5 ({}:{})", config.upstream_socks5_host, config.upstream_socks5_port)));
        }

        let target_str = format!("{}:{}", target_host, target_port);

        if config.routing_mode == "direct" || config.routing_mode == "vpn" {
            let stream = TcpStream::connect(&target_str).await.map_err(|e| e.to_string())?;
            return Ok((stream, if config.routing_mode == "direct" { "Direct Adapter" } else { "VPN Gateway" }.to_string()));
        }

        let direct_future = TcpStream::connect(&target_str);
        let socks_future = Self::connect_socks5(
            &config.upstream_socks5_host,
            config.upstream_socks5_port,
            target_host,
            target_port,
        );

        tokio::pin!(direct_future);
        tokio::pin!(socks_future);

        tokio::select! {
            res = &mut direct_future => {
                match res {
                    Ok(s) => Ok((s, "Direct Fast-Path".to_string())),
                    Err(_) => {
                        let fallback_socks = socks_future.await?;
                        Ok((fallback_socks, "SOCKS5 Tunnel Fallback".to_string()))
                    }
                }
            }
            res = &mut socks_future => {
                match res {
                    Ok(s) => Ok((s, "SOCKS5 Proxy Winner".to_string())),
                    Err(_) => {
                        let fallback_direct = direct_future.await.map_err(|e| e.to_string())?;
                        Ok((fallback_direct, "Direct Path Fallback".to_string()))
                    }
                }
            }
        }
    }

    pub async fn start_proxy(
        &self,
        target_host: String,
        target_port: u16,
        config: PingMasterConfigDto,
        channel: Channel<PingMasterMetricDto>,
    ) -> Result<u16, String> {
        self.stop();

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| e.to_string())?;

        let local_port = listener
            .local_addr()
            .map_err(|e| e.to_string())?
            .port();

        self.running.store(true, Ordering::SeqCst);
        *self.active_local_port.lock() = Some(local_port);
        *self.target_endpoint.lock() = Some(format!("{}:{}", target_host, target_port));
        *self.config.lock() = config.clone();
        self.packets_transferred.store(0, Ordering::SeqCst);
        self.bytes_transferred.store(0, Ordering::SeqCst);

        let running_flag = Arc::clone(&self.running);
        let packets_counter = Arc::clone(&self.packets_transferred);
        let bytes_counter = Arc::clone(&self.bytes_transferred);
        let rolling_pings = Arc::clone(&self.rolling_ping_ms);
        let host_clone = target_host.clone();
        let cfg_clone = config.clone();

        tokio::spawn(async move {
            let channel_ref = channel.clone();
            let running_metric = Arc::clone(&running_flag);
            let pings_metric = Arc::clone(&rolling_pings);
            let packets_metric = Arc::clone(&packets_counter);
            let bytes_metric = Arc::clone(&bytes_counter);
            let host_metric = host_clone.clone();
            let cfg_metric = cfg_clone.clone();

            tokio::spawn(async move {
                let vpn_info = Self::detect_vpn_environment();

                while running_metric.load(Ordering::Relaxed) {
                    let ping_start = Instant::now();
                    let ping_measure = match tokio::time::timeout(
                        Duration::from_millis(1200),
                        Self::connect_upstream(&host_metric, target_port, &cfg_metric),
                    ).await {
                        Ok(Ok((stream, _))) => {
                            Self::tune_socket(&stream, cfg_metric.anti_bufferbloat, cfg_metric.dscp_qos_enabled);
                            let ms = ping_start.elapsed().as_millis() as i64;
                            drop(stream);
                            ms
                        }
                        _ => -1,
                    };

                    let mut jitter = 0.0;
                    let mut min_ping = 0;
                    let mut max_ping = 0;

                    if ping_measure > 0 {
                        let mut p_guard = pings_metric.lock();
                        p_guard.push(ping_measure);
                        if p_guard.len() > 30 {
                            p_guard.remove(0);
                        }

                        if !p_guard.is_empty() {
                            min_ping = *p_guard.iter().min().unwrap_or(&ping_measure);
                            max_ping = *p_guard.iter().max().unwrap_or(&ping_measure);
                        }

                        if p_guard.len() >= 2 {
                            let mut diff_sum = 0.0;
                            for i in 1..p_guard.len() {
                                diff_sum += (p_guard[i] - p_guard[i - 1]).abs() as f64;
                            }
                            jitter = (diff_sum / (p_guard.len() - 1) as f64 * 10.0).round() / 10.0;
                        }
                    }

                    let quality_grade = if ping_measure <= 0 {
                        "Standby"
                    } else if jitter <= 1.5 && ping_measure < 40 {
                        "S-Tier Esports (Zero-Bufferbloat)"
                    } else if jitter <= 4.0 && ping_measure < 80 {
                        "A-Tier Optimal Hit-Reg"
                    } else if jitter <= 10.0 {
                        "B-Tier Moderate Latency"
                    } else {
                        "Degraded (Jitter Detected)"
                    };

                    let active_route_name = if cfg_metric.routing_mode == "socks5" {
                        format!("SOCKS5 ({}:{})", cfg_metric.upstream_socks5_host, cfg_metric.upstream_socks5_port)
                    } else if vpn_info.vpn_active && cfg_metric.routing_mode != "direct" {
                        format!("VPN Tunnel ({})", vpn_info.active_adapter_name.as_deref().unwrap_or("Virtual NIC"))
                    } else {
                        "Direct Wire (Physical NIC)".to_string()
                    };

                    let _ = channel_ref.send(PingMasterMetricDto {
                        local_proxy_port: local_port,
                        target_host: host_metric.clone(),
                        target_port,
                        current_ping_ms: ping_measure,
                        jitter_ms: jitter,
                        min_ping_ms: min_ping,
                        max_ping_ms: max_ping,
                        packets_optimized: packets_metric.load(Ordering::Relaxed),
                        bytes_transferred: bytes_metric.load(Ordering::Relaxed),
                        tcp_nodelay_active: true,
                        bufferbloat_reduced: cfg_metric.anti_bufferbloat,
                        active_route: active_route_name,
                        vpn_detected: vpn_info.vpn_active,
                        active_vpn_adapter: vpn_info.active_adapter_name.clone(),
                        routing_mode: cfg_metric.routing_mode.clone(),
                        hit_reg_quality: quality_grade.to_string(),
                        packet_loss_percent: if ping_measure < 0 { 100.0 } else { 0.0 },
                    });

                    tokio::time::sleep(Duration::from_millis(900)).await;
                }
            });

            while running_flag.load(Ordering::Relaxed) {
                if let Ok((mut client_stream, _)) = listener.accept().await {
                    Self::tune_socket(&client_stream, cfg_clone.anti_bufferbloat, cfg_clone.dscp_qos_enabled);

                    let target_h = host_clone.clone();
                    let target_p = target_port;
                    let target_cfg = cfg_clone.clone();
                    let p_counter = Arc::clone(&packets_counter);
                    let b_counter = Arc::clone(&bytes_counter);

                    tokio::spawn(async move {
                        let mut first_chunk = [0u8; 1024];
                        let read_count = match client_stream.read(&mut first_chunk).await {
                            Ok(n) if n > 0 => n,
                            _ => return,
                        };

                        let initial_payload = match Self::rewrite_minecraft_handshake(&first_chunk[..read_count], &target_h, target_p) {
                            Some(rewritten) => rewritten,
                            None => first_chunk[..read_count].to_vec(),
                        };

                        if let Ok((mut server_stream, _)) = Self::connect_upstream(&target_h, target_p, &target_cfg).await {
                            Self::tune_socket(&server_stream, target_cfg.anti_bufferbloat, target_cfg.dscp_qos_enabled);

                            if server_stream.write_all(&initial_payload).await.is_err() {
                                return;
                            }

                            p_counter.fetch_add(1, Ordering::Relaxed);
                            b_counter.fetch_add(initial_payload.len() as u64, Ordering::Relaxed);

                            let (mut c_read, mut c_write) = client_stream.into_split();
                            let (mut s_read, mut s_write) = server_stream.into_split();

                            let p1 = Arc::clone(&p_counter);
                            let b1 = Arc::clone(&b_counter);
                            let client_to_server = tokio::spawn(async move {
                                let mut buf = [0u8; 8192];
                                while let Ok(n) = c_read.read(&mut buf).await {
                                    if n == 0 { break; }
                                    if s_write.write_all(&buf[..n]).await.is_err() { break; }
                                    p1.fetch_add(1, Ordering::Relaxed);
                                    b1.fetch_add(n as u64, Ordering::Relaxed);
                                }
                            });

                            let p2 = Arc::clone(&p_counter);
                            let b2 = Arc::clone(&b_counter);
                            let server_to_client = tokio::spawn(async move {
                                let mut buf = [0u8; 8192];
                                while let Ok(n) = s_read.read(&mut buf).await {
                                    if n == 0 { break; }
                                    if c_write.write_all(&buf[..n]).await.is_err() { break; }
                                    p2.fetch_add(1, Ordering::Relaxed);
                                    b2.fetch_add(n as u64, Ordering::Relaxed);
                                }
                            });

                            let _ = tokio::join!(client_to_server, server_to_client);
                        }
                    });
                }
            }
        });

        Ok(local_port)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.active_local_port.lock() = None;
        *self.target_endpoint.lock() = None;
        self.rolling_ping_ms.lock().clear();
    }
}