use nurtex::Bot;
use nurtex::bot::handlers::Handlers;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём обработчики
  let mut handlers = Handlers::new();

  // Устанавливаем обработчик на событие "spawn"
  handlers.on_spawn(async |username| {
    println!("Бот {} заспавнился!", username);
    Ok(())
  });

  // Устанавливаем обработчик на событие "chat"
  handlers.on_chat(async |username, payload| {
    println!("Бот {} получил сообщение: {}", username, payload.message);
    Ok(())
  });

  // Создаём бота с обработчиками и подключаем его к серверу
  Bot::create("nurtex_bot").with_handlers(handlers).connect_with_handle("localhost", 25565).await?
}
