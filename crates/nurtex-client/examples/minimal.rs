use std::io;

use nurtex_protocol::connection::ClientsidePacket;
use nurtex_protocol::packets::play::{ClientsidePlayPacket, ServersidePlayPacket};
use nurtex_client::Client;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём клиента
  let mut client = Client::create("NurtexBot", "1.21.11");

  // Подключаем клиента к серверу
  client.connect_to("localhost", 25565).await?;

  // Подписываемся на отправку пакетов
  let mut packet_rx = client.subscribe_to_reader();

  // Запускаем цикл обработки пакетов
  loop {
    if let Ok(p) = packet_rx.recv().await {
      match p {
        ClientsidePacket::Play(ClientsidePlayPacket::KeepAlive(p)) => {
          // Отвечаем на полученный KeepAlive
          client.send_packet(ServersidePlayPacket::KeepAlive(nurtex_protocol::packets::play::MultisideKeepAlive { id: p.id }));
        }
        _ => {}
      }
    }
  }
}
