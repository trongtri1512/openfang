//! Telegram Bot API adapter for Portal.
//!
//! Handles:
//! - Parsing incoming webhook updates (messages)
//! - Sending replies via Bot API
//! - Setting/deleting webhooks

use super::{IncomingMessage, SendResult};
use serde_json::Value;
use tracing::{info, warn};

/// Parse a Telegram webhook update into an IncomingMessage.
pub fn parse_webhook(body: &Value) -> Option<IncomingMessage> {
    // Telegram sends updates with a "message" field for regular messages
    let message = body.get("message")
        .or_else(|| body.get("edited_message"))?;

    let text = message.get("text")?.as_str()?.to_string();
    let chat = message.get("chat")?;
    let chat_id = chat.get("id")?.as_i64()?.to_string();
    let from = message.get("from")?;
    let sender_id = from.get("id")?.as_i64()?.to_string();

    let first_name = from.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
    let last_name = from.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
    let sender_name = if last_name.is_empty() {
        first_name.to_string()
    } else {
        format!("{} {}", first_name, last_name)
    };

    let message_id = message.get("message_id").and_then(|v| v.as_i64()).map(|v| v.to_string());

    Some(IncomingMessage {
        sender_id,
        sender_name: Some(sender_name),
        text,
        platform_message_id: message_id,
        chat_id,
    })
}

/// Send a reply message via Telegram Bot API.
pub async fn send_reply(config: &Value, chat_id: &str, text: &str) -> SendResult {
    let bot_token = match config.get("bot_token").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return SendResult {
            success: false,
            platform_message_id: None,
            error: Some("Missing bot_token in config".into()),
        },
    };

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown"
    });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let resp_body: Value = resp.json().await.unwrap_or_default();
                let msg_id = resp_body
                    .get("result")
                    .and_then(|r| r.get("message_id"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string());
                info!(chat_id = %chat_id, "Telegram message sent successfully");
                SendResult {
                    success: true,
                    platform_message_id: msg_id,
                    error: None,
                }
            } else {
                let err_body = resp.text().await.unwrap_or_default();
                warn!(chat_id = %chat_id, status = %status, "Telegram sendMessage failed: {}", err_body);
                SendResult {
                    success: false,
                    platform_message_id: None,
                    error: Some(format!("Telegram API error {}: {}", status, err_body)),
                }
            }
        }
        Err(e) => {
            warn!(chat_id = %chat_id, "Telegram sendMessage connection error: {}", e);
            SendResult {
                success: false,
                platform_message_id: None,
                error: Some(format!("Connection error: {}", e)),
            }
        }
    }
}

/// Set webhook URL on Telegram for a bot.
pub async fn set_webhook(bot_token: &str, webhook_url: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/setWebhook", bot_token);
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "url": webhook_url,
        "allowed_updates": ["message", "edited_message"]
    });

    let resp = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Failed to connect to Telegram: {}", e))?;

    let status = resp.status();
    let resp_body: Value = resp.json().await.unwrap_or_default();

    if status.is_success() && resp_body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        info!(webhook_url = %webhook_url, "Telegram webhook set successfully");
        Ok(())
    } else {
        let desc = resp_body.get("description").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        Err(format!("Telegram setWebhook failed: {}", desc))
    }
}

/// Delete webhook from Telegram bot.
pub async fn delete_webhook(bot_token: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/deleteWebhook", bot_token);
    let client = reqwest::Client::new();

    let resp = client.post(&url).send().await
        .map_err(|e| format!("Failed to connect to Telegram: {}", e))?;

    if resp.status().is_success() {
        info!("Telegram webhook deleted");
        Ok(())
    } else {
        Err("Failed to delete webhook".into())
    }
}

/// Get bot info to verify a bot token is valid.
pub async fn get_bot_info(bot_token: &str) -> Result<Value, String> {
    let url = format!("https://api.telegram.org/bot{}/getMe", bot_token);
    let client = reqwest::Client::new();

    let resp = client.get(&url).send().await
        .map_err(|e| format!("Connection error: {}", e))?;

    let status = resp.status();
    let body: Value = resp.json().await.unwrap_or_default();

    if status.is_success() && body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok(body.get("result").cloned().unwrap_or_default())
    } else {
        let desc = body.get("description").and_then(|v| v.as_str()).unwrap_or("Invalid token");
        Err(format!("Bot verification failed: {}", desc))
    }
}
