use std::sync::Arc;

use tokio::sync::RwLock;

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
pub fn create_swarm(configs: Vec<SwarmObject>) -> Swarm {
  let mut swarm = Swarm::new();

  for config in configs {
    swarm.add_bot(&config.username, config.plugins);
  }

  swarm
}

/// Вспомогательная функция создание shared-роя ботов.
pub fn create_shared_swarm(configs: Vec<SwarmObject>) -> SharedSwarm {
  Arc::new(RwLock::new(create_swarm(configs)))
}

/// Неблокирующий запуск shared-роя ботов на сервер.
pub fn launch_shared_swarm(
  swarm: SharedSwarm,
  server_host: String,
  server_port: u16,
  join_delay: u64,
) {
  tokio::spawn(async move {
    let mut swarm_guard = swarm.write().await;
    let bots = std::mem::take(&mut swarm_guard.bots);
    drop(swarm_guard);

    if join_delay > 0 {
      for bot in bots {
        let handle = bot.spawn(&server_host, server_port);
        swarm.write().await.handles.push(handle);
        sleep(join_delay).await;
      }
    } else {
      let mut handles = Vec::new();

      for bot in bots {
        handles.push(bot.spawn(&server_host, server_port));
      }

      swarm.write().await.handles.extend(handles);
    }
  });
}
