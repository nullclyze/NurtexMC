use std::io::{self};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::Bot;
use crate::storage::Storage;
use crate::swarm::{JoinDelay, Speedometer};

/// Рой ботов
pub struct Swarm {
  /// Список всех ботов
  pub bots: Vec<Arc<Bot>>,

  /// Список всех хэндлов
  handles: Vec<JoinHandle<core::result::Result<(), std::io::Error>>>,

  /// Спидометр (опционально)
  speedometer: Option<Arc<Speedometer>>,

  /// Общее хранилище данных
  shared_storage: Arc<RwLock<Storage>>,
}

impl Swarm {
  /// Метод создания нового роя
  pub fn create() -> Self {
    Self {
      bots: Vec::new(),
      handles: Vec::new(),
      speedometer: None,
      shared_storage: Arc::new(RwLock::new(Storage::null())),
    }
  }

  /// Метод создания нового роя с указанием ёмкости
  pub fn create_with_capacity(capacity: usize) -> Self {
    Self {
      bots: Vec::with_capacity(capacity),
      handles: Vec::with_capacity(capacity),
      speedometer: None,
      shared_storage: Arc::new(RwLock::new(Storage::null())),
    }
  }

  /// Метод создания нового роя со спидометром
  pub fn create_with_speedometer(speedometer: Arc<Speedometer>) -> Self {
    Self {
      bots: Vec::new(),
      handles: Vec::new(),
      speedometer: Some(speedometer),
      shared_storage: Arc::new(RwLock::new(Storage::null())),
    }
  }

  /// Метод установки спидометра
  pub fn set_speedometer(mut self, speedometer: Arc<Speedometer>) -> Self {
    self.speedometer = Some(speedometer);
    self
  }

  /// Метод установки общего хранилища
  pub fn set_shared_storage(mut self, storage: Arc<RwLock<Storage>>) -> Self {
    self.shared_storage = storage;
    self
  }

  /// Метод получения спидометра
  pub fn get_speedometer(&self) -> Option<Arc<Speedometer>> {
    if let Some(speedometer) = &self.speedometer {
      Some(Arc::clone(speedometer))
    } else {
      None
    }
  }

  /// Метод получения общего хранилища
  pub fn get_shared_storage(&self) -> Arc<RwLock<Storage>> {
    Arc::clone(&self.shared_storage)
  }

  /// Последовательный for-each
  pub async fn for_each_consistent<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<Bot>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    for i in &self.bots {
      let bot = Arc::clone(i);
      f(bot).await;
    }
  }

  /// Параллельный for-each
  pub fn for_each_parallel<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<Bot>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    let f = Arc::new(f);

    for i in &self.bots {
      let f_clone = Arc::clone(&f);
      let bot = Arc::clone(i);

      tokio::spawn(f_clone(bot));
    }
  }

  /// Метод добавления бота в рой
  pub fn add_bot(&mut self, bot: Bot) {
    self.bots.push(Arc::new(bot.set_storage(Arc::clone(&self.shared_storage))));
  }

  /// Метод добавления нескольких ботов в рой
  pub fn add_bots(&mut self, bots: Vec<Bot>) {
    for bot in bots {
      self.bots.push(Arc::new(bot.set_storage(Arc::clone(&self.shared_storage))));
    }
  }

  /// Метод проверки уникальности юзернейма.
  /// Он сверяет данный юзернейм со всеми юзернеймами уже ранее добавленных ботов в рой
  pub fn username_is_unique(&mut self, username: &str) -> bool {
    for bot in &self.bots {
      if username == bot.username() {
        return false;
      }
    }

    true
  }

  /// Метод обычного запуска роя
  pub async fn launch(&mut self, server_host: impl Into<String>, server_port: u16, join_delay: JoinDelay) {
    let host = server_host.into();
    let total_bots = self.bots.len();

    for (index, bot) in self.bots.iter().enumerate() {
      let handle = bot.connect_with_handle(&host, server_port);
      self.handles.push(handle);

      let delay = join_delay.activate(index, total_bots);

      if index < total_bots - 1 {
        tokio::time::sleep(Duration::from_millis(delay)).await;
      }
    }
  }

  /// Метод мгновенного запуска роя (без задержки)
  pub fn instant_launch(&mut self, server_host: impl Into<String>, server_port: u16) {
    let host = server_host.into();

    for bot in &self.bots {
      let handle = bot.connect_with_handle(&host, server_port);
      self.handles.push(handle);
    }
  }

  /// Метод **тихого** запуска роя (не блокирует текущий поток).
  /// Важно понимать что он **НЕ** добавляет хэндлы подключений ботов,
  /// соответственно любое взаимодействие с ними будет невозможным,
  /// так же **могут быть проблемы** при остановке роя (редко и
  /// только если выполняются долгие блокирующие операции с подключениями)
  pub fn quiet_launch(&mut self, server_host: impl Into<String>, server_port: u16, join_delay: JoinDelay) {
    let host = server_host.into();
    let total_bots = self.bots.len();

    let mut bots = Vec::with_capacity(self.bots.len());
    for bot in &self.bots {
      bots.push(Arc::clone(bot));
    }

    tokio::spawn(async move {
      for (index, bot) in bots.iter().enumerate() {
        bot.connect_with_handle(&host, server_port);

        if index < total_bots - 1 {
          let delay = join_delay.activate(index, total_bots);
          tokio::time::sleep(Duration::from_millis(delay)).await;
        }
      }
    });
  }

  /// Метод получения количества ботов в рое
  pub fn bots_count(&self) -> usize {
    self.bots.len()
  }

  /// Метод получения количества хэндлов (обычно равняется количеству ботов)
  pub fn handles_count(&self) -> usize {
    self.handles.len()
  }

  /// Метод проверки существования ботов в рое
  pub fn is_null(&self) -> bool {
    self.bots.is_empty()
  }

  /// Метод получения всех юзернеймов ботов
  pub fn get_bot_usernames(&self) -> Vec<String> {
    self.bots.iter().map(|bot| bot.username().to_string()).collect()
  }

  /// Метод выключения и очистки роя.
  /// После использования этого метода список ботов и их хэндлов полностью очищается,
  /// запустить тот же рой будет невозможно без нового добавления ботов через метод `add_bot`
  pub async fn shutdown(&mut self) -> io::Result<()> {
    self.abort_handles();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // По сути все задачи ботов, связанные с подключением, должны уничтожиться
    // и соответственно все `NurtexConnection` должны быть доступны для записи
    for bot in &self.bots {
      bot.shutdown().await?;
    }

    if let Some(speedometer) = &self.speedometer {
      speedometer.stop();
    }

    self.handles.clear();
    self.bots.clear();
    self.shared_storage.write().await.clear();

    Ok(())
  }

  /// Метод отмены всех хэндлов, если нужно корректно и полноценно
  /// остановить рой, используй метод `shutdown`
  pub fn abort_handles(&self) {
    for handle in &self.handles {
      handle.abort();
    }
  }

  /// Метод ожидания завершения всех хэндлов
  pub async fn wait_handles(&mut self) {
    for handle in &mut self.handles {
      if !handle.is_finished() {
        // Думаю, здесь логичнее игнорировать любые ошибки
        let _ = handle.await;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use crate::bot::Bot;
  use crate::swarm::{JoinDelay, Swarm};

  #[tokio::test]
  async fn test_instant() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.instant_launch("localhost", 25565);

    tokio::time::sleep(Duration::from_secs(3)).await;

    swarm.for_each_parallel(async |bot| {
      let position = bot.get_position().await;
      let rotation = bot.get_rotation().await;

      println!("[{}] Позиция: {:?}, Ротация: {:?}", bot.username(), position, rotation);
    });

    tokio::time::sleep(Duration::from_secs(8)).await;

    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_quiet() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.quiet_launch("localhost", 25565, JoinDelay::fixed(500));
    tokio::time::sleep(Duration::from_secs(5)).await;
    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_wait_handles() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(6);

    for i in 0..6 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::fixed(200)).await;
    swarm.wait_handles().await;

    Ok(())
  }

  #[tokio::test]
  async fn test_shared_storage() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(6);

    for i in 0..6 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::fixed(200)).await;

    for _ in 0..5 {
      let storage = swarm.get_shared_storage();

      let entities = {
        let guard = storage.read().await;
        guard.entities.clone()
      };

      println!("Сущности: {:?}", entities);

      tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
  }
}
