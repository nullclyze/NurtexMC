use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::bot::Bot;
use crate::bot::account::BotAccount;
use crate::bot::transmitter::{BotPackage, NullPackage};
use crate::swarm::{SharedSwarm, Swarm, SwarmObject};
use crate::utils::time::sleep;

pub mod bot;
pub mod swarm;
pub mod utils;

pub mod export;

/// Вспомогательная функция создания бота с дефолтным пакетом.
/// Чтобы указать явный пакет используй функцию `create_bot_with_package`
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём бота
/// let mut bot = create_bot("NurtexBot");
///
/// // Подключаем бота к серверу
/// bot.connect_to("server.com", 25565).await?;
/// ```
pub fn create_bot(username: &str) -> Bot<NullPackage> {
  let account = BotAccount::new(username);
  Bot::create(account)
}

/// Вспомогательная функция создания бота с кастомным пакетом данных.
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём бота
/// let mut bot = create_bot_with_package::<CustomPackage>("NurtexBot");
///
/// // Запускаем бота на сервер
/// bot.connect_to("server.com", 25565).await?;
/// ```
pub fn create_bot_with_package<P: BotPackage>(username: &str) -> Bot<P> {
  let account = BotAccount::new(username);
  Bot::create(account)
}

/// Вспомогательная функция создания роя ботов с дефолтным пакетом.
/// Чтобы указать явный пакет используй функцию `create_swarm_with_package`
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём объекты
/// let mut objects = Vec::new();
///
/// for i in 0..5 {
///   let account = BotAccount::new(format!("bot_{}", i));
///   objects.push(SwarmObject::new(account));
/// }
///
/// // Создаём рой
/// let mut swarm = create_swarm(objects);
///
/// // Запускаем рой на сервер
/// swarm.launch("server.com", 25565, 1000).await;
/// ```
pub fn create_swarm(objects: Vec<SwarmObject>) -> Swarm<NullPackage> {
  let mut swarm = Swarm::create();

  for object in objects {
    swarm.add_object(object);
  }

  swarm
}

/// Вспомогательная функция создания роя ботов с кастомным пакетом данных.
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём рой
/// let mut swarm = create_swarm_with_package::<CustomPackage>(objects);
/// 
/// // Запускаем рой на сервер
/// swarm.launch("server.com", 25565, 1000).await;
/// ```
pub fn create_swarm_with_package<P: BotPackage>(objects: Vec<SwarmObject>) -> Swarm<P> {
  let mut swarm = Swarm::create();

  for object in objects {
    swarm.add_object(object);
  }

  swarm
}

/// Вспомогательная функция создания shared-роя ботов с дефолтным пакетом.
/// Чтобы указать явный пакет используй функцию `create_shared_swarm_with_package`
pub fn create_shared_swarm(objects: Vec<SwarmObject>) -> SharedSwarm<NullPackage> {
  Arc::new(RwLock::new(create_swarm(objects)))
}

/// Вспомогательная функция создания shared-роя ботов с кастомным пакетом данных
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём shared-рой
/// let mut swarm = create_shared_swarm_with_package::<CustomPackage>(objects);
///
/// // Запускаем рой на сервер
/// swarm.write().await.launch("server.com", 25565, 1000).await;
/// ```
pub fn create_shared_swarm_with_package<P: BotPackage>(objects: Vec<SwarmObject>) -> SharedSwarm<P> {
  Arc::new(RwLock::new(create_swarm_with_package(objects)))
}

/// Функция запуска shared-роя, неблокирующая поток на время запуска
pub fn launch_shared_swarm<P: BotPackage>(swarm: SharedSwarm<P>, server_host: impl Into<String>, server_port: u16, join_delay: u64) {
  let host = server_host.into();

  tokio::spawn(async move {
    let bots = std::mem::take(&mut swarm.write().await.bots);

    let mut handles = Vec::new();

    for bot in bots {
      let handle = bot.spawn(&host, server_port);

      handles.push(handle);

      if join_delay > 0 {
        sleep(join_delay).await;
      }
    }

    {
      let mut swarm_guard = swarm.write().await;
      swarm_guard.handles.extend(handles);
    }
  });
}

/// Вспомогательная функция уничтожения shared-роя
pub async fn destroy_shared_swarm<P: BotPackage>(swarm: SharedSwarm<P>) -> std::io::Result<()> {
  match timeout(Duration::from_secs(5), swarm.write()).await {
    Ok(mut guard) => {
      guard.destroy().await;
      Ok(())
    }
    Err(_) => match timeout(Duration::from_millis(100), swarm.write()).await {
      Ok(mut guard) => {
        guard.force_destroy();
        Ok(())
      }
      Err(_) => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Failed to acquire write lock for swarm destruction")),
    },
  }
}
