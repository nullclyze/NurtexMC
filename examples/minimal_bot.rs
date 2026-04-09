use std::io;

use nurtex::create_bot;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём бота и подключаем его к серверу
  create_bot("NurtexBot").connect_to("localhost", 25565).await
}
