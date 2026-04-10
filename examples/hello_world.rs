use std::io;

use nurtex::create_bot;
use nurtex::utils::time::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём бота
  let mut bot = create_bot("NurtexBot");

  // Клонируем терминал для отдельной задачи
  let terminal = bot.get_terminal();

  tokio::spawn(async move {
    // Ждём пока бот подключится
    sleep(5000).await;

    // Отправляем сообщение в чат
    terminal.chat("Привет, мир!").await;
  });

  // Подключаем бота к серверу
  bot.connect_to("localhost", 25565).await
}
