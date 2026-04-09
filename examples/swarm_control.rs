use std::io;

use nurtex::bot::account::BotAccount;
use nurtex::utils::time::sleep;
use nurtex::{create_shared_swarm, destroy_shared_swarm, launch_shared_swarm};
use nurtex::swarm::SwarmObject;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём объекты роя
  let mut objects = Vec::new();

  for i in 0..5 {
    let account = BotAccount::new(format!("bot_{}", i));
    objects.push(SwarmObject::new(account));
  }

  // Создаём shared-рой
  let swarm = create_shared_swarm(objects);

  // Запускаем рой ботов на сервер
  launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 1000);

  // Ждём пока все боты зайдут
  sleep(7000).await;

  // Берём RwLock защиту
  let swarm_guard = swarm.read().await;

  swarm_guard.for_each(|terminal| async move {
    terminal.chat("Привет, мир!").await;
  });

  sleep(2000).await;

  swarm_guard.for_each(|terminal| async move {
    terminal.chat("Переподключаюсь...").await;
    terminal.reconnect("localhost", 25565, 500).await;
  });

  sleep(5000).await;

  // Дропаем защиту
  drop(swarm_guard);

  // Уничтожаем рой ботов
  destroy_shared_swarm(swarm).await?;

  Ok(())
}
