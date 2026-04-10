use std::io;

use azalea_entity::HumanoidArm;
use azalea_protocol::common::client_information::ClientInformation;
use azalea_protocol::packets::game::s_interact::InteractionHand;
use nurtex::bot::Bot;
use nurtex::bot::account::BotAccount;
use nurtex::bot::components::position::Position;
use nurtex::bot::components::rotation::Rotation;
use nurtex::bot::events::EventInvoker;
use nurtex::bot::options::{AutoReconnectPlugin, BotInformation, BotPlugins};
use nurtex::bot::terminal::BotCommand;
use nurtex::bot::transmitter::BotPackage;
use nurtex::swarm::SwarmObject;
use nurtex::utils::time::sleep;
use nurtex::{create_shared_swarm_with_package, launch_shared_swarm};

// Объявляем структуру кастомного пакета данных
#[derive(Clone)]
struct CustomPackage {
  username: String,
  position: Position,
  rotation: Rotation,
}

// Создаём логику описания кастомного пакета
impl BotPackage for CustomPackage {
  fn describe<P: BotPackage>(bot: &Bot<P>) -> Self {
    Self {
      username: bot.account.username.clone(),
      position: bot.components.position,
      rotation: bot.components.rotation,
    }
  }
}

#[tokio::main]
async fn main() -> io::Result<()> {
  // Создаём объекты роя
  let mut objects = Vec::new();

  for i in 0..5 {
    let account = BotAccount::new(format!("bot_{}", i));

    // Создаём EventInvoker
    let event_invoker = create_event_invoker();

    // Создаём объект роя и настраиваем его
    let object = SwarmObject::new(account)
      .set_plugins(BotPlugins {
        auto_reconnect: AutoReconnectPlugin {
          enabled: true,
          reconnect_delay: 1000,
        },
        ..Default::default()
      })
      .set_information(BotInformation {
        client: ClientInformation {
          main_hand: HumanoidArm::Left,
          ..Default::default()
        },
        ..Default::default()
      })
      .set_event_invoker(event_invoker)
      .set_transmitter_interval(2000)
      .set_use_shared_storage(i % 2 == 0);

    objects.push(object);
  }

  // Создаём shared-рой с указанием кастомного пакета
  let swarm = create_shared_swarm_with_package::<CustomPackage>(objects);

  // Инициализируем все передатчики роя
  swarm.read().await.for_each_transmitters(|transmitter| async move {
    let mut receiver = transmitter.subscribe();

    while let Ok(package) = receiver.recv().await {
      println!("------ Пакет данных бота {} ------", package.username);
      println!("Позиция: {:?}", package.position);
      println!("Ротация: {:?}", package.rotation);
    }
  });

  // Запускаем рой ботов на сервер
  launch_shared_swarm(swarm.clone(), "localhost", 25565, 1000);

  // Ждём пока все боты зайдут
  sleep(7000).await;

  // Берём RwLock защиту
  let swarm_guard = swarm.read().await;

  // Отправляем сообщение в чат от всех ботов
  swarm_guard
    .for_each_parallel(|terminal| async move {
      terminal.chat("Привет, мир!").await;
      terminal.send(BotCommand::SwingArm(InteractionHand::MainHand)).await;
    })
    .await;

  sleep(3000).await;

  // Переподключаем каждого 2 бота
  for (index, terminal) in swarm_guard.terminals.iter().enumerate() {
    if index % 2 == 0 {
      terminal.reconnect("localhost", 25565, 1000).await;
    }
  }

  // Дропаем защиту
  drop(swarm_guard);

  // Немного ждём
  sleep(5000).await;

  // Мгновенно очищаем и уничтожаем рой
  swarm.write().await.force_destroy();

  Ok(())
}

/// Вспомогательная функция создания EventInvoker
fn create_event_invoker() -> EventInvoker {
  let mut event_invoker = EventInvoker::new();

  event_invoker.on_spawn(|terminal| async move {
    println!("Бот {} заспавнился!", terminal.account.username);
  });

  event_invoker.on_chat(|terminal, payload| async move {
    println!("Бот {} получил сообщение: {}", terminal.account.username, payload.message);
  });

  event_invoker.on_death(|terminal| async move {
    println!("Бот {} умер.", terminal.account.username);
  });

  event_invoker.on_disconnect(|terminal, payload| async move {
    println!("Бот {} отключился по причине: {}", terminal.account.username, payload.reason);
  });

  event_invoker
}
