//! Zalo channel module — native Rust implementation.
//!
//! Connects to Zalo via reverse-engineered Web API (based on zca-js protocol).
//! Supports cookie login and QR login. No CLI or Node.js dependency.

pub mod auth;
pub mod messaging;
pub mod session;

use crate::types::{
    ChannelAdapter, ChannelContent, ChannelMessage, ChannelStatus, ChannelType, ChannelUser,
};
use async_trait::async_trait;
use chrono::Utc;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use self::auth::{ZaloAuth, ZaloCredentials};
use self::messaging::{ThreadType as ZaloThreadType, ZaloMessaging, ZaloServiceMap};
use self::session::SessionManager;

const MAX_MESSAGE_LEN: usize = 4096;

/// Zalo channel adapter — native Rust HTTP client.
#[allow(dead_code)]
pub struct ZaloAdapter {
    /// Authentication handler
    auth: ZaloAuth,
    /// Messaging client (uses dynamic service map from login)
    messaging: ZaloMessaging,
    /// Session state manager
    session: SessionManager,
    /// Cookie for API calls
    cookie: Option<String>,
    /// Cookie file path (read cookie from file on connect)
    cookie_path: String,
    /// Default agent name for routing
    default_agent: Option<String>,
    /// Shutdown signal
    shutdown_tx: Arc<watch::Sender<bool>>,
    shutdown_rx: watch::Receiver<bool>,
    /// Connection state
    connected: Arc<AtomicBool>,
    /// Message counters
    messages_received: Arc<AtomicU64>,
    messages_sent: Arc<AtomicU64>,
}

impl ZaloAdapter {
    /// Create a new native Zalo adapter.
    pub fn new(
        cookie_path: String,
        imei: Option<String>,
        user_agent: Option<String>,
        default_agent: Option<String>,
    ) -> Self {
        let creds = ZaloCredentials {
            imei: imei.unwrap_or_else(auth::generate_imei),
            cookie: None,
            user_agent: user_agent.unwrap_or_else(|| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0"
                    .into()
            }),
        };
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            auth: ZaloAuth::new(creds),
            messaging: ZaloMessaging::new(),
            session: SessionManager::new(),
            cookie: None,
            cookie_path,
            default_agent,
            shutdown_tx: Arc::new(shutdown_tx),
            shutdown_rx,
            connected: Arc::new(AtomicBool::new(false)),
            messages_received: Arc::new(AtomicU64::new(0)),
            messages_sent: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Login with cookie from config or parameter.
    #[allow(dead_code)]
    async fn login_cookie(&mut self, cookie: &str) -> Result<(), String> {
        let login_data = self.auth.login_with_cookie(cookie).await?;

        // Apply service map to messaging client
        if let Some(ref map) = login_data.zpw_service_map_v3 {
            let service_map = ZaloServiceMap::from_login_data(map);
            self.messaging.set_service_map(service_map);
            info!("Zalo: service map applied from login response");
        }

        // Set login credentials for messaging
        self.messaging
            .set_login_info(&login_data.uid, login_data.zpw_enk.as_deref());

        self.session
            .set_session(
                login_data.uid.clone(),
                login_data.zpw_enk,
                login_data.zpw_key,
            )
            .await;
        self.cookie = Some(cookie.to_string());
        info!("Zalo logged in: uid={}", login_data.uid);
        Ok(())
    }

    /// Try to load cookie from cookie_path file.
    fn try_load_cookie(&self) -> Result<Option<String>, String> {
        if self.cookie_path.is_empty() {
            return Ok(None);
        }

        // Expand ~ to home dir
        let expanded = if self.cookie_path.starts_with("~/") {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(&self.cookie_path[2..]))
                .unwrap_or_else(|| std::path::PathBuf::from(&self.cookie_path))
        } else {
            std::path::PathBuf::from(&self.cookie_path)
        };

        if expanded.exists() {
            let content = std::fs::read_to_string(&expanded)
                .map_err(|e| format!("Failed to read cookie file: {e}"))?;

            let trimmed = content.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }

            // Support JSON format {"cookie": "..."} or raw cookie string
            if trimmed.starts_with('{') {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if let Some(cookie) = json["cookie"].as_str() {
                        return Ok(Some(cookie.to_string()));
                    }
                }
            }

            Ok(Some(trimmed.to_string()))
        } else {
            info!("Zalo: cookie file not found at {:?}", expanded);
            Ok(None)
        }
    }

    /// Get QR code for login.
    pub async fn get_qr_code(&mut self) -> Result<auth::QrCodeResult, String> {
        self.auth.get_qr_code().await
    }
}

#[async_trait]
impl ChannelAdapter for ZaloAdapter {
    fn name(&self) -> &str {
        "zalo"
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Custom("zalo".to_string())
    }

    async fn start(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = ChannelMessage> + Send>>, Box<dyn std::error::Error>>
    {
        info!("Zalo adapter: connecting in native mode...");
        warn!("⚠️  Zalo Personal API is unofficial. Use at your own risk.");

        // Try cookie login: from cookie_path file
        let cookie = self.try_load_cookie()?;
        if let Some(ref cookie_str) = cookie {
            // We can't mutate &self in start(), so we use a separate reqwest client
            let creds = self.auth.credentials().clone();
            let temp_auth = ZaloAuth::new(creds);
            let login_data = temp_auth.login_with_cookie(cookie_str).await
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

            info!("Zalo Personal: connected via cookie auth (uid={})", login_data.uid);
            self.connected.store(true, Ordering::Relaxed);
        } else {
            warn!("Zalo: No cookie found at '{}'. Configure cookie_path in config.toml or use QR login via admin dashboard.", self.cookie_path);
        }

        // For now, return a pending stream — messages will be received via webhook/polling
        // The WebSocket listener requires zpw_enk encryption which is complex to implement
        info!("Zalo listener: active (webhook/polling mode)");
        let (tx, rx) = tokio::sync::mpsc::channel::<ChannelMessage>(256);

        // Keep sender alive — in future, WebSocket listener will push messages here
        let connected = self.connected.clone();
        let mut shutdown_rx = self.shutdown_rx.clone();
        tokio::spawn(async move {
            let _ = shutdown_rx.changed().await;
            connected.store(false, Ordering::Relaxed);
            drop(tx);
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn send(
        &self,
        user: &ChannelUser,
        content: ChannelContent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = match &content {
            ChannelContent::Text(t) => t.clone(),
            ChannelContent::Image { caption, .. } => caption
                .clone()
                .unwrap_or_else(|| "(Image — not supported yet)".to_string()),
            ChannelContent::File { filename, .. } => {
                format!("(File: {filename} — not supported yet)")
            }
            _ => "(Unsupported content type)".to_string(),
        };

        let cookie = self
            .cookie
            .as_ref()
            .ok_or("Zalo not logged in — no cookie available")?;

        // Split long messages
        let chunks = crate::types::split_message(&text, MAX_MESSAGE_LEN);
        for chunk in chunks {
            self.messaging
                .send_text(
                    &user.platform_id,
                    ZaloThreadType::User,
                    chunk,
                    cookie,
                )
                .await
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        }

        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn send_typing(&self, _user: &ChannelUser) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Zalo: typing indicator not supported by API");
        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.shutdown_tx.send(true);
        self.session.invalidate().await;
        self.connected.store(false, Ordering::Relaxed);
        info!("Zalo adapter: disconnected");
        Ok(())
    }

    fn status(&self) -> ChannelStatus {
        ChannelStatus {
            connected: self.connected.load(Ordering::Relaxed),
            started_at: None,
            last_message_at: None,
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            last_error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_adapter_creation() {
        let adapter = ZaloAdapter::new(
            "~/.openfang/zalo-cookie.txt".to_string(),
            None,
            None,
            None,
        );
        assert_eq!(adapter.name(), "zalo");
        assert_eq!(
            adapter.channel_type(),
            ChannelType::Custom("zalo".to_string())
        );
    }

    #[test]
    fn test_imei_generation() {
        let imei = auth::generate_imei();
        assert_eq!(imei.len(), 12);
        assert!(imei.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_extract_login_version() {
        let html = r#"<script src="https://stc-zlogin.zdn.vn/main-2.44.10.js"></script>"#;
        assert_eq!(
            auth::extract_login_version(html),
            Some("2.44.10".to_string())
        );
    }

    #[test]
    fn test_extract_login_version_not_found() {
        let html = "<html><body>no version here</body></html>";
        assert_eq!(auth::extract_login_version(html), None);
    }

    #[test]
    fn test_service_map_from_login_data() {
        let map = serde_json::json!({
            "chat": ["https://chat1.zalo.me", "https://chat2.zalo.me"],
            "group": ["https://group1.zalo.me"],
        });
        let sm = messaging::ZaloServiceMap::from_login_data(&map);
        assert_eq!(sm.chat_url(), "https://chat1.zalo.me");
        assert_eq!(sm.group_url(), "https://group1.zalo.me");
    }
}
