use nurtex_protocol::{packets::play::ServersideChatMessage, types::AdditionalMessageInfo};

use crate::bot::{Bot, capture_connection};

pub trait BotChatExt {
  /// Метод отправки сообщения в чат
  fn chat_message(&self, message: impl Into<String>) -> impl std::future::Future<Output = std::io::Result<()>>;

  /// Метод отправки сообщения в чат с заданными опциями
  fn chat_message_with_opts(
    &self,
    message: impl Into<String>,
    timestamp: i64,
    signature: Option<Vec<u8>>,
    additional_info: AdditionalMessageInfo,
  ) -> impl std::future::Future<Output = std::io::Result<()>>;
}

impl BotChatExt for Bot {
  async fn chat_message(&self, message: impl Into<String>) -> std::io::Result<()> {
    capture_connection(&self.connection, async |conn| {
      conn
        .write_play_packet(nurtex_protocol::packets::play::ServersidePlayPacket::ChatMessage(ServersideChatMessage {
          message: message.into(),
          timestamp: {
            match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
              Ok(d) => d.as_secs() as i64,
              Err(_) => 0,
            }
          },
          salt: 0,
          signature: None,
          additional_info: AdditionalMessageInfo::default(),
        }))
        .await
    })
    .await
  }

  async fn chat_message_with_opts(&self, message: impl Into<String>, timestamp: i64, signature: Option<Vec<u8>>, additional_info: AdditionalMessageInfo) -> std::io::Result<()> {
    capture_connection(&self.connection, async |conn| {
      conn
        .write_play_packet(nurtex_protocol::packets::play::ServersidePlayPacket::ChatMessage(ServersideChatMessage {
          message: message.into(),
          timestamp: timestamp,
          salt: 0,
          signature: signature,
          additional_info: additional_info,
        }))
        .await
    })
    .await
  }
}
