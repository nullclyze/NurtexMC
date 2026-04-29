use std::time::Duration;

use nurtex::Cluster;
use nurtex::bot::{Bot, BotChatExt};
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём список роев
  let mut swarms = Vec::new();

  // Создаём 3 роя
  for s_ind in 0..3 {
    // Создаём рой
    let mut swarm = Swarm::create().set_join_delay(JoinDelay::fixed(1000)).bind("localhost", 25565);

    // Создаём 2 бота
    for b_ind in 0..2 {
      // Создаём бота и добавляем его в рой
      swarm.add_bot(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
    }

    // Добавляем рой в список
    swarms.push(swarm);
  }

  // Создаём кластер и добавляем в него рои
  let mut cluster = Cluster::create().with_swarms(swarms);

  // Запускаем кластер
  cluster.launch();

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(5)).await;

  // Проходимся параллельно по всем роям
  cluster.for_each_parallel(async |swarm| {
    // Проходимся параллельно по всем ботам
    swarm.for_each_parallel(async |bot| {
      // Отправляем в чат сообщение
      let _ = bot.chat_message(format!("Привет, я {}!", bot.username())).await;
    });
  });

  // Внось ждём немножко
  tokio::time::sleep(Duration::from_secs(5)).await;

  Ok(())
}
