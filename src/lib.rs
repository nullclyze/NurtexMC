use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::timeout;

// Ре-экспорт
pub use azalea_core::*;
pub use azalea_crypto::*;
pub use azalea_entity::*;
pub use azalea_protocol::*;
pub use uuid::Uuid;

use crate::core::swarm::{SharedSwarm, Swarm, SwarmObject};
use crate::utils::sleep;

pub mod core;
pub mod utils;

/// Вспомогательная функция создание роя ботов.
pub fn create_swarm(objects: Vec<SwarmObject>) -> Swarm {
  let mut swarm = Swarm::new();

  for object in objects {
    swarm.add_bot(object);
  }

  swarm
}

/// Вспомогательная функция создание shared-роя ботов.
pub fn create_shared_swarm(objects: Vec<SwarmObject>) -> SharedSwarm {
  Arc::new(RwLock::new(create_swarm(objects)))
}

/// Неблокирующий запуск shared-роя ботов на сервер.
pub fn launch_shared_swarm(
  swarm: SharedSwarm,
  server_host: String,
  server_port: u16,
  join_delay: u64,
) {
  tokio::spawn(async move {
    let bots = {
      let mut swarm_guard = swarm.write().await;
      std::mem::take(&mut swarm_guard.bots)
    };

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

/// Вспомогательная функция уничтожения shared-роя с таймаутом.
pub async fn destroy_shared_swarm(
  swarm: SharedSwarm,
  timeout_duration: Duration,
) -> std::io::Result<()> {
  match timeout(timeout_duration, swarm.write()).await {
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
