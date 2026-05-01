use std::time::Duration;

use nurtex::bot::{Bot, BotChatExt};
use nurtex::proxy::Proxy;
use nurtex_proxy::ProxyType;

/// Адрес SOCKS5 прокси, например `164.85.71.8:1080`
const PROXY_ADDRESS: &str = "YOUR_SOCKS5_PROXY";

/// Хост **публичного** сервера, например `mc.server.com`
const SERVER_HOST: &str = "PUBLIC_HOST";

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота с SOCKS5 прокси
  let proxy = Proxy::new(PROXY_ADDRESS, ProxyType::Socks5);
  let mut bot = Bot::create_with_proxy("nurtex_bot", proxy);

  // Подключаем бота к публичному серверу
  bot.connect(SERVER_HOST, 25565);

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(8)).await;

  // Отправляем сообщение в чат
  bot.chat_message(format!("Привет, я {}!", bot.username())).await?;

  // Ожидаем окончания хэндла подключения
  bot.wait_handle().await
}
