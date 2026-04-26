use nurtex::bot::Bot;
use nurtex_protocol::connection::ClientsidePacket;
use nurtex_protocol::packets::play::ClientsidePlayPacket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к серверу
  bot.connect("localhost", 25565);

  // Получаем и подписываемся на читателя пакетов
  let mut packet_rx = bot.get_reader().subscribe();

  // Запускаем бесконечный цикл
  loop {
    // Обрабатываем только пакеты в состоянии Play
    if let Ok(ClientsidePacket::Play(packet)) = packet_rx.recv().await {
      match packet {
        ClientsidePlayPacket::PlayerChat(p) => {
          println!("Бот {} получил сообщение: {}", bot.username(), p.message);
        }
        ClientsidePlayPacket::KeepAlive(p) => {
          println!("Бот {} получил KeepAlive с ID {}", bot.username(), p.id);
        }
        _ => {}
      }
    }
  }
}
