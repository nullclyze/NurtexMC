use std::io;

use nurtex::bot::account::BotAccount;
use nurtex::create_swarm;
use nurtex::swarm::SwarmObject;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём объекты роя
  let mut objects = Vec::new();

  for i in 0..5 {
    let account = BotAccount::new(format!("bot_{}", i));
    objects.push(SwarmObject::new(account));
  }

  // Запускаем рой ботов на сервер с интервалом в 1000 мс
  create_swarm(objects).launch("localhost", 25565, 1000).await;

  Ok(())
}
