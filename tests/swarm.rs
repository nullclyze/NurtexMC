#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::bot::account::BotAccount;
  use nurtex::bot::options::{AutoReconnectPlugin, BotPlugins};
  use nurtex::bot::events::EventInvoker;
  use nurtex::swarm::SwarmObject;
  use nurtex::utils::time::sleep;
  use nurtex::{create_shared_swarm, destroy_shared_swarm, launch_shared_swarm};

  #[tokio::test]
  async fn launch_swarm() -> io::Result<()> {
    let mut objects = Vec::new();

    for i in 0..=5 {
      let mut event_invoker = EventInvoker::new();

      event_invoker.on_spawn(async |terminal| {
        println!("Бот {} заспавнился!", terminal.account.username);
      });

      event_invoker.on_chat(async |terminal, payload| {
        println!(
          "Бот {} получил сообщение: {}",
          terminal.account.username, payload.message
        );
      });

      let account = BotAccount::new(format!("bot_{}", i));

      let object = SwarmObject::new(account)
        .set_event_invoker(event_invoker)
        .set_plugins(BotPlugins {
          auto_reconnect: AutoReconnectPlugin {
            enabled: true,
            reconnect_delay: 1000,
          },
          ..Default::default()
        });

      objects.push(object);
    }

    let swarm = create_shared_swarm(objects);

    launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 25);

    sleep(8000).await;

    swarm.read().await.for_each_parallel(|terminal| async move {
      terminal.chat("Test").await;
    }).await;

    {
      let guard = swarm.read().await;
      let shared_storage = guard.shared_storage.read().await;

      println!("Общее число сущностей: {}", shared_storage.entities.len());
      println!("Общее число чанков: {}", shared_storage.chunks.len());
    }

    sleep(60000).await;

    destroy_shared_swarm(swarm).await?;

    Ok(())
  }
}
