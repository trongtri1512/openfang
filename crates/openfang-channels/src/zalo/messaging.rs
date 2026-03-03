//! Zalo messaging — send/receive text, images, stickers, files.
//! Based on zca-js v2 protocol: uses dynamic service map URLs + encrypted params.

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Thread type for Zalo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThreadType {
    /// Direct message (1:1)
    User = 0,
    /// Group chat
    Group = 1,
}

/// Zalo service map — dynamic URLs obtained after login.
/// Based on `zpw_service_map_v3` response from `getLoginInfo`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZaloServiceMap {
    /// Chat API endpoints (message sending for User threads)
    #[serde(default)]
    pub chat: Vec<String>,
    /// Group API endpoints
    #[serde(default)]
    pub group: Vec<String>,
    /// Group poll endpoints
    #[serde(default)]
    pub group_poll: Vec<String>,
    /// File upload endpoints
    #[serde(default)]
    pub file: Vec<String>,
    /// Friend endpoints
    #[serde(default)]
    pub friend: Vec<String>,
    /// Profile endpoints
    #[serde(default)]
    pub profile: Vec<String>,
    /// Sticker endpoints
    #[serde(default)]
    pub sticker: Vec<String>,
    /// Reaction endpoints
    #[serde(default)]
    pub reaction: Vec<String>,
    /// Conversation endpoints
    #[serde(default)]
    pub conversation: Vec<String>,
}

impl ZaloServiceMap {
    /// Parse from zpw_service_map_v3 JSON value (as returned by login).
    pub fn from_login_data(map: &serde_json::Value) -> Self {
        let get_urls = |key: &str| -> Vec<String> {
            map[key]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };
        Self {
            chat: get_urls("chat"),
            group: get_urls("group"),
            group_poll: get_urls("group_poll"),
            file: get_urls("file"),
            friend: get_urls("friend"),
            profile: get_urls("profile"),
            sticker: get_urls("sticker"),
            reaction: get_urls("reaction"),
            conversation: get_urls("conversation"),
        }
    }

    /// Get the best chat URL.
    pub fn chat_url(&self) -> &str {
        self.chat
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best group URL.
    pub fn group_url(&self) -> &str {
        self.group
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best reaction URL.
    pub fn reaction_url(&self) -> &str {
        self.reaction
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }
}

/// Zalo messaging client — uses dynamic service map from login.
pub struct ZaloMessaging {
    client: reqwest::Client,
    /// Dynamic service map from login response
    service_map: ZaloServiceMap,
    /// Secret key from login (zpw_enk)
    secret_key: Option<String>,
    /// User's UID
    uid: Option<String>,
    /// API version params
    zpw_ver: u32,
    zpw_type: u32,
}

impl ZaloMessaging {
    /// Create with default endpoints.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            service_map: ZaloServiceMap::default(),
            secret_key: None,
            uid: None,
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Set login credentials after successful authentication.
    pub fn set_login_info(&mut self, uid: &str, secret_key: Option<&str>) {
        self.uid = Some(uid.to_string());
        self.secret_key = secret_key.map(String::from);
    }

    /// Update service map (e.g., after login).
    pub fn set_service_map(&mut self, map: ZaloServiceMap) {
        self.service_map = map;
    }

    /// Add API version query params to a URL.
    fn add_api_params(&self, base: &str) -> String {
        if base.contains('?') {
            format!(
                "{}&zpw_ver={}&zpw_type={}",
                base, self.zpw_ver, self.zpw_type
            )
        } else {
            format!(
                "{}?zpw_ver={}&zpw_type={}",
                base, self.zpw_ver, self.zpw_type
            )
        }
    }

    /// Send a text message (works for both User and Group threads).
    pub async fn send_text(
        &self,
        thread_id: &str,
        thread_type: ThreadType,
        content: &str,
        cookie: &str,
    ) -> Result<String, String> {
        let base_url = if thread_type == ThreadType::User {
            format!("{}/api/message", self.service_map.chat_url())
        } else {
            format!("{}/api/group", self.service_map.group_url())
        };

        let endpoint = self.add_api_params(&format!("{}?nretry=0", base_url));

        let params = serde_json::json!({
            "toid": thread_id,
            "msg": content,
            "clientId": generate_client_id(),
        });

        let response = self
            .client
            .post(&endpoint)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Send message failed: {e}"))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Invalid send response: {e}"))?;

        let error_code = body["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(format!(
                "Send failed: {} - {}",
                error_code,
                body["error_message"].as_str().unwrap_or("unknown")
            ));
        }

        let msg_id = body["data"]["msgId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        debug!("Sent message {} to {}", msg_id, thread_id);
        Ok(msg_id)
    }

    /// Send a reaction to a message.
    pub async fn send_reaction(
        &self,
        msg_id: &str,
        thread_id: &str,
        reaction: &str,
        cookie: &str,
    ) -> Result<(), String> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
            "rType": reaction,
        });

        let endpoint = self.add_api_params(&format!(
            "{}/api/message/reaction",
            self.service_map.reaction_url()
        ));

        self.client
            .post(&endpoint)
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Reaction failed: {e}"))?;

        Ok(())
    }

    /// Undo (recall) a message.
    pub async fn undo_message(&self, msg_id: &str, thread_id: &str, cookie: &str) -> Result<(), String> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
        });

        let endpoint =
            self.add_api_params(&format!("{}/api/message/undo", self.service_map.chat_url()));

        self.client
            .post(&endpoint)
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Undo message failed: {e}"))?;

        Ok(())
    }

    /// Get current service map info (for debugging).
    pub fn service_info(&self) -> serde_json::Value {
        serde_json::json!({
            "chat_url": self.service_map.chat_url(),
            "group_url": self.service_map.group_url(),
            "has_secret_key": self.secret_key.is_some(),
            "uid": self.uid.as_deref().unwrap_or("not set"),
            "zpw_ver": self.zpw_ver,
        })
    }
}

impl Default for ZaloMessaging {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a client-side message ID.
fn generate_client_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("cli_{}", ts % 9_999_999_999)
}
