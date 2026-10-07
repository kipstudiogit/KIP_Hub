use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use keyring::Entry;
use machine_uid::get as get_machine_uid;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};

pub const APP_NAME: &str = "K.I.P.";
pub const APP_VERSION: &str = "2.0.0";
pub const CLOUDFLARE_URL: &str = "https://kip-backend.noisyfutlor98.workers.dev";
pub const DISCORD_CLIENT_ID: &str = "1546169959799984301";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub lang: String,
    pub autostart: bool,
    pub theme: String,
    pub custom_color: String,
    pub appearance: String,
    pub mica: bool,
    pub rpc: bool,
    pub scale: f32,
    pub instances: Vec<String>,
    pub current_instance: String,
    pub auto_backup: bool,
    pub ptero_url: String,
    pub low_graphics: bool,
    pub ai_provider: String,
    pub ai_model: String,
    pub ollama_url: String,
    pub close_on_launch: bool,
    pub ram_allocation: i32,
    pub jvm_gc: String,
    pub jvm_preset: String,
    pub shield_auto_scan: bool,
    pub voice_noise_suppression: bool,
    pub eula_accepted: bool,
    pub telemetry_opt_in: bool,
    pub kip_username: String,
    pub offline_username: String,
    pub game_resolution: String,
    pub game_fullscreen: bool,
    pub custom_java_path: String,
    pub custom_jvm_args: String,
    pub theme_accent: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_mc_dir = get_default_mc_dir().to_string_lossy().to_string();
        Self {
            lang: "en".to_string(),
            autostart: false,
            theme: "Indigo".to_string(),
            custom_color: "".to_string(),
            appearance: "Dark".to_string(),
            mica: true,
            rpc: true,
            scale: 1.0,
            instances: vec![default_mc_dir.clone()],
            current_instance: default_mc_dir,
            auto_backup: false,
            ptero_url: "".to_string(),
            low_graphics: false,
            ai_provider: "google".to_string(),
            ai_model: "gemini-1.5-flash".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            close_on_launch: false,
            ram_allocation: 0,
            jvm_gc: "G1GC".to_string(),
            jvm_preset: "balanced".to_string(),
            shield_auto_scan: true,
            voice_noise_suppression: true,
            eula_accepted: false,
            telemetry_opt_in: false,
            kip_username: "".to_string(),
            offline_username: "Player".to_string(),
            game_resolution: "1920x1080".to_string(),
            game_fullscreen: false,
            custom_java_path: "".to_string(),
            custom_jvm_args: "".to_string(),
            theme_accent: "indigo".to_string(),
        }
    }
}

pub fn get_app_data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("KIP_Hub");
    let _ = fs::create_dir_all(&path);
    path
}

pub fn get_default_mc_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(mut path) = dirs::config_dir() {
            path.push(".minecraft");
            return path;
        }
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".minecraft");
        path
    }
    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("minecraft");
        path
    }
    #[cfg(target_os = "linux")]
    {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".minecraft");
        path
    }
}

pub fn update_paths(_new_dir: &str) {}

pub fn load_app_config() -> AppConfig {
    let config_path = get_app_data_dir().join("config.json");
    if let Ok(data) = fs::read_to_string(&config_path) {
        if let Ok(mut config) = serde_json::from_str::<AppConfig>(&data) {
            let def = get_default_mc_dir().to_string_lossy().to_string();
            if !PathBuf::from(&config.current_instance).exists() {
                config.current_instance = def.clone();
                if !config.instances.contains(&def) {
                    config.instances.push(def);
                }
                save_app_config(&config);
            }
            return config;
        }
    }
    AppConfig::default()
}

pub fn save_app_config(config: &AppConfig) {
    let config_path = get_app_data_dir().join("config.json");
    if let Ok(data) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path, data);
    }
}

fn get_machine_key() -> Vec<u8> {
    let uid = get_machine_uid().unwrap_or_else(|_| "fallback_machine_key".to_string());
    let unique_str = format!("{}_KIP_SALT_9901", uid);
    let mut hasher = Sha256::new();
    hasher.update(unique_str.as_bytes());
    hasher.finalize().to_vec()
}

fn xor_encrypt(data: &str) -> String {
    let key = get_machine_key();
    let xored: Vec<u8> = data.as_bytes().iter().enumerate().map(|(i, b)| b ^ key[i % key.len()]).collect();
    BASE64.encode(xored)
}

fn xor_decrypt(data: &str) -> String {
    let key = get_machine_key();
    if let Ok(decoded) = BASE64.decode(data) {
        let xored: Vec<u8> = decoded.into_iter().enumerate().map(|(i, b)| b ^ key[i % key.len()]).collect();
        String::from_utf8(xored).unwrap_or_default()
    } else {
        String::new()
    }
}

pub fn get_secret(key: &str) -> String {
    if let Ok(entry) = Entry::new("KIP_Hub", key) {
        if let Ok(pw) = entry.get_password() {
            if !pw.is_empty() {
                return pw;
            }
        }
    }
    let secrets_path = get_app_data_dir().join("secrets.json");
    if let Ok(data) = fs::read_to_string(secrets_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(val) = json.get(key).and_then(|v| v.as_str()) {
                return xor_decrypt(val);
            }
        }
    }
    String::new()
}

pub fn set_secret(key: &str, value: &str) {
    if let Ok(entry) = Entry::new("KIP_Hub", key) {
        if value.is_empty() {
            let _ = entry.delete_credential();
        } else {
            let _ = entry.set_password(value);
        }
    }
    let secrets_path = get_app_data_dir().join("secrets.json");
    let mut map = if let Ok(data) = fs::read_to_string(&secrets_path) {
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&data).unwrap_or_default()
    } else {
        serde_json::Map::new()
    };
    if value.is_empty() {
        map.remove(key);
    } else {
        map.insert(key.to_string(), serde_json::Value::String(xor_encrypt(value)));
    }
    let _ = fs::write(secrets_path, serde_json::to_string_pretty(&map).unwrap_or_default());
}
