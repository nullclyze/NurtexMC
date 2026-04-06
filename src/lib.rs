use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::bot::Bot;
use crate::bot::account::BotAccount;
use crate::swarm::{SharedSwarm, Swarm, SwarmObject};
use crate::utils::time::sleep;

pub mod bot;
pub mod swarm;
pub mod utils;

pub mod export;

/// Вспомогательная функция создание бота
pub fn create_bot(username: &str) -> Bot {
  let account = BotAccount::new(username);
  Bot::create(account)
}

/// Вспомогательная функция создание роя ботов
pub fn create_swarm(objects: Vec<SwarmObject>) -> Swarm {
  let mut swarm = Swarm::create();

  for object in objects {
    swarm.add_object(object);
  }

  swarm
}

/// Вспомогательная функция создание shared-роя ботов
pub fn create_shared_swarm(objects: Vec<SwarmObject>) -> SharedSwarm {
  Arc::new(RwLock::new(create_swarm(objects)))
}

/// Функция запуска shared-роя, неблокирующая поток на время запуска
pub fn launch_shared_swarm(
  swarm: SharedSwarm,
  server_host: String,
  server_port: u16,
  join_delay: u64,
) {
  tokio::spawn(async move {
    let bots = std::mem::take(&mut swarm.write().await.bots);

    let mut handles = Vec::new();

    for bot in bots {
      let handle = bot.spawn(&server_host, server_port);

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
pub async fn destroy_shared_swarm(swarm: SharedSwarm) -> std::io::Result<()> {
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
      Err(_) => Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Failed to acquire write lock for swarm destruction",
      )),
    },
  }
}
