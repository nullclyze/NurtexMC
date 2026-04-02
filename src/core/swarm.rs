#![allow(dead_code)]

use std::io;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::core::bot::Bot;
use crate::core::common::{BotCommand, BotPlugins, BotTerminal};
use crate::core::data::{Storage, StorageLock};
use crate::utils::sleep;

pub struct Swarm {
  /// Список всех ботов, после запуска данный список будет пустым
  pub bots: Vec<Bot>,

  /// Список всех терминалов, используется для управления определёнными ботами
  pub terminals: Vec<Arc<BotTerminal>>,

  /// Список всех задач (задач подключений)
  pub handles: Vec<JoinHandle<io::Result<()>>>,

  /// Shared-хранилище роя, в нём хранятся общие данные ботов о мире
  pub shared_storage: StorageLock,
}

pub type SharedSwarm = Arc<RwLock<Swarm>>;

impl Swarm {
  pub fn new() -> Self {
    Self {
      bots: Vec::new(),
      terminals: Vec::new(),
      handles: Vec::new(),
      shared_storage: Arc::new(RwLock::new(Storage::new())),
    }
  }

  /// Метод добавления бота в рой
  pub fn add_bot(&mut self, object: SwarmObject) {
    let mut bot = Bot::new(&object.username)
      .set_uuid(object.uuid)
      .set_plugins(object.plugins);

    if object.use_shared_storage {
      bot = bot.set_shared_storage(self.shared_storage.clone());
    }

    let terminal = Arc::clone(&bot.terminal);

    self.bots.push(bot);
    self.terminals.push(terminal);
  }

  /// Метод получения бота по его юзернейму
  pub fn get_bot(&self, username: &str) -> Option<&Bot> {
    self.bots.iter().find(|b| b.username == username)
  }

  /// Метод получение мутабельной ссылки на бота по его юзернейму
  pub fn get_bot_mut(&mut self, username: &str) -> Option<&mut Bot> {
    self.bots.iter_mut().find(|b| b.username == username)
  }

  /// Метод, запускающий всех ботов из роя, который блокирует поток на время запуска
  pub async fn launch_blocking(&mut self, server_host: &str, server_port: u16, join_delay: u64) {
    let bots = std::mem::take(&mut self.bots);

    for bot in bots {
      self.handles.push(bot.spawn(server_host, server_port));
      sleep(join_delay).await;
    }
  }

  /// Метод отправки команды всем ботам из роя
  pub async fn send(&self, command: BotCommand) {
    for terminal in &self.terminals {
      terminal.send(command.clone()).await;
    }
  }

  /// Метод отправки команды определённому боту из роя
  pub async fn send_to(&self, username: &str, command: BotCommand) {
    for terminal in &self.terminals {
      if terminal.receiver.as_str() == username {
        terminal.send(command).await;
        break;
      }
    }
  }

  /// Метод очистки и выключения роя
  pub async fn destroy(&mut self) {
    for terminal in &self.terminals {
      terminal.send(BotCommand::Disconnect).await;
    }

    sleep(1000).await;

    for handle in &self.handles {
      handle.abort();
    }

    self.bots.clear();
    self.terminals.clear();
    self.handles.clear();
  }

  /// Метод принудительного уничтожения роя без ожидания
  pub fn force_destroy(&mut self) {
    for handle in &self.handles {
      handle.abort();
    }

    self.bots.clear();
    self.terminals.clear();
    self.handles.clear();
  }
}

#[derive(Debug)]
pub struct SwarmObject {
  /// Юзернейм бота
  pub username: String,

  /// UUID бота (по умолчанию нулевой)
  pub uuid: Uuid,

  /// Плагины бота
  pub plugins: BotPlugins,

  /// Флаг, который определяет, будет ли бот использовать shared-хранилище роя (по умолчанию true)
  pub use_shared_storage: bool,
}

impl SwarmObject {
  pub fn new(username: String) -> Self {
    Self {
      username,
      uuid: Uuid::nil(),
      plugins: BotPlugins::default(),
      use_shared_storage: true,
    }
  }

  /// Метод установки UUID
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = uuid;
    self
  }

  /// Метод установки плагинов
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }
}
