use std::time::Duration;

use nurtex::Cluster;
use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() {
  // Создаём кластер
  let mut cluster = Cluster::create();

  // Создаём цикл на 3 повторения
  for i in 0..3 {
    // Добавляем рои в кластер.
    // Важно: Нужно добавлять рои каждый раз после `shutdown` (выключения кластера)
    for s_ind in 0..6 {
      let mut swarm = Swarm::create().set_join_delay(JoinDelay::fixed(500)).bind("localhost", 25565);

      for b_ind in 0..3 {
        swarm.add_bot(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
      }

      cluster.add_swarm(swarm);
    }

    // Запускаем кластер
    cluster.launch();

    // Ждём немножко
    tokio::time::sleep(Duration::from_secs(6)).await;

    // Отключаем и очищаем кластер
    cluster.shutdown().await;

    // Ждём перед следующим запуском (за исключением последнего запуска)
    if i != 2 {
      tokio::time::sleep(Duration::from_secs(3)).await;
    }
  }
}
