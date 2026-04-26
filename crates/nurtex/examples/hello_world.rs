use std::time::Duration;

use nurtex::bot::{Bot, BotChatExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к серверу
  bot.connect("localhost", 25565);

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(3)).await;

  // Отправляем сообщение в чат
  bot.chat_message("Привет, мир!").await?;

  // Ожидаем окончания хэндла подключения
  bot.wait_handle().await
}
