use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde_json::{json, Value};

pub struct AuthManager {
    client: Client,
    client_id: String,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            client_id: "c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb".to_string(),
        }
    }

    pub async fn get_device_code(&self) -> Result<Value, String> {
        let body = format!(
            "client_id={}&scope={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode("XboxLive.signin offline_access")
        );
        
        let res = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        res.json::<Value>().await.map_err(|e| e.to_string())
    }

    pub async fn poll_token(&self, device_code: &str) -> Result<Option<Value>, String> {
        let body = format!(
            "grant_type={}&client_id={}&device_code={}",
            urlencoding::encode("urn:ietf:params:oauth:grant-type:device_code"),
            urlencoding::encode(&self.client_id),
            urlencoding::encode(device_code)
        );
        
        let res = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        let status = res.status();
        let data: Value = res.json().await.map_err(|e| e.to_string())?;

        if status.is_success() {
            return Ok(Some(data));
        }

        if let Some(err) = data.get("error").and_then(|v| v.as_str()) {
            if err == "authorization_pending" || err == "slow_down" {
                return Ok(None);
            }
            return Err(format!("OAuth error: {}", err));
        }

        Err("Token polling expired or failed.".to_string())
    }

    pub async fn refresh_token(&self, refresh_token_val: &str) -> Result<Value, String> {
        let body = format!(
            "grant_type=refresh_token&client_id={}&refresh_token={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(refresh_token_val)
        );
        
        let res = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        res.json::<Value>().await.map_err(|e| e.to_string())
    }

    pub async fn authenticate_minecraft(&self, ms_access_token: &str) -> Result<Value, String> {
        let xbl_payload = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms_access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let xbl_res = self.client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("x-xbl-contract-version", "1")
            .json(&xbl_payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !xbl_res.status().is_success() {
            return Err(format!("Xbox Live auth failed with status {}", xbl_res.status()));
        }
        let xbl_data: Value = xbl_res.json().await.map_err(|e| e.to_string())?;
        let xbl_token = xbl_data["Token"].as_str().unwrap_or_default();

        let xsts_payload = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let xsts_res = self.client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("x-xbl-contract-version", "1")
            .json(&xsts_payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !xsts_res.status().is_success() {
            return Err(format!("XSTS auth failed with status {}", xsts_res.status()));
        }
        let xsts_data: Value = xsts_res.json().await.map_err(|e| e.to_string())?;
        
        let xsts_token = xsts_data["Token"]
            .as_str()
            .ok_or_else(|| "Missing XSTS token in response".to_string())?;
            
        let uhs = xsts_data
            .get("DisplayClaims")
            .and_then(|dc| dc.get("xui"))
            .and_then(|xui| xui.as_array())
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("uhs"))
            .and_then(|uhs| uhs.as_str())
            .ok_or_else(|| "Failed to obtain Xbox UserHash (uhs).".to_string())?;

        let mc_payload = json!({
            "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
        });

        let mc_res = self.client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&mc_payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !mc_res.status().is_success() {
            return Err(format!("Minecraft authentication failed: status {}", mc_res.status()));
        }
        let mc_data: Value = mc_res.json().await.map_err(|e| e.to_string())?;
        let mc_access_token = mc_data["access_token"]
            .as_str()
            .ok_or_else(|| "Missing Minecraft access token".to_string())?;

        let prof_res = self.client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Authorization", format!("Bearer {}", mc_access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !prof_res.status().is_success() {
            return Err("No active Minecraft license found on this account.".to_string());
        }
        
        let prof_data: Value = prof_res.json().await.map_err(|e| e.to_string())?;
        
        Ok(json!({
            "access_token": mc_access_token,
            "uuid": prof_data["id"].as_str().unwrap_or_default(),
            "name": prof_data["name"].as_str().unwrap_or_default()
        }))
    }
}