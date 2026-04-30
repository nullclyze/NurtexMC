use std::time::Duration;

use nurtex::bot::Bot;
use nurtex::protocol::packets::play::{ServersidePlayPacket, ServersideSwingArm};
use nurtex::protocol::types::RelativeHand;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к серверу
  bot.connect("localhost", 25565);

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(3)).await;

  // Запускаем цикл на 6 повторений
  for _ in 0..6 {
    // Отправляем пакет SwingArm
    bot.send_packet(ServersidePlayPacket::SwingArm(ServersideSwingArm { hand: RelativeHand::MainHand }));

    // Ждём 1 секунду
    tokio::time::sleep(Duration::from_secs(1)).await;
  }

  Ok(())
}
