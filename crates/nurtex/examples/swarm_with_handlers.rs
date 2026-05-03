use nurtex::bot::handlers::Handlers;
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() {
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

  // Создаём рой
  let mut swarm = Swarm::create()
    .with_join_delay(JoinDelay::fixed(500))
    .with_handlers(handlers) // Устанавливаем обработчики
    .bind("localhost", 25565);

  // Добавляем ботов в рой.
  // Важно: При использовании обработчиков в рое, ботов
  // нужно добавлять строго после установки обработчиков в рой,
  // иначе боты, добавленные до установки обработчиков,
  // не будут их использовать
  for i in 0..6 {
    swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
  }

  // Запускаем рой
  swarm.launch().await;

  // Ожидаем завершения всех хэндлов
  swarm.wait_handles().await
}
