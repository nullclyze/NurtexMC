#![allow(dead_code)]

use std::io;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::core::bot::{Bot, BotCommand, BotPlugins, BotTerminal};
use crate::utils::sleep;

pub struct Swarm {
  pub bots: Vec<Bot>,
  pub terminals: Vec<BotTerminal>,
  pub handles: Vec<JoinHandle<io::Result<()>>>,
}

pub type SharedSwarm = Arc<RwLock<Swarm>>;

impl Swarm {
  pub fn new() -> Self {
    Self {
      bots: Vec::new(),
      terminals: Vec::new(),
      handles: Vec::new(),
    }
  }

  /// Метод добавления бота в рой.
  pub fn add_bot(&mut self, username: &str, plugins: BotPlugins) {
    let (mut bot, terminal) = Bot::new(username, Uuid::nil());

    bot = bot.set_plugins(plugins);

    self.bots.push(bot);
    self.terminals.push(terminal);
  }

  /// Метод получения бота по его юзернейму.
  pub fn get_bot(&self, username: &str) -> Option<&Bot> {
    self.bots.iter().find(|b| b.username == username)
  }

  /// Метод получение мутабельной ссылки на бота по его юзернейму.
  pub fn get_bot_mut(&mut self, username: &str) -> Option<&mut Bot> {
    self.bots.iter_mut().find(|b| b.username == username)
  }

  /// Метод, запускающий всех ботов из роя, который блокирует поток на время запуска.
  pub async fn launch_blocking(
    &mut self,
    server_host: String,
    server_port: u16,
    join_delay: u64,
  ) {
    let bots = std::mem::take(&mut self.bots);
    
    for mut bot in bots {
      let host = server_host.clone();

      self.handles.push(tokio::spawn(async move {
        bot.connect_to(&host, server_port).await
      }));

      sleep(join_delay).await;
    }
  }

  /// Метод отправки команды всем ботам из роя.
  pub async fn send(&self, command: BotCommand) {
    for terminal in &self.terminals {
      terminal.send(command.clone()).await;
    }
  }

  /// Метод отправки команды определённому боту из роя.
  pub async fn send_to(&self, username: &str, command: BotCommand) {
    for terminal in &self.terminals {
      if terminal.receiver.as_str() == username {
        terminal.send(command).await;
        break;
      } 
    }
  }

  /// Метод очистки и выключения роя.
  pub async fn destroy(&mut self) {
    for terminal in &self.terminals {
      terminal.send(BotCommand::Disconnect).await;
    }

    self.bots.clear();
    self.terminals.clear();
    self.handles.clear();
  }
}

#[derive(Debug)]
pub struct BotConfig {
  pub username: String,
  pub uuid: Option<Uuid>,
  pub plugins: BotPlugins,
}

impl BotConfig {
  pub fn new(username: String) -> Self {
    Self {
      username,
      uuid: None,
      plugins: BotPlugins::default(),
    }
  }

  /// Метод установки UUID.
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = Some(uuid);
    self
  }

  /// Метод установки плагинов.
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }
}
