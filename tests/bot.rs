#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::bot::Bot;
  use nurtex::bot::components::inventory::Inventory;
  use nurtex::bot::components::position::Position;
  use nurtex::bot::events::EventInvoker;
  use nurtex::bot::transmitter::BotPackage;
  use nurtex::create_bot_with_package;

  #[derive(Debug, Clone)]
  struct MyPackage {
    position: Position,
    on_ground: bool,
    inventory: Inventory,
  }

  impl BotPackage for MyPackage {
    fn describe<P: BotPackage>(bot: &Bot<P>) -> Self {
      Self {
        position: bot.get_position(),
        on_ground: bot.physics.on_ground,
        inventory: bot.components.inventory.clone(),
      }
    }
  }

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let bot = create_bot_with_package::<MyPackage>("NurtexBot");

    let transmitter = bot.get_transmitter();

    tokio::spawn(async move {
      let mut receiver = transmitter.subscribe();

      while let Ok(package) = receiver.recv().await {
        println!("-----------------------------------------");
        println!("Позиция бота: {:?}", package.position);
        println!("Состояние on_ground: {}", package.on_ground);

        for (id, container) in package.inventory.containers {
          println!("Контейнер {}: {:?}", id, container);
        }
      }
    });

    let mut event_invoker = EventInvoker::new();

    event_invoker.on_death(|terminal| async move {
      println!("Бот {} умер.", terminal.username);
    });

    event_invoker.on_spawn(|terminal| async move {
      println!("Бот {} заспавнился!", terminal.username);
    });

    event_invoker.on_chunk_loaded(|terminal, payload| async move {
      println!("Загружен новый чанк для {} ({}, {})", terminal.username, payload.x, payload.z);
    });

    event_invoker.on_update_rotation(|terminal, payload| async move {
      println!("Ротация бота {} обновлена: {:?} -> {:?}", terminal.username, payload.old_rotation, payload.rotation);
    });

    event_invoker.on_chat(|terminal, payload| async move {
      println!("Бот {} получил сообщение: {}", terminal.username, payload.message);

      if payload.message.contains("reconnect") {
        terminal.reconnect("localhost", 25565, 100).await;
      }
    });

    event_invoker.on_disconnect(|terminal, payload| async move {
      println!("Бот {} отключился по причине: {}", terminal.username, payload.reason);
    });

    bot.set_transmitter_interval(2000).set_event_invoker(event_invoker).connect_to("localhost", 25565).await?;

    Ok(())
  }
}
