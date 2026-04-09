use std::io;

use nurtex::bot::Bot;
use nurtex::bot::account::BotAccount;
use nurtex::bot::components::position::Position;
use nurtex::bot::components::rotation::Rotation;
use nurtex::bot::transmitter::BotPackage;
use nurtex::bot::world::StorageLock;

// Объявляем структуру кастомного пакета данных
#[derive(Clone, Debug)]
struct CustomPackage {
  position: Position,
  rotation: Rotation,
  on_ground: bool,
  local_storage: StorageLock,
}

// Создаём логику описания кастомного пакета
impl BotPackage for CustomPackage {
  fn describe<P: BotPackage>(bot: &Bot<P>) -> Self {
    Self {
      position: bot.components.position,
      rotation: bot.components.rotation,
      on_ground: bot.physics.on_ground,
      local_storage: bot.local_storage.clone(),
    }
  }
}

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём бота и явно указываем его тип
  let account = BotAccount::new("NurtexBot");
  let bot: Bot<CustomPackage> = Bot::create(account);

  // Получаем передатчик пакетов для отдельной задачи
  let transmitter = bot.get_transmitter();

  // Спавним отдельную задачу для обработки пакетов
  tokio::spawn(async move {
    // Подписываемся на передачу пакетов
    let mut receiver = transmitter.subscribe();

    // Обрабатываем пакеты
    while let Ok(package) = receiver.recv().await {
      println!("------ Получен новый пакет данных ------");
      println!("Позиция: {:?}", package.position);
      println!("Ротация: {:?}", package.rotation);
      println!("Флаг on_ground: {}", package.on_ground);
      println!("Кол-во сущностей: {}", package.local_storage.read().await.entities.len());
    }
  });

  bot
    .set_transmitter_interval(500) // Задаём интервал передатчика (опционально)
    .connect_to("localhost", 25565)
    .await
}
