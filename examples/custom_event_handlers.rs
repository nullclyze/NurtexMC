use std::io;

use nurtex::bot::events::EventInvoker;
use nurtex::create_bot;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём бота
  let bot = create_bot("NurtexBot");

  // Создаём инициатор событий
  let mut event_invoker = EventInvoker::new();

  // Устанавливаем обработчик события "spawn"
  event_invoker.on_spawn(|terminal| async move {
    println!("Бот {} заспавнился!", terminal.account.username);
  });

  // Устанавливаем обработчик события "chat"
  event_invoker.on_chat(|terminal, payload| async move {
    println!("Бот {} получил сообщение: {}", terminal.account.username, payload.message);
  });

  // Устанавливаем обработчик события "death"
  event_invoker.on_death(|terminal| async move {
    println!("Бот {} умер.", terminal.account.username);
  });

  // Устанавливаем обработчик события "disconnect"
  event_invoker.on_disconnect(|terminal, payload| async move {
    println!("Бот {} отключился по причине: {}", terminal.account.username, payload.reason);
  });

  bot
    .set_event_invoker(event_invoker) // Задаём инициатор событий
    .connect_to("localhost", 25565)
    .await
}
