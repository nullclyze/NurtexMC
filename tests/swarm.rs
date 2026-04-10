#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::bot::Bot;
  use nurtex::bot::account::BotAccount;
  use nurtex::bot::components::position::Position;
  use nurtex::bot::events::EventInvoker;
  use nurtex::bot::options::{AutoReconnectPlugin, BotPlugins};
  use nurtex::bot::transmitter::BotPackage;
  use nurtex::swarm::SwarmObject;
  use nurtex::utils::time::sleep;
  use nurtex::{create_shared_swarm_with_package, destroy_shared_swarm, launch_shared_swarm};

  #[derive(Debug, Clone)]
  struct MyPackage {
    username: String,
    position: Position,
  }

  impl BotPackage for MyPackage {
    fn describe<P: BotPackage>(bot: &Bot<P>) -> Self {
      Self {
        username: bot.account.username.clone(),
        position: bot.components.position,
      }
    }
  }

  #[tokio::test]
  async fn launch_swarm() -> io::Result<()> {
    let mut objects = Vec::new();

    for i in 0..=5 {
      let mut event_invoker = EventInvoker::new();

      event_invoker.on_spawn(|terminal| async move {
        println!("Бот {} заспавнился!", terminal.account.username);
      });

      event_invoker.on_chat(|terminal, payload| async move {
        println!("Бот {} получил сообщение: {}", terminal.account.username, payload.message);
      });

      let account = BotAccount::new(format!("bot_{}", i));

      let object = SwarmObject::new(account).set_event_invoker(event_invoker).set_plugins(BotPlugins {
        auto_reconnect: AutoReconnectPlugin {
          enabled: true,
          reconnect_delay: 1000,
        },
        ..Default::default()
      });

      objects.push(object);
    }

    let swarm = create_shared_swarm_with_package::<MyPackage>(objects);

    swarm.read().await.for_each_transmitters(|transmitter| async move {
      let mut receiver = transmitter.subscribe();

      while let Ok(package) = receiver.recv().await {
        println!("Позиция бота {}: {:?}", package.username, package.position);
      }
    });

    launch_shared_swarm(swarm.clone(), "localhost", 25565, 25);

    sleep(8000).await;

    swarm.read().await.for_each(|terminal| async move {
      terminal.reconnect("localhost", 25565, 1000).await;
    });

    sleep(8000).await;

    destroy_shared_swarm(swarm).await?;

    Ok(())
  }
}
