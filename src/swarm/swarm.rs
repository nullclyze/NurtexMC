#![allow(dead_code)]

use std::io;
use std::sync::Arc;

use azalea_protocol::connect::Proxy;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::Bot;
use crate::bot::account::BotAccount;
use crate::bot::events::EventInvoker;
use crate::bot::options::{BotInformation, BotPlugins};
use crate::bot::terminal::{BotCommand, BotTerminal};
use crate::bot::world::{Storage, StorageLock};
use crate::utils::time::sleep;

/// Рой ботов. Данная структура содержит
/// в себе специальные shared-хранилища.
/// Управлять ботами из роя напрямую **нельзя**,
/// так как функция запуска забирает себе все
/// объекты из `bots`. Для управления используются
/// терминалы (поле `terminals`).
/// 
/// Пример параллельного управления ботами:
/// ```rust, ignore
/// // Проходимся по всем терминалам из роя
/// swarm.read().await.for_each_parallel(|terminal| async move {
///   // Отправляем сообщение в чат при помощи терминала
///   terminal.chat("Привет, мир!").await;
/// }).await;
/// ```
pub struct Swarm {
  /// Список всех ботов
  pub bots: Vec<Bot>,

  /// Список всех терминалов, используется для управления определёнными ботами
  pub terminals: Vec<Arc<BotTerminal>>,

  /// Список всех задач (задач подключений)
  pub handles: Vec<JoinHandle<io::Result<()>>>,

  /// Shared-хранилище роя, в нём хранятся общие данные ботов о мире
  pub shared_storage: StorageLock,
}

/// Вспомогательная обёртка для Swarm
pub type SharedSwarm = Arc<RwLock<Swarm>>;

impl Swarm {
  /// Метод создания нового роя
  pub fn create() -> Self {
    Self {
      bots: Vec::new(),
      terminals: Vec::new(),
      handles: Vec::new(),
      shared_storage: Arc::new(RwLock::new(Storage::new()))
    }
  }

  /// Последовательный асинхронный for-each
  pub async fn for_each_async<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut,
    Fut: std::future::Future<Output = ()>,
  {
    for terminal in &self.terminals {
      f(Arc::clone(terminal)).await;
    }
  }

  /// Параллельный асинхронный for-each
  pub async fn for_each_parallel<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    let f = Arc::new(f);
    let mut handles = Vec::with_capacity(self.bots.len());

    for terminal in &self.terminals {
      let f_clone = Arc::clone(&f);
      let terminal_clone = Arc::clone(terminal);

      let handle = tokio::spawn(async move {
        f_clone(terminal_clone).await;
      });

      handles.push(handle);
    }

    for handle in handles {
      handle.await.ok();
    }
  }

  /// Параллельный синхронный for-each
  pub fn for_each<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<BotTerminal>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    let f = Arc::new(f);

    for terminal in &self.terminals {
      let f_clone = Arc::clone(&f);
      let terminal_clone = Arc::clone(terminal);

      tokio::spawn(async move {
        f_clone(terminal_clone).await;
      });
    }
  }

  /// Метод добавления объекта бота в рой
  pub fn add_object(&mut self, object: SwarmObject) {
    let mut bot = Bot::create(object.account)
      .set_connection_timeout(object.connection_timeout)
      .set_plugins(object.plugins)
      .set_information(object.information);

    if object.use_shared_storage {
      bot = bot.set_shared_storage(self.shared_storage.clone());
    }

    if let Some(proxy) = object.proxy {
      bot = bot.set_proxy(proxy);
    }

    if let Some(invoker) = object.event_invoker {
      bot = bot.set_event_invoker(invoker);
    }

    let terminal = Arc::clone(&bot.terminal);

    self.bots.push(bot);
    self.terminals.push(terminal);
  }

  /// Метод запуска роя, блокирующий поток на время запуска
  pub async fn launch(&mut self, server_host: &str, server_port: u16, join_delay: u64) {
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
      if terminal.account.username.as_str() == username {
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

/// Объект роя, выполняющий роль **вспомогательной структуры**, которая содержит информацию.
/// Данный объект **НЕ является** полноценным ботом для роя, это лишь обёртка над его поверхностной
/// информацией (проще говоря опции).
pub struct SwarmObject {
  /// Аккаунт объекта бота
  pub account: BotAccount,

  plugins: BotPlugins,
  event_invoker: Option<EventInvoker>,
  connection_timeout: u64,
  proxy: Option<Proxy>,
  information: BotInformation,
  use_shared_storage: bool,
}

impl SwarmObject {
  /// Метод создания нового объекта роя
  pub fn new(account: BotAccount) -> Self {
    Self {
      account: account,
      plugins: BotPlugins::default(),
      event_invoker: None,
      connection_timeout: 14000,
      proxy: None,
      information: BotInformation::default(),
      use_shared_storage: true,
    }
  }

  /// Метод установки плагинов
  pub fn set_plugins(mut self, plugins: BotPlugins) -> Self {
    self.plugins = plugins;
    self
  }

  /// Метод установки инициатора событий
  pub fn set_event_invoker(mut self, invoker: EventInvoker) -> Self {
    self.event_invoker = Some(invoker);
    self
  }

  /// Метод установки таймаута подключения
  pub fn set_connection_timeout(mut self, timeout: u64) -> Self {
    self.connection_timeout = timeout;
    self
  }

  /// Метод установки информации
  pub fn set_information(mut self, information: BotInformation) -> Self {
    self.information = information;
    self
  }

  /// Метод установки прокси
  pub fn set_proxy(mut self, proxy: Proxy) -> Self {
    self.proxy = Some(proxy);
    self
  }

  /// Метод установки значения для флага `use_shared_storage`,
  /// который отвечат за использование Shared-хранилища ботами
  pub fn set_use_shared_storage(mut self, state: bool) -> Self {
    self.use_shared_storage = state;
    self
  }
}
