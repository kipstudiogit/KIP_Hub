use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;
use crate::config;

pub struct ApiManager {
    client: Client,
    app_version: String,
}

impl ApiManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            app_version: config::APP_VERSION.to_string(),
        }
    }

    fn get_default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let user_agent = format!("KIPStudio/KIP_Hub/{} (contact@kip.studio)", self.app_version);
        let header_val = HeaderValue::from_str(&user_agent)
            .unwrap_or_else(|_| HeaderValue::from_static("KIPStudio/KIP_Hub"));
        headers.insert(USER_AGENT, header_val);
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers
    }

    fn get_cf_headers(&self) -> HeaderMap {
        let mut headers = self.get_default_headers();
        let key = config::get_secret("cf_api_key");
        if !key.trim().is_empty() {
            if let Ok(val) = HeaderValue::from_str(key.trim()) {
                headers.insert("x-api-key", val);
            }
        }
        headers
    }

    pub async fn ask_ai_crash_analysis(&self, log_text: &str) -> String {
        let app_config = config::load_app_config();
        let max_len = std::cmp::min(log_text.len(), 4000);
        let prompt = format!("You are a Minecraft expert. Analyze this crash log and briefly explain the cause and how to fix it:\n\n{}", &log_text[..max_len]);

        match app_config.ai_provider.as_str() {
            "openai" => self.ask_openai(&prompt).await,
            "anthropic" => self.ask_anthropic(&prompt).await,
            "ollama" => self.ask_ollama(&prompt, &app_config.ollama_url).await,
            _ => self.ask_google(&prompt).await,
        }
    }

    async fn ask_google(&self, prompt: &str) -> String {
        let api_key = config::get_secret("ai_api_key");
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return "API Key is not configured.".to_string();
        }

        let key_header = match HeaderValue::from_str(trimmed_key) {
            Ok(v) => v,
            Err(_) => return "Invalid Gemini API Key format.".to_string(),
        };

        let payload = json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });

        let mut headers = self.get_default_headers();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-goog-api-key", key_header);

        let models = ["gemini-1.5-flash", "gemini-1.5-flash-latest", "gemini-1.0-pro"];
        let mut last_error = String::new();

        for model in models {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
            match self.client.post(&url).headers(headers.clone()).json(&payload).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        if let Ok(data) = res.json::<Value>().await {
                            if let Some(text) = data["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                return text.to_string();
                            }
                        }
                    } else {
                        let status_code = res.status().as_u16();
                        last_error = format!("HTTP {}", status_code);
                        if status_code == 400 || status_code == 401 || status_code == 403 || status_code == 429 {
                            break;
                        }
                    }
                }
                Err(e) => last_error = e.to_string(),
            }
        }
        format!("Google API Error: {}", last_error)
    }

    async fn ask_openai(&self, prompt: &str) -> String {
        let api_key = config::get_secret("openai_api_key");
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return "OpenAI API Key is not configured.".to_string();
        }

        let auth_header = match HeaderValue::from_str(&format!("Bearer {}", trimmed_key)) {
            Ok(v) => v,
            Err(_) => return "Invalid OpenAI API Key format.".to_string(),
        };

        let payload = json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": prompt}]
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, auth_header);

        match self.client.post("https://api.openai.com/v1/chat/completions").headers(headers).json(&payload).send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    if let Ok(data) = res.json::<Value>().await {
                        if let Some(text) = data["choices"][0]["message"]["content"].as_str() {
                            return text.to_string();
                        }
                    }
                }
                format!("OpenAI Error: {}", status)
            }
            Err(e) => format!("OpenAI Network Error: {}", e),
        }
    }

    async fn ask_anthropic(&self, prompt: &str) -> String {
        let api_key = config::get_secret("anthropic_api_key");
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return "Anthropic API Key is not configured.".to_string();
        }

        let key_header = match HeaderValue::from_str(trimmed_key) {
            Ok(v) => v,
            Err(_) => return "Invalid Anthropic API Key format.".to_string(),
        };

        let payload = json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1000,
            "messages": [{"role": "user", "content": prompt}]
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-api-key", key_header);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        match self.client.post("https://api.anthropic.com/v1/messages").headers(headers).json(&payload).send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    if let Ok(data) = res.json::<Value>().await {
                        if let Some(text) = data["content"][0]["text"].as_str() {
                            return text.to_string();
                        }
                    }
                }
                format!("Anthropic Error: {}", status)
            }
            Err(e) => format!("Anthropic Network Error: {}", e),
        }
    }

    async fn ask_ollama(&self, prompt: &str, url: &str) -> String {
        let base_url = if url.trim().is_empty() { "http://localhost:11434" } else { url.trim_end_matches('/') };
        let payload = json!({
            "model": "llama3.1",
            "prompt": prompt,
            "stream": false
        });

        match self.client.post(format!("{}/api/generate", base_url)).json(&payload).send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    if let Ok(data) = res.json::<Value>().await {
                        if let Some(text) = data["response"].as_str() {
                            return text.to_string();
                        }
                    }
                }
                format!("Ollama Error: {}", status)
            }
            Err(e) => format!("Ollama Network Error: {}", e),
        }
    }

    pub async fn upload_to_mclogs(&self, log_content: &str) -> Option<String> {
        if log_content.len() > 10 * 1024 * 1024 {
            return None;
        }

        let body = format!("content={}", urlencoding::encode(log_content));
        if let Ok(res) = self.client.post("https://api.mclo.gs/1/log")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await 
        {
            if let Ok(data) = res.json::<Value>().await {
                if data["success"].as_bool().unwrap_or(false) {
                    return data["url"].as_str().map(|s| s.to_string());
                }
            }
        }
        None
    }

    pub async fn search_modrinth(&self, query: &str, project_type: &str, loader: &str, game_version: &str, category: &str, sort_index: &str, offset: i32) -> Result<Value, String> {
        let mut url = match Url::parse("https://api.modrinth.com/v2/search") {
            Ok(u) => u,
            Err(e) => return Err(e.to_string()),
        };

        let index_to_use = if query.trim().is_empty() && (sort_index == "relevance" || sort_index.is_empty()) {
            "downloads"
        } else {
            match sort_index {
                "downloads" | "newest" | "updated" | "follows" => sort_index,
                _ => "relevance",
            }
        };

        let mut facets: Vec<Vec<String>> = Vec::new();
        if !project_type.is_empty() {
            facets.push(vec![format!("project_type:{}", project_type)]);
        }
        if !loader.is_empty() && project_type == "mod" {
            facets.push(vec![format!("categories:{}", loader)]);
        }
        if !game_version.is_empty() {
            facets.push(vec![format!("versions:{}", game_version)]);
        }
        if !category.is_empty() {
            facets.push(vec![format!("categories:{}", category)]);
        }

        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("limit", "12");
            pairs.append_pair("offset", &offset.to_string());
            pairs.append_pair("index", index_to_use);

            if !query.trim().is_empty() {
                pairs.append_pair("query", query.trim());
            }

            if !facets.is_empty() {
                let facets_json = serde_json::to_string(&facets).unwrap_or_default();
                pairs.append_pair("facets", &facets_json);
            }
        }

        let res = self.client.get(url)
            .headers(self.get_default_headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("Modrinth API error {}: {}", status, body));
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mut hits = Vec::new();

        if let Some(items) = data["hits"].as_array() {
            for item in items {
                hits.push(json!({
                    "project_id": item["project_id"],
                    "title": item["title"],
                    "author": item["author"],
                    "description": item["description"],
                    "icon_url": item["icon_url"].as_str().unwrap_or(""),
                    "downloads": item["downloads"].as_i64().unwrap_or(0),
                    "follows": item["follows"].as_i64().unwrap_or(0),
                    "categories": item["categories"],
                    "provider": "modrinth"
                }));
            }
        }

        Ok(json!({ "success": true, "hits": hits }))
    }

    pub async fn get_modrinth_details(&self, project_id: &str, loader: &str, game_version: &str) -> Result<Value, String> {
        let det_url = format!("https://api.modrinth.com/v2/project/{}", project_id);
        let det_res = self.client.get(&det_url).headers(self.get_default_headers()).send().await.map_err(|e| e.to_string())?;
        if !det_res.status().is_success() {
            return Err("Project not found on Modrinth.".to_string());
        }
        let details: Value = det_res.json().await.map_err(|e| e.to_string())?;

        let ver_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        let ver_res = self.client.get(&ver_url).headers(self.get_default_headers()).send().await.map_err(|e| e.to_string())?;
        let versions_raw: Value = if ver_res.status().is_success() {
            ver_res.json().await.unwrap_or(json!([]))
        } else {
            json!([])
        };

        let mut valid_versions = Vec::new();
        if let Some(arr) = versions_raw.as_array() {
            for v in arr {
                let matches_loader = loader.is_empty() || v["loaders"].as_array().map_or(false, |l| l.iter().any(|val| val == loader));
                let matches_mc = game_version.is_empty() || v["game_versions"].as_array().map_or(false, |g| g.iter().any(|val| val == game_version));

                if matches_loader && matches_mc {
                    valid_versions.push(json!({
                        "id": v["id"],
                        "version_number": v["version_number"],
                        "name": v["name"],
                        "date": v["date_published"],
                        "changelog": v["changelog"],
                        "files": v["files"]
                    }));
                }
            }
        }

        Ok(json!({
            "success": true,
            "details": {
                "body": details["body"].as_str().unwrap_or(""),
                "gallery": details["gallery"]
            },
            "versions": valid_versions
        }))
    }

    pub async fn search_curseforge(&self, query: &str, project_type: &str, loader: &str, game_version: &str, category: &str, sort_index: &str, offset: i32) -> Result<Value, String> {
        let headers = self.get_cf_headers();
        if !headers.contains_key("x-api-key") {
            return Err("CurseForge API Key is not configured. Please add it in Settings.".to_string());
        }

        let class_id = match project_type {
            "resourcepack" => 12,
            "shader" => 6552,
            _ => 6,
        };

        let sort_field = match sort_index {
            "downloads" => 2,
            "updated" | "newest" => 3,
            _ => 1,
        };

        let modloader_type = match loader {
            "forge" => 1,
            "fabric" => 4,
            "quilt" => 5,
            "neoforge" => 6,
            _ => 0,
        };

        let category_id = match category {
            "technology" => 412,
            "magic" => 406,
            "utility" => 424,
            "worldgen" => 408,
            "optimization" => 436,
            _ => 0,
        };

        let mut query_params = vec![
            ("gameId", "432".to_string()),
            ("classId", class_id.to_string()),
            ("sortField", sort_field.to_string()),
            ("sortOrder", "desc".to_string()),
            ("pageSize", "12".to_string()),
            ("index", offset.to_string()),
        ];

        if !query.is_empty() { query_params.push(("searchFilter", query.to_string())); }
        if !game_version.is_empty() { query_params.push(("gameVersion", game_version.to_string())); }
        if modloader_type != 0 && project_type == "mod" { query_params.push(("modLoaderType", modloader_type.to_string())); }
        if category_id != 0 { query_params.push(("categoryId", category_id.to_string())); }

        let query_str = query_params.iter().map(|(k, v)| format!("{}={}", k, urlencoding::encode(v))).collect::<Vec<_>>().join("&");
        let url = format!("https://api.curseforge.com/v1/mods/search?{}", query_str);

        let res = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().as_u16() == 403 {
            return Err("Invalid CurseForge API Key. Please check your settings.".to_string());
        }
        if res.status().as_u16() == 404 {
            return Ok(json!({ "success": true, "hits": [] }));
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mut hits = Vec::new();

        if let Some(items) = data["data"].as_array() {
            for item in items {
                let author = item["authors"].as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or("Unknown");

                let icon_url = item["logo"]["thumbnailUrl"].as_str()
                    .or_else(|| item["logo"]["url"].as_str())
                    .unwrap_or("");

                let categories: Vec<&str> = item["categories"].as_array()
                    .map(|c| c.iter().filter_map(|cat| cat["name"].as_str()).collect())
                    .unwrap_or_default();

                hits.push(json!({
                    "project_id": item["id"].as_i64().unwrap_or(0).to_string(),
                    "title": item["name"],
                    "author": author,
                    "description": item["summary"],
                    "icon_url": icon_url,
                    "downloads": item["downloadCount"].as_i64().unwrap_or(0),
                    "follows": 0,
                    "categories": categories,
                    "provider": "curseforge"
                }));
            }
        }

        Ok(json!({ "success": true, "hits": hits }))
    }

    pub async fn get_curseforge_details(&self, project_id: &str, loader: &str, game_version: &str) -> Result<Value, String> {
        let headers = self.get_cf_headers();
        if !headers.contains_key("x-api-key") {
            return Err("CurseForge API Key is not configured.".to_string());
        }

        let url = format!("https://api.curseforge.com/v1/mods/{}", project_id);
        let res = self.client.get(&url).headers(headers.clone()).send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err("Project not found on CurseForge.".to_string());
        }
        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mod_data = &data["data"];

        let mut files_query = Vec::new();
        if !game_version.is_empty() {
            files_query.push(format!("gameVersion={}", urlencoding::encode(game_version)));
        }
        let modloader_type = match loader {
            "forge" => 1,
            "fabric" => 4,
            "quilt" => 5,
            "neoforge" => 6,
            _ => 0,
        };
        if modloader_type != 0 {
            files_query.push(format!("modLoaderType={}", modloader_type));
        }

        let files_url = format!("https://api.curseforge.com/v1/mods/{}/files?{}", project_id, files_query.join("&"));
        let files_res = self.client.get(&files_url).headers(headers).send().await.map_err(|e| e.to_string())?;
        let files_data: Value = if files_res.status().is_success() {
            files_res.json().await.unwrap_or(json!({ "data": [] }))
        } else {
            json!({ "data": [] })
        };

        let mut valid_versions = Vec::new();
        if let Some(files) = files_data["data"].as_array() {
            for f in files {
                let id_str = f["id"].as_i64().unwrap_or(0).to_string();
                let dl_url = f["downloadUrl"].as_str().map(|s| s.to_string()).unwrap_or_else(|| {
                    if id_str.len() >= 4 {
                        format!("https://edge.forgecdn.net/files/{}/{}/{}", &id_str[..4], &id_str[4..], f["fileName"].as_str().unwrap_or(""))
                    } else {
                        String::new()
                    }
                });

                valid_versions.push(json!({
                    "id": id_str,
                    "version_number": f["displayName"],
                    "name": f["fileName"],
                    "date": f["fileDate"],
                    "changelog": "",
                    "files": [{ "filename": f["fileName"], "url": dl_url, "primary": true }]
                }));
            }
        }

        Ok(json!({
            "success": true,
            "details": {
                "body": mod_data["summary"].as_str().unwrap_or(""),
                "gallery": mod_data["screenshots"]
            },
            "versions": valid_versions
        }))
    }

    pub async fn kip_auth_login(&self, username: &str, password: &str) -> Result<Value, String> {
        let payload = json!({
            "type": "kip_login",
            "username": username,
            "password": password
        });

        let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
        match self.client.post(&target_url).headers(self.get_default_headers()).json(&payload).send().await {
            Ok(res) => {
                let data: Value = res.json().await.unwrap_or_default();
                if data["success"].as_bool().unwrap_or(false) {
                    Ok(json!({
                        "success": true,
                        "token": data["token"].as_str().unwrap_or(""),
                        "username": data["username"].as_str().unwrap_or(username)
                    }))
                } else {
                    let msg = data["msg"].as_str().unwrap_or("Invalid credentials.");
                    Ok(json!({ "success": false, "msg": msg }))
                }
            }
            Err(_) => {
                let fallback_token = format!("kip_local_{}", uuid::Uuid::new_v4().simple());
                Ok(json!({ "success": true, "token": fallback_token, "username": username }))
            }
        }
    }

    pub async fn kip_auth_register(&self, username: &str, email: &str, password: &str) -> Result<Value, String> {
        let payload = json!({
            "type": "kip_register",
            "username": username,
            "email": email,
            "password": password
        });

        let target_url = format!("{}/", config::CLOUDFLARE_URL.trim_end_matches('/'));
        match self.client.post(&target_url).headers(self.get_default_headers()).json(&payload).send().await {
            Ok(res) => {
                let data: Value = res.json().await.unwrap_or_default();
                if data["success"].as_bool().unwrap_or(false) {
                    Ok(json!({
                        "success": true,
                        "token": data["token"].as_str().unwrap_or(""),
                        "username": data["username"].as_str().unwrap_or(username)
                    }))
                } else {
                    let msg = data["msg"].as_str().unwrap_or("Registration failed.");
                    Ok(json!({ "success": false, "msg": msg }))
                }
            }
            Err(_) => {
                let fallback_token = format!("kip_local_{}", uuid::Uuid::new_v4().simple());
                Ok(json!({ "success": true, "token": fallback_token, "username": username }))
            }
        }
    }

    pub async fn get_ptero_server_status(&self, panel_url: &str, api_key: &str) -> Result<Value, String> {
        if api_key.trim().is_empty() || panel_url.trim().is_empty() {
            return Err("Missing credentials".to_string());
        }

        let auth_val = match HeaderValue::from_str(&format!("Bearer {}", api_key.trim())) {
            Ok(v) => v,
            Err(_) => return Err("Invalid API key format".to_string()),
        };

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth_val);
        headers.insert(ACCEPT, HeaderValue::from_static("Application/vnd.pterodactyl.v1+json"));

        let url = format!("{}/api/client", panel_url.trim_end_matches('/'));
        let res = self.client.get(&url).headers(headers.clone()).send().await.map_err(|e| e.to_string())?;
        
        if !res.status().is_success() {
            return Err(format!("Pterodactyl error: {}", res.status()));
        }

        let data: Value = res.json().await.map_err(|e| e.to_string())?;
        let mut servers = Vec::new();

        if let Some(items) = data["data"].as_array() {
            for srv in items {
                let cid = srv["attributes"]["identifier"].as_str().unwrap_or("");
                let name = srv["attributes"]["name"].as_str().unwrap_or("");
                
                let res_url = format!("{}/api/client/servers/{}/resources", panel_url.trim_end_matches('/'), cid);
                let state = match self.client.get(&res_url).headers(headers.clone()).send().await {
                    Ok(r) if r.status().is_success() => {
                        if let Ok(res_data) = r.json::<Value>().await {
                            res_data["attributes"]["current_state"].as_str().unwrap_or("offline").to_string()
                        } else {
                            "offline".to_string()
                        }
                    }
                    _ => "offline".to_string(),
                };

                servers.push(json!({
                    "id": cid,
                    "name": name,
                    "state": state
                }));
            }
        }

        Ok(json!(servers))
    }

    pub async fn send_ptero_power_action(&self, panel_url: &str, server_id: &str, action: &str, api_key: &str) -> Result<bool, String> {
        if api_key.trim().is_empty() || panel_url.trim().is_empty() || server_id.trim().is_empty() {
            return Ok(false);
        }

        let auth_val = match HeaderValue::from_str(&format!("Bearer {}", api_key.trim())) {
            Ok(v) => v,
            Err(_) => return Ok(false),
        };

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth_val);
        headers.insert(ACCEPT, HeaderValue::from_static("Application/vnd.pterodactyl.v1+json"));

        let url = format!("{}/api/client/servers/{}/power", panel_url.trim_end_matches('/'), server_id);
        let payload = json!({ "signal": action });
        let res = self.client.post(&url).headers(headers).json(&payload).send().await.map_err(|e| e.to_string())?;
        Ok(res.status().is_success())
    }
}