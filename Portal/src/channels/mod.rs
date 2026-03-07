//! Independent channel adapters for Portal.
//! These handle webhook parsing and message sending WITHOUT going through OpenFang's channel system.

pub mod telegram;

/// A message received from an external channel.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    /// Sender ID from the channel platform
    pub sender_id: String,
    /// Sender display name (if available)
    pub sender_name: Option<String>,
    /// The text message content
    pub text: String,
    /// Original platform-specific message ID
    #[allow(dead_code)]
    pub platform_message_id: Option<String>,
    /// Chat/conversation ID on the platform
    pub chat_id: String,
}

/// Result of sending a reply
#[derive(Debug)]
pub struct SendResult {
    pub success: bool,
    #[allow(dead_code)]
    pub platform_message_id: Option<String>,
    pub error: Option<String>,
}
