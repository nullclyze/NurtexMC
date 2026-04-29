use nurtex::Cluster;
use nurtex::bot::Bot;
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

  // Создаём кластер и сразу запускаем его
  Cluster::create().with_swarms(swarms).launch_and_wait().await
}
