use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::common::BotTerminal;
use crate::core::events::{BotEvent, ChatPayload, PacketPayload};

pub type AsyncEventInvoker =
  Box<dyn Fn(Arc<BotTerminal>, BotEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct EventInvoker {
  login_finished_handler:
    Option<Arc<dyn Fn(Arc<BotTerminal>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
  config_finished_handler:
    Option<Arc<dyn Fn(Arc<BotTerminal>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
  spawn_handler:
    Option<Arc<dyn Fn(Arc<BotTerminal>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
  death_handler:
    Option<Arc<dyn Fn(Arc<BotTerminal>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
  disconnect_handler:
    Option<Arc<dyn Fn(Arc<BotTerminal>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>>,
  chat_handler: Option<
    Arc<
      dyn Fn(Arc<BotTerminal>, ChatPayload) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
    >,
  >,
  packet_handler: Option<
    Arc<
      dyn Fn(Arc<BotTerminal>, PacketPayload) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
    >,
  >,
}

impl EventInvoker {
  pub fn new() -> Self {
    Self {
      login_finished_handler: None,
      config_finished_handler: None,
      spawn_handler: None,
      death_handler: None,
      disconnect_handler: None,
      chat_handler: None,
      packet_handler: None,
    }
  }

  pub fn on_login_finished<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.login_finished_handler = Some(Arc::new(move |terminal| Box::pin(handler(terminal))));
  }

  pub fn on_config_finished<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.config_finished_handler = Some(Arc::new(move |terminal| Box::pin(handler(terminal))));
  }

  pub fn on_spawn<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.spawn_handler = Some(Arc::new(move |terminal| Box::pin(handler(terminal))));
  }

  pub fn on_death<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.death_handler = Some(Arc::new(move |terminal| Box::pin(handler(terminal))));
  }

  pub fn on_disconnect<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.disconnect_handler = Some(Arc::new(move |terminal| Box::pin(handler(terminal))));
  }

  pub fn on_chat<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>, ChatPayload) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.chat_handler = Some(Arc::new(move |terminal, payload| {
      Box::pin(handler(terminal, payload))
    }));
  }

  pub fn on_packet<F, Fut>(&mut self, handler: F)
  where
    F: Fn(Arc<BotTerminal>, PacketPayload) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.packet_handler = Some(Arc::new(move |terminal, payload| {
      Box::pin(handler(terminal, payload))
    }));
  }

  /// Метод триггеринга определённого события.
  pub async fn trigger(&self, terminal: Arc<BotTerminal>, event: BotEvent) {
    match event {
      BotEvent::LoginFinished => {
        if let Some(handler) = &self.login_finished_handler {
          handler(terminal).await;
        }
      }
      BotEvent::ConfigurationFinished => {
        if let Some(handler) = &self.config_finished_handler {
          handler(terminal).await;
        }
      }
      BotEvent::Spawn => {
        if let Some(handler) = &self.spawn_handler {
          handler(terminal).await;
        }
      }
      BotEvent::Death => {
        if let Some(handler) = &self.death_handler {
          handler(terminal).await;
        }
      }
      BotEvent::Disconnect => {
        if let Some(handler) = &self.disconnect_handler {
          handler(terminal).await;
        }
      }
      BotEvent::Chat(payload) => {
        if let Some(handler) = &self.chat_handler {
          handler(terminal, payload).await;
        }
      }
      BotEvent::Packet(payload) => {
        if let Some(handler) = &self.packet_handler {
          handler(terminal, payload).await;
        }
      }
    }
  }
}
