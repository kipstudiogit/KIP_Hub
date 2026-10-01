use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde_json::{json, Value};
use crate::config;

pub struct BuilderManager {
    safe_fallback_keys: Vec<&'static str>,
}

impl BuilderManager {
    pub fn new() -> Self {
        Self {
            safe_fallback_keys: vec![
                "key.keyboard.keypad.0", "key.keyboard.keypad.1", "key.keyboard.keypad.2", "key.keyboard.keypad.3",
                "key.keyboard.keypad.4", "key.keyboard.keypad.5", "key.keyboard.keypad.6", "key.keyboard.keypad.7",
                "key.keyboard.keypad.8", "key.keyboard.keypad.9", "key.keyboard.f6", "key.keyboard.f7",
                "key.keyboard.f8", "key.keyboard.f9", "key.keyboard.f10", "key.keyboard.left.bracket",
                "key.keyboard.right.bracket", "key.keyboard.backslash", "key.keyboard.apostrophe", "key.keyboard.comma",
                "key.keyboard.period", "key.keyboard.slash", "key.keyboard.minus", "key.keyboard.equal",
            ],
        }
    }

    pub fn get_foundation_mods(&self, loader: &str) -> Vec<&'static str> {
        match loader.to_lowercase().as_str() {
            "fabric" => vec!["sodium", "lithium", "ferrite-core", "modmenu", "entityculling", "krypton", "indium", "cloth-config"],
            "forge" => vec!["embeddium", "ferrite-core", "entityculling", "cloth-config"],
            "neoforge" => vec!["embeddium", "ferrite-core", "entityculling", "cloth-config"],
            "quilt" => vec!["sodium", "lithium", "ferrite-core", "modmenu", "entityculling", "cloth-config"],
            _ => Vec::new(),
        }
    }

    pub fn resolve_keybinds(&self, mc_dir: &str) -> i32 {
        let options_path = Path::new(mc_dir).join("options.txt");
        if !options_path.exists() {
            return 0;
        }

        let content = match fs::read_to_string(&options_path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut keymaps: HashMap<String, String> = HashMap::new();
        let mut resolved_lines = Vec::new();
        let mut changes_made = 0;
        let mut fallback_idx = 0;

        for line in content.lines() {
            if line.starts_with("key_") {
                if let Some((k_id, k_val)) = line.trim().split_once(':') {
                    if k_val.contains("key.keyboard.unknown") {
                        resolved_lines.push(line.to_string());
                        continue;
                    }

                    if keymaps.contains_key(k_val) {
                        if fallback_idx < self.safe_fallback_keys.len() {
                            let new_val = self.safe_fallback_keys[fallback_idx];
                            fallback_idx += 1;
                            resolved_lines.push(format!("{}:{}", k_id, new_val));
                            keymaps.insert(new_val.to_string(), k_id.to_string());
                            changes_made += 1;
                        } else {
                            resolved_lines.push(format!("{}:key.keyboard.unknown", k_id));
                            changes_made += 1;
                        }
                    } else {
                        keymaps.insert(k_val.to_string(), k_id.to_string());
                        resolved_lines.push(line.to_string());
                    }
                } else {
                    resolved_lines.push(line.to_string());
                }
            } else {
                resolved_lines.push(line.to_string());
            }
        }

        if changes_made > 0 {
            let tmp_path = options_path.with_extension("tmp");
            if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(&tmp_path) {
                for line in resolved_lines {
                    let _ = writeln!(file, "{}", line);
                }
                let _ = file.sync_all();
                drop(file);
                
                let _ = fs::remove_file(&options_path);
                let _ = fs::rename(tmp_path, options_path);
            }
        }

        changes_made
    }

    pub async fn generate_mod_list(&self, prompt: &str, mc_version: &str, loader: &str) -> Result<Vec<String>, String> {
        let sys_prompt = format!(
            "You are an expert Minecraft modpack compiler. The user wants a modpack for Minecraft {} on {}.\nReturn ONLY a strictly valid JSON array of strings containing Modrinth project slugs that match the request.\nRule 1: NO markdown formatting, NO backticks.\nRule 2: NO explanation text.\nRule 3: Use ONLY double quotes for strings.\nRule 4: Do not include performance mods (sodium, etc).\nRule 5: Maximum 15 mods.\nExample response: [\"jei\", \"mouse-tweaks\", \"waystones\"]",
            mc_version, loader
        );

        let app_config = config::load_app_config();
        let client = reqwest::Client::new();
        let mut res_text = String::new();

        match app_config.ai_provider.as_str() {
            "openai" => {
                let api_key = config::get_secret("openai_api_key");
                if api_key.trim().is_empty() {
                    return Err("OpenAI API Key is missing.".to_string());
                }
                let payload = json!({
                    "model": "gpt-4o-mini",
                    "messages": [
                        {"role": "system", "content": sys_prompt},
                        {"role": "user", "content": prompt}
                    ]
                });
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                if let Ok(auth_val) = HeaderValue::from_str(&format!("Bearer {}", api_key.trim())) {
                    headers.insert(AUTHORIZATION, auth_val);
                }

                let res = client.post("https://api.openai.com/v1/chat/completions")
                    .headers(headers)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let data: Value = res.json().await.map_err(|e| e.to_string())?;
                res_text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            }
            "anthropic" => {
                let api_key = config::get_secret("anthropic_api_key");
                if api_key.trim().is_empty() {
                    return Err("Anthropic API Key is missing.".to_string());
                }
                let payload = json!({
                    "model": "claude-3-haiku-20240307",
                    "max_tokens": 1000,
                    "system": sys_prompt,
                    "messages": [{"role": "user", "content": prompt}]
                });
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                if let Ok(key_val) = HeaderValue::from_str(api_key.trim()) {
                    headers.insert("x-api-key", key_val);
                }
                headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

                let res = client.post("https://api.anthropic.com/v1/messages")
                    .headers(headers)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let data: Value = res.json().await.map_err(|e| e.to_string())?;
                res_text = data["content"][0]["text"].as_str().unwrap_or("").to_string();
            }
            "ollama" => {
                let url = if app_config.ollama_url.trim().is_empty() { "http://localhost:11434" } else { app_config.ollama_url.trim_end_matches('/') };
                let payload = json!({
                    "model": "llama3.1",
                    "system": sys_prompt,
                    "prompt": prompt,
                    "stream": false
                });
                let res = client.post(format!("{}/api/generate", url))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let data: Value = res.json().await.map_err(|e| e.to_string())?;
                res_text = data["response"].as_str().unwrap_or("").to_string();
            }
            _ => {
                let api_key = config::get_secret("ai_api_key");
                if api_key.trim().is_empty() {
                    return Err("Google Gemini API Key is missing.".to_string());
                }
                let payload = json!({
                    "contents": [{"parts": [{"text": format!("{}\nUser request: {}", sys_prompt, prompt)}]}]
                });
                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                if let Ok(key_val) = HeaderValue::from_str(api_key.trim()) {
                    headers.insert("x-goog-api-key", key_val);
                }

                let models = ["gemini-1.5-flash", "gemini-1.5-flash-latest", "gemini-1.0-pro"];
                for model in models {
                    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
                    if let Ok(res) = client.post(&url).headers(headers.clone()).json(&payload).send().await {
                        if res.status().is_success() {
                            if let Ok(data) = res.json::<Value>().await {
                                if let Some(t) = data["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                    res_text = t.to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let start = res_text.find('[');
        let end = res_text.rfind(']');
        if let (Some(s), Some(e)) = (start, end) {
            if s < e {
                let json_slice = &res_text[s..=e];
                if let Ok(Value::Array(items)) = serde_json::from_str(json_slice) {
                    let list: Vec<String> = items.into_iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_lowercase().replace(' ', "-")))
                        .collect();
                    return Ok(list);
                }
            }
        }

        Err("Failed to parse AI mod list.".to_string())
    }
}