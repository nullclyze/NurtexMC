use nurtex::bot::Bot;
use nurtex::protocol::connection::ClientsidePacket;
use nurtex::protocol::packets::play::ClientsidePlayPacket;
use nurtex::swarm::Swarm;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой и список читателей пакетов
  let mut swarm = Swarm::create().bind("localhost", 25565);
  let mut packet_readers = Vec::new();

  // Инициализируем ботов и читателей пакетов
  for i in 0..6 {
    let bot = Bot::create(format!("nurtex_bot_{}", i));
    packet_readers.push((bot.subscribe_to_reader(), bot.username().to_string()));
    swarm.add_bot(bot);
  }

  // Мгновенно запускаем ботов на сервер (без задержки)
  swarm.instant_launch();

  // Проходимся по всем читателям
  for (mut packet_rx, username) in packet_readers {
    // Обязательно спавним отдельную задачу, чтобы не блокировать поток
    tokio::spawn(async move {
      loop {
        // Берём пакеты только из Play состояния
        if let Ok(ClientsidePacket::Play(packet)) = packet_rx.recv().await {
          // Обрабатываем пакеты
          match packet {
            ClientsidePlayPacket::KeepAlive(p) => {
              println!("Бот {} получил KeepAlive с ID {}", username, p.id);
            }
            ClientsidePlayPacket::PlayerChat(p) => {
              println!("Бот {} получил сообщение: {}", username, p.message);
            }
            _ => {}
          }
        }
      }
    });
  }

  // Просто ждём завершения всех хэндлов
  swarm.wait_handles().await;

  Ok(())
}
