use std::time::Duration;

use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой
  let mut swarm = Swarm::create();

  // Добавляем ботов в рой
  for i in 0..6 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Запускаем ботов на сервер с фиксированной задержкой в 500мс
  swarm.launch("localhost", 25565, JoinDelay::fixed(500)).await;

  // Ждём 10 секунд и завершаем процесс
  tokio::time::sleep(Duration::from_secs(10)).await;

  Ok(())
}
