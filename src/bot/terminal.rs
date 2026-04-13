use azalea_protocol::packets::game::{ServerboundGamePacket, s_interact::InteractionHand};
use tokio::sync::mpsc;

/// Команда для терминала бота
#[derive(Clone, Debug)]
pub enum BotCommand {
  Chat(String),
  SetDirection { yaw: f32, pitch: f32 },
  SetPosition { x: f64, y: f64, z: f64 },
  SwingArm(InteractionHand),
  StartUseItem(InteractionHand),
  ReleaseUseItem,
  SendPacket(ServerboundGamePacket),
  Disconnect,
  Reconnect { server_host: String, server_port: u16, interval: u64 },
}

/// Терминал бота, используется для отправки команд.
///
/// Пример использования:
/// ```rust, ignore
/// // Создаём бота
/// let mut bot = create_bot("NurtexBot");
///
/// // Клонируем терминал до запуска бота
/// let terminal = bot.terminal.clone();
///
/// // Спавним отдельную асинхронную задачу перед запуском бота
/// tokio::spawn(async move {
///   // Ждём пока бот подключится
///   tokio::time::sleep(Duration::from_millis(5000)).await;
///
///   // Получаем юзернейм бота через терминал
///   let username = terminal.username;
///
///   // Отправляем сообщение в чат при помощи терминала
///   terminal.chat(format!("Привет, мир! Мой юзернейм: {}", username)).await;
/// });
///
/// // Подключаем бота к серверу
/// bot.connect_to("server.com", 25565).await?;
/// ```
#[derive(Clone)]
pub struct BotTerminal {
  /// Юзернейм бота
  pub username: String,

  /// Отправитель команд
  pub cmd: mpsc::Sender<BotCommand>,
}

impl BotTerminal {
  /// Метод отправки команды в терминал
  pub async fn send(&self, command: BotCommand) {
    let _ = self.cmd.send(command).await;
  }

  /// Вспомогательный метод отправки команды Chat в терминал
  pub async fn chat(&self, message: impl Into<String>) {
    self.send(BotCommand::Chat(message.into())).await;
  }

  /// Вспомогательный метод отправки команды Disconnect в терминал
  pub async fn disconnect(&self) {
    self.send(BotCommand::Disconnect).await;
  }

  /// Вспомогательный метод отправки команды Disconnect в терминал
  pub async fn reconnect(&self, server_host: impl Into<String>, server_port: u16, interval: u64) {
    self
      .send(BotCommand::Reconnect {
        server_host: server_host.into(),
        server_port,
        interval,
      })
      .await;
  }
}
