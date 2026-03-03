//! Zalo personal messaging channel adapter via openzca CLI.
//!
//! Uses the [openzca](https://github.com/trongtri1512/openzca) CLI as a subprocess gateway.
//!
//! - **Incoming**: Spawns `openzca listen --supervised --raw --keep-alive` and parses JSON
//!   lines from stdout into `ChannelMessage` events.
//! - **Outgoing**: Runs `openzca msg send <threadId> <text>` for each reply.
//! - **Typing**: Runs `openzca msg typing <threadId>`.

use crate::types::{
    ChannelAdapter, ChannelContent, ChannelMessage, ChannelStatus, ChannelType, ChannelUser,
};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, warn};

const MAX_MESSAGE_LEN: usize = 4096;

/// Zalo personal messaging adapter using openzca CLI subprocess.
pub struct ZaloAdapter {
    /// Path to the openzca CLI binary.
    cli_path: String,
    /// Optional profile name for multi-account support.
    profile: Option<String>,
    /// Shutdown signal.
    shutdown_tx: Arc<watch::Sender<bool>>,
    shutdown_rx: watch::Receiver<bool>,
    /// Whether the listener is currently running.
    connected: Arc<AtomicBool>,
    /// Message counters for status.
    messages_received: Arc<AtomicU64>,
    messages_sent: Arc<AtomicU64>,
    /// Listener child process PID (for cleanup).
    listener_pid: Arc<std::sync::Mutex<Option<u32>>>,
}

impl ZaloAdapter {
    /// Create a new Zalo adapter.
    pub fn new(cli_path: String, profile: Option<String>) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            cli_path,
            profile,
            shutdown_tx: Arc::new(shutdown_tx),
            shutdown_rx,
            connected: Arc::new(AtomicBool::new(false)),
            messages_received: Arc::new(AtomicU64::new(0)),
            messages_sent: Arc::new(AtomicU64::new(0)),
            listener_pid: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Build the base command with optional --profile flag.
    fn base_cmd(&self) -> Command {
        let mut cmd = Command::new(&self.cli_path);
        if let Some(ref profile) = self.profile {
            cmd.arg("--profile").arg(profile);
        }
        cmd
    }

    /// Send a text message via `openzca msg send`.
    async fn cli_send(
        &self,
        thread_id: &str,
        text: &str,
        is_group: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = self.base_cmd();
        cmd.arg("msg").arg("send").arg(thread_id).arg(text);
        if is_group {
            cmd.arg("--group");
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("openzca msg send failed: {stderr}");
            return Err(format!("openzca msg send failed: {stderr}").into());
        }

        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Parse a JSON line from `openzca listen --raw` into a `ChannelMessage`.
    fn parse_listen_event(json: &serde_json::Value) -> Option<ChannelMessage> {
        // openzca --raw outputs JSON with these key fields:
        // threadId, senderId, content, chatType, msgType, timestamp,
        // senderName (for groups), metadata.*
        let content_str = json.get("content")?.as_str().unwrap_or_default();
        if content_str.is_empty() {
            return None;
        }

        let thread_id = json
            .get("threadId")
            .or_else(|| json.get("conversationId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let sender_id = json
            .get("senderId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        if sender_id.is_empty() || thread_id.is_empty() {
            return None;
        }

        let sender_name = json
            .get("senderName")
            .and_then(|v| v.as_str())
            .unwrap_or(&sender_id)
            .to_string();

        let chat_type = json
            .get("chatType")
            .and_then(|v| v.as_str())
            .unwrap_or("user");
        let is_group = chat_type == "group";

        let msg_type = json
            .get("msgType")
            .and_then(|v| v.as_str())
            .unwrap_or("chat.text");

        // Parse timestamp (milliseconds epoch)
        let timestamp = json
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
            .unwrap_or_else(Utc::now);

        let msg_id = json
            .get("msgId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        // Build content based on message type
        let content = if msg_type.starts_with("chat.photo")
            || msg_type.starts_with("chat.video")
            || msg_type.starts_with("chat.voice")
            || msg_type.starts_with("share.file")
        {
            // Media messages: content already has [media attached: ...] format from openzca
            ChannelContent::Text(content_str.to_string())
        } else {
            ChannelContent::Text(content_str.to_string())
        };

        // Build metadata
        let mut metadata = HashMap::new();
        if let Some(target_id) = json.get("targetId").and_then(|v| v.as_str()) {
            metadata.insert(
                "target_id".to_string(),
                serde_json::Value::String(target_id.to_string()),
            );
        }
        if let Some(quote) = json.get("quote") {
            metadata.insert("quote".to_string(), quote.clone());
        }
        if let Some(mentions) = json.get("mentions") {
            metadata.insert("mentions".to_string(), mentions.clone());
        }
        metadata.insert(
            "msg_type".to_string(),
            serde_json::Value::String(msg_type.to_string()),
        );

        Some(ChannelMessage {
            channel: ChannelType::Custom("zalo".to_string()),
            platform_message_id: msg_id,
            sender: ChannelUser {
                platform_id: if is_group {
                    // For groups, use threadId for routing, senderId in metadata
                    thread_id.clone()
                } else {
                    sender_id.clone()
                },
                display_name: sender_name,
                openfang_user: None,
            },
            content,
            target_agent: None,
            timestamp,
            is_group,
            thread_id: Some(thread_id),
            metadata,
        })
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
        let (tx, rx) = mpsc::channel::<ChannelMessage>(256);
        let mut shutdown_rx = self.shutdown_rx.clone();
        let connected = self.connected.clone();
        let messages_received = self.messages_received.clone();
        let listener_pid = self.listener_pid.clone();

        // Build the listen command
        let mut cmd = self.base_cmd();
        cmd.arg("listen")
            .arg("--supervised")
            .arg("--raw")
            .arg("--keep-alive");

        // Spawn the openzca listener subprocess
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        let mut child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn openzca listener (is openzca installed? `npm install -g openzca@latest`): {e}"
            )
        })?;

        // Store PID for cleanup
        if let Some(pid) = child.id() {
            if let Ok(mut guard) = listener_pid.lock() {
                *guard = Some(pid);
            }
        }

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to capture openzca stdout")?;

        info!("Zalo adapter: openzca listener started");
        connected.store(true, Ordering::Relaxed);

        // Spawn stderr reader (log warnings)
        let stderr = child.stderr.take();
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.is_empty() {
                        debug!("openzca stderr: {line}");
                    }
                }
            });
        }

        // Spawn stdout reader (parse JSON events)
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            loop {
                tokio::select! {
                    line_result = lines.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                if line.trim().is_empty() {
                                    continue;
                                }

                                // Parse JSON
                                let json: serde_json::Value = match serde_json::from_str(&line) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        debug!("openzca: non-JSON line: {e} — {line}");
                                        continue;
                                    }
                                };

                                // Skip lifecycle events (connected, heartbeat, closed, session_id)
                                if let Some(event_type) = json.get("type").and_then(|v| v.as_str()) {
                                    match event_type {
                                        "connected" => {
                                            info!("Zalo adapter: openzca connected to Zalo");
                                            continue;
                                        }
                                        "heartbeat" => {
                                            debug!("Zalo adapter: heartbeat");
                                            continue;
                                        }
                                        "closed" | "error" => {
                                            warn!("Zalo adapter: openzca event: {event_type}");
                                            continue;
                                        }
                                        "session_id" => {
                                            debug!("Zalo adapter: session started");
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }

                                // Parse into ChannelMessage
                                if let Some(msg) = ZaloAdapter::parse_listen_event(&json) {
                                    messages_received.fetch_add(1, Ordering::Relaxed);
                                    if tx.send(msg).await.is_err() {
                                        info!("Zalo adapter: channel closed, stopping listener");
                                        break;
                                    }
                                }
                            }
                            Ok(None) => {
                                info!("Zalo adapter: openzca stdout closed");
                                break;
                            }
                            Err(e) => {
                                error!("Zalo adapter: read error: {e}");
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            info!("Zalo adapter: shutdown signal received");
                            break;
                        }
                    }
                }
            }

            connected.store(false, Ordering::Relaxed);
            // Kill child process
            let _ = child.kill().await;
            info!("Zalo adapter: listener stopped");
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
                .unwrap_or_else(|| "(Image — not supported in Zalo CLI mode)".to_string()),
            ChannelContent::File { filename, .. } => {
                format!("(File: {filename} — not supported in Zalo CLI mode)")
            }
            _ => "(Unsupported content type)".to_string(),
        };

        // Determine if this is a group message from metadata
        // The platform_id is the threadId for groups, senderId for DMs
        let is_group = false; // Default to DM; thread_id metadata would indicate group

        // Split long messages
        let chunks = crate::types::split_message(&text, MAX_MESSAGE_LEN);
        for chunk in chunks {
            self.cli_send(&user.platform_id, chunk, is_group).await?;
        }

        Ok(())
    }

    async fn send_typing(&self, user: &ChannelUser) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = self.base_cmd();
        cmd.arg("msg").arg("typing").arg(&user.platform_id);

        // Best-effort, don't block on failure
        let _ = cmd.output().await;
        Ok(())
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.shutdown_tx.send(true);
        self.connected.store(false, Ordering::Relaxed);
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
        let adapter = ZaloAdapter::new("openzca".to_string(), None);
        assert_eq!(adapter.name(), "zalo");
        assert_eq!(adapter.channel_type(), ChannelType::Custom("zalo".to_string()));
    }

    #[test]
    fn test_zalo_adapter_with_profile() {
        let adapter = ZaloAdapter::new("openzca".to_string(), Some("work".to_string()));
        assert_eq!(adapter.profile.as_deref(), Some("work"));
    }

    #[test]
    fn test_parse_text_message() {
        let json: serde_json::Value = serde_json::json!({
            "threadId": "123456",
            "senderId": "789012",
            "senderName": "Test User",
            "content": "Hello from Zalo!",
            "chatType": "user",
            "msgType": "chat.text",
            "timestamp": 1709450000000i64,
            "msgId": "msg-001",
        });

        let msg = ZaloAdapter::parse_listen_event(&json).unwrap();
        assert_eq!(msg.platform_message_id, "msg-001");
        assert!(!msg.is_group);
        assert_eq!(msg.thread_id.as_deref(), Some("123456"));
        match msg.content {
            ChannelContent::Text(t) => assert_eq!(t, "Hello from Zalo!"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_parse_group_message() {
        let json: serde_json::Value = serde_json::json!({
            "threadId": "group-123",
            "senderId": "user-456",
            "senderName": "Group Member",
            "content": "Hello group!",
            "chatType": "group",
            "msgType": "chat.text",
            "timestamp": 1709450000000i64,
            "msgId": "msg-002",
        });

        let msg = ZaloAdapter::parse_listen_event(&json).unwrap();
        assert!(msg.is_group);
        // Group messages use threadId as platform_id
        assert_eq!(msg.sender.platform_id, "group-123");
    }

    #[test]
    fn test_parse_empty_content_returns_none() {
        let json: serde_json::Value = serde_json::json!({
            "threadId": "123",
            "senderId": "456",
            "content": "",
            "chatType": "user",
            "msgType": "chat.text",
        });

        assert!(ZaloAdapter::parse_listen_event(&json).is_none());
    }

    #[test]
    fn test_parse_lifecycle_missing_fields_returns_none() {
        // Missing threadId
        let json: serde_json::Value = serde_json::json!({
            "senderId": "456",
            "content": "hello",
        });
        assert!(ZaloAdapter::parse_listen_event(&json).is_none());
    }

    #[test]
    fn test_parse_message_with_quote() {
        let json: serde_json::Value = serde_json::json!({
            "threadId": "123",
            "senderId": "456",
            "content": "Reply to this\n[reply context: 789: original message]",
            "chatType": "user",
            "msgType": "chat.text",
            "msgId": "msg-003",
            "quote": {
                "msgId": "original-msg-id",
                "content": "original message"
            }
        });

        let msg = ZaloAdapter::parse_listen_event(&json).unwrap();
        assert!(msg.metadata.contains_key("quote"));
    }
}
