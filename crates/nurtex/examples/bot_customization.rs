use nurtex::bot::Bot;
use nurtex::bot::plugins::{AutoReconnectPlugin, BotPlugins};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота и настраиваем его
  let mut bot = Bot::create("nurtex_bot")
    .set_connection_timeout(10000) // Задаём таймаут подключения
    .set_plugins(BotPlugins {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true, // Включаем плагин (по умолчанию выключен)
        reconnect_delay: 2000, // Задержка переподключения в мс
        max_attempts: -1, // Бесконечные попытки на переподключение
      },
      ..Default::default()
    });

  // Подключаем бота к серверу и ожидаем окончания хэндла подключения.
  // Чтобы проверить работоспособность плагина AutoReconnect, можно
  // написать в чате `/kick nurtex_bot test` и подождить 2 секунды
  bot.connect("localhost", 25565);
  bot.wait_handle().await
}
