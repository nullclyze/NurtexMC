use std::io;

use nurtex::create_swarm;
use nurtex::swarm::SwarmObject;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём объекты роя
  let mut objects = Vec::new();

  for i in 0..5 {
    objects.push(SwarmObject::new(format!("bot_{}", i)));
  }

  // Запускаем рой ботов на сервер с интервалом в 1000 мс
  create_swarm(objects).launch("localhost", 25565, 1000).await;

  Ok(())
}
