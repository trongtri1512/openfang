//! Zalo authentication — Cookie login, QR login.
//! Based on zca-js (https://github.com/RFS-ADRENO/zca-js) protocol.

use serde::{Deserialize, Serialize};
use tracing::info;

/// Authentication credentials for Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaloCredentials {
    /// IMEI identifier (device fingerprint)
    pub imei: String,
    /// Cookie string from Zalo Web
    pub cookie: Option<String>,
    /// User agent string
    pub user_agent: String,
}

impl Default for ZaloCredentials {
    fn default() -> Self {
        Self {
            imei: generate_imei(),
            cookie: None,
            user_agent:
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0"
                    .into(),
        }
    }
}

/// Login data from Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub uid: String,
    pub zpw_enk: Option<String>,
    pub zpw_key: Option<String>,
    /// Service map v3 — dynamic URLs for each API category
    pub zpw_service_map_v3: Option<serde_json::Value>,
    /// WebSocket URLs for real-time listening
    pub zpw_ws: Option<Vec<String>>,
}

/// QR code generation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResult {
    /// QR scan code identifier
    pub code: String,
    /// Base64 PNG image data
    pub image: String,
}

/// Standard headers matching zca-js browser fingerprint.
fn zalo_headers(user_agent: &str) -> Vec<(&'static str, String)> {
    vec![
        ("accept", "*/*".into()),
        (
            "accept-language",
            "vi-VN,vi;q=0.9,fr-FR;q=0.8,fr;q=0.7,en-US;q=0.6,en;q=0.5".into(),
        ),
        ("content-type", "application/x-www-form-urlencoded".into()),
        ("priority", "u=1, i".into()),
        (
            "sec-ch-ua",
            "\"Chromium\";v=\"130\", \"Google Chrome\";v=\"130\", \"Not?A_Brand\";v=\"99\"".into(),
        ),
        ("sec-ch-ua-mobile", "?0".into()),
        ("sec-ch-ua-platform", "\"Windows\"".into()),
        ("sec-fetch-dest", "empty".into()),
        ("sec-fetch-mode", "cors".into()),
        ("sec-fetch-site", "same-origin".into()),
        (
            "referer",
            "https://id.zalo.me/account?continue=https%3A%2F%2Fzalo.me%2Fpc".into(),
        ),
        ("referrer-policy", "strict-origin-when-cross-origin".into()),
        ("user-agent", user_agent.into()),
    ]
}

/// Zalo login methods.
pub struct ZaloAuth {
    credentials: ZaloCredentials,
    client: reqwest::Client,
    /// Cached login page version (e.g., "2.44.10")
    login_version: Option<String>,
}

impl ZaloAuth {
    pub fn new(credentials: ZaloCredentials) -> Self {
        Self {
            credentials,
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap_or_default(),
            login_version: None,
        }
    }

    /// Login with cookie (fastest method).
    pub async fn login_with_cookie(&self, cookie: &str) -> Result<LoginData, String> {
        info!("Zalo auth: logging in with cookie...");

        // Validate cookie format
        if !cookie.contains("zpw_sek") {
            return Err("Invalid Zalo cookie: must contain zpw_sek".into());
        }

        // Step 1: Login to get user info + secret key + service map
        let login_response = self
            .client
            .get("https://wpa.chat.zalo.me/api/login/getLoginInfo")
            .header("cookie", cookie)
            .header("user-agent", &self.credentials.user_agent)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .send()
            .await
            .map_err(|e| format!("Login request failed: {e}"))?;

        let login_body: serde_json::Value = login_response
            .json()
            .await
            .map_err(|e| format!("Invalid login response: {e}"))?;

        let login_error = login_body["error_code"].as_i64().unwrap_or(-1);
        if login_error != 0 {
            return Err(format!(
                "Login failed with error code: {} - {}",
                login_error,
                login_body["error_message"].as_str().unwrap_or("unknown")
            ));
        }

        let login_data = &login_body["data"];

        // Step 2: Get server info
        let _ = self
            .client
            .get("https://wpa.chat.zalo.me/api/login/getServerInfo")
            .header("cookie", cookie)
            .header("user-agent", &self.credentials.user_agent)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .query(&[("imei", &self.credentials.imei)])
            .query(&[("type", "30")])
            .query(&[("client_version", "671")])
            .query(&[("computer_name", "Web")])
            .send()
            .await
            .map_err(|e| format!("Get server info failed: {e}"))?;

        // Parse zpw_ws array
        let zpw_ws = login_data["zpw_ws"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        Ok(LoginData {
            uid: login_data["uid"].as_str().unwrap_or("").into(),
            zpw_enk: login_data["zpw_enk"].as_str().map(String::from),
            zpw_key: login_data["zpw_key"].as_str().map(String::from),
            zpw_service_map_v3: login_data["zpw_service_map_v3"]
                .as_object()
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            zpw_ws,
        })
    }

    // ─── ZCA-JS QR LOGIN FLOW ────────────────────────────

    /// Step 1: Load login page to get JS version number.
    async fn load_login_page(&mut self) -> Result<String, String> {
        info!("Zalo QR: loading login page...");

        let response = self
            .client
            .get("https://id.zalo.me/account?continue=https%3A%2F%2Fchat.zalo.me%2F")
            .header(
                "accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("accept-language", "vi-VN,vi;q=0.9,en-US;q=0.6,en;q=0.5")
            .header("user-agent", &self.credentials.user_agent)
            .header("referer", "https://chat.zalo.me/")
            .header("sec-fetch-dest", "document")
            .header("sec-fetch-mode", "navigate")
            .header("sec-fetch-site", "same-site")
            .header("upgrade-insecure-requests", "1")
            .send()
            .await
            .map_err(|e| format!("Failed to load login page: {e}"))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read login page: {e}"))?;

        // Extract version: https://stc-zlogin.zdn.vn/main-X.Y.Z.js
        // Use simple string search instead of regex to avoid regex dependency
        let version = extract_login_version(&html)
            .ok_or_else(|| "Cannot get Zalo login version from page. API may have changed.".to_string())?;

        info!("Zalo QR: got login version: {}", version);
        self.login_version = Some(version.clone());
        Ok(version)
    }

    /// Step 2: Get login info (sets session cookies).
    async fn get_login_info(&self, version: &str) -> Result<(), String> {
        let form = format!("continue=https%3A%2F%2Fzalo.me%2Fpc&v={}", version);

        let mut req = self
            .client
            .post("https://id.zalo.me/account/logininfo")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        req.send()
            .await
            .map_err(|e| format!("logininfo failed: {e}"))?;

        Ok(())
    }

    /// Step 3: Verify client (device verification).
    async fn verify_client(&self, version: &str) -> Result<(), String> {
        let form = format!(
            "type=device&continue=https%3A%2F%2Fzalo.me%2Fpc&v={}",
            version
        );

        let mut req = self
            .client
            .post("https://id.zalo.me/account/verify-client")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        req.send()
            .await
            .map_err(|e| format!("verify-client failed: {e}"))?;

        Ok(())
    }

    /// Step 4: Generate QR code.
    async fn generate_qr(&self, version: &str) -> Result<QrCodeResult, String> {
        let form = format!("continue=https%3A%2F%2Fzalo.me%2Fpc&v={}", version);

        let mut req = self
            .client
            .post("https://id.zalo.me/account/authen/qr/generate")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("QR generate failed: {e}"))?;

        let text = response
            .text()
            .await
            .map_err(|e| format!("QR generate read error: {e}"))?;

        let data: serde_json::Value = serde_json::from_str(&text).map_err(|_| {
            format!(
                "QR generate returned non-JSON. First 200 chars: {}",
                text.chars().take(200).collect::<String>()
            )
        })?;

        let error_code = data["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(format!(
                "QR generate error {}: {}",
                error_code,
                data["error_message"].as_str().unwrap_or("unknown")
            ));
        }

        let code = data["data"]["code"]
            .as_str()
            .ok_or_else(|| "No 'code' in QR response".to_string())?;
        let image = data["data"]["image"]
            .as_str()
            .ok_or_else(|| "No 'image' in QR response".to_string())?;

        Ok(QrCodeResult {
            code: code.to_string(),
            image: image.to_string(),
        })
    }

    /// Full QR code generation: load page → get session → verify → generate.
    pub async fn get_qr_code(&mut self) -> Result<QrCodeResult, String> {
        info!("Zalo auth: starting QR login flow (zca-js protocol)...");

        let version = self.load_login_page().await?;
        self.get_login_info(&version).await?;
        self.verify_client(&version).await?;
        let qr = self.generate_qr(&version).await?;

        info!("Zalo QR: generated successfully (code={})", qr.code);
        Ok(qr)
    }

    /// Wait for QR scan (long-polling).
    pub async fn wait_for_scan(&self, code: &str) -> Result<(String, String), String> {
        let version = self.login_version.as_deref().unwrap_or("2.44.10");

        loop {
            let form = format!(
                "code={}&continue=https%3A%2F%2Fchat.zalo.me%2F&v={}",
                code, version
            );

            let mut req = self
                .client
                .post("https://id.zalo.me/account/authen/qr/waiting-scan")
                .body(form);

            for (k, v) in zalo_headers(&self.credentials.user_agent) {
                req = req.header(k, v);
            }

            let response = req
                .send()
                .await
                .map_err(|e| format!("waiting-scan failed: {e}"))?;

            let data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("waiting-scan parse error: {e}"))?;

            let error_code = data["error_code"].as_i64().unwrap_or(-1);

            match error_code {
                8 => continue, // Still waiting, retry
                0 => {
                    let avatar = data["data"]["avatar"].as_str().unwrap_or("").to_string();
                    let name = data["data"]["display_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    return Ok((avatar, name));
                }
                -13 => return Err("QR code expired".into()),
                _ => {
                    return Err(format!(
                        "waiting-scan error {}: {}",
                        error_code,
                        data["error_message"].as_str().unwrap_or("unknown")
                    ));
                }
            }
        }
    }

    /// Wait for user confirm on phone (long-polling).
    pub async fn wait_for_confirm(&self, code: &str) -> Result<(), String> {
        let version = self.login_version.as_deref().unwrap_or("2.44.10");

        loop {
            let form = format!(
                "code={}&gToken=&gAction=CONFIRM_QR&continue=https%3A%2F%2Fchat.zalo.me%2F&v={}",
                code, version
            );

            let mut req = self
                .client
                .post("https://id.zalo.me/account/authen/qr/waiting-confirm")
                .body(form);

            for (k, v) in zalo_headers(&self.credentials.user_agent) {
                req = req.header(k, v);
            }

            let response = req
                .send()
                .await
                .map_err(|e| format!("waiting-confirm failed: {e}"))?;

            let data: serde_json::Value = response.json().await.map_err(|e| {
                format!("waiting-confirm parse error: {e}")
            })?;

            let error_code = data["error_code"].as_i64().unwrap_or(-1);

            match error_code {
                8 => continue,
                0 => return Ok(()),
                -13 => return Err("User declined QR login".into()),
                _ => {
                    return Err(format!(
                        "waiting-confirm error {}: {}",
                        error_code,
                        data["error_message"].as_str().unwrap_or("unknown")
                    ));
                }
            }
        }
    }

    /// Get credentials reference.
    pub fn credentials(&self) -> &ZaloCredentials {
        &self.credentials
    }
}

/// Extract login version from HTML without regex dependency.
/// Looks for: https://stc-zlogin.zdn.vn/main-X.Y.Z.js
pub fn extract_login_version(html: &str) -> Option<String> {
    let marker = "stc-zlogin.zdn.vn/main-";
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find(".js")?;
    let version = &rest[..end];
    // Validate: should be digits and dots only
    if version.chars().all(|c| c.is_ascii_digit() || c == '.') && !version.is_empty() {
        Some(version.to_string())
    } else {
        None
    }
}

/// Generate a random IMEI-like device identifier.
pub fn generate_imei() -> String {
    // Use timestamp + process ID for uniqueness without rand dependency
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id() as u128;
    let id = (ts ^ (pid << 32)) % 999_999_999_999;
    format!("{:012}", id)
}
