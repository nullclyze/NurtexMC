#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::bot::components::position::Position;
  use nurtex::bot::events::EventInvoker;
  use nurtex::bot::transmitter::BotPackage;
  use nurtex::create_bot_with_package;

  #[derive(Debug, Clone)]
  struct MyPackage {
    position: Position,
  }

  impl BotPackage for MyPackage {
    fn describe<P: BotPackage>(bot: &nurtex::bot::Bot<P>) -> Self {
      Self {
        position: bot.components.position,
      }
    }
  }

  #[tokio::test]
  async fn launch_bot() -> io::Result<()> {
    let bot = create_bot_with_package::<MyPackage>("NurtexBot");

    let username = bot.account.username.clone();
    let transmitter = bot.get_transmitter();

    tokio::spawn(async move {
      let mut receiver = transmitter.subscribe();

      while let Ok(package) = receiver.recv().await {
        println!("Позиция бота {}: {:?}", username, package.position);
      }
    });

    let mut event_invoker = EventInvoker::new();

    event_invoker.on_spawn(|terminal| async move {
      println!("Бот {} заспавнился!", terminal.account.username);
    });

    event_invoker.on_chunk_loaded(|terminal, payload| async move {
      println!("Загружен новый чанк для {} ({}, {})", terminal.account.username, payload.x, payload.z);
    });

    event_invoker.on_update_rotation(|terminal, payload| async move {
      println!("Ротация бота {} обновлена: {:?} -> {:?}", terminal.account.username, payload.old_rotation, payload.rotation);
    });

    event_invoker.on_chat(move |terminal, payload| async move {
      println!("Бот {} получил сообщение: {}", terminal.account.username, payload.message);

      if payload.message.contains("reconnect") {
        terminal.reconnect("localhost", 25565, 100).await;
      }
    });

    event_invoker.on_disconnect(|terminal, payload| async move {
      println!("Бот {} отключился по причине: {}", terminal.account.username, payload.reason);
    });

    bot.set_event_invoker(event_invoker).connect_to("localhost", 25565).await?;

    Ok(())
  }
}
