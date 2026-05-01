# Основа

## Как подключить библиотеку `nurtex`

Нужно добавить в ваш `Cargo.toml`:

```toml
[dependencies]
nurtex = "1.1.0" # Может быть другая версия
```

Или написать в терминале:

```bash
cargo add nurtex
```


## Первая программа

После того как библиотека `nurtex` была успешна установленна и подключена, можно перейти к написанию **первой** программы.

Давай напишем простого **Minecraft** бота, задачей которого будет **просто подключиться** к серверу (в данном примере будет рассматриваться подключение к локальному серверу используя версию `1.21.11`):

```rust
use nurtex::Bot;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём нашего бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к локальному серверу
  bot.connect("localhost", 25565);

  // Просто ожидаем завершения хэндла (процесса подключения) бота
  bot.wait_handle().await
}
```

Так же данный код можно упростить:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём нашего бота и сразу подключем к серверу и получаем хэндл
  // подключения, после просто ожидаем его завершения
  nurtex::Bot::create("nurtex_bot")
    .connect_with_handle("localhost", 25565)
    .await?
}
```


## Отправка сообщений в чат

Теперь напишем бота, который подключиться к серверу и 5 раз отправит определённое сообщение в чат:

```rust
use std::time::Duration;

use nurtex::Bot;
use nurtex::bot::BotChatExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём нашего бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к локальному серверу
  bot.connect("localhost", 25565);

  // Рекомендуется немного подождать после использования метода
  // `connect`, так как он не блокирует текущий поток
  // на время подключения (то есть мы можем начать взаимодействовать
  // с ботом до того как он полностью подключится к серверу,
  // и это может привести к нежданным проблемам / ошибкам)
  tokio::time::sleep(Duration::from_secs(3)).await;

  // Создаём цикл на 5 повторений
  for _ in 0..5 {
    // Отправляем сообщение в чат
    bot.chat_message(format!("Привет, я {}!", bot.username())).await?;

    // Немного ждём
    tokio::time::sleep(Duration::from_secs(2)).await;
  }

  // Не выходим, ожидаем завершения хэндла
  bot.wait_handle().await
}
```


## Создание роя

Библиотека `nurtex` позволяет создавать рой (армию, группу - не важно) ботов, он позволяет эффективно работать сразу с несколькими (2 и более) ботами. Одной и особенностью роя является общее хранение данных о мире: вместо того чтобы использовать уникальные хранилища для каждого бота, рой объеденяет все данные в одном месте, тем самым избегая ситуаций с безумным потреблением оперативной памяти.

Давай напишем минимальную программу с использованием роя. Её задачей будет запустить сразу несколько ботов на сервер и просто ожидать закрытия:

```rust
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём наш рой
  let mut swarm = Swarm::create();

  // Создадим 5 ботов и добавим их в рой
  for i in 0..5 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Подключим рой к серверу с интервалом подключения в 500мс
  swarm.launch("localhost", 25565, JoinDelay::fixed(500)).await;

  // Ожидаем завершения хэндлов
  swarm.wait_handles().await;

  Ok(())
}
```


## Актуальные примеры

Все актуальные примеры можно найти здесь: [смотреть](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex/examples)


# Особенности

Библиотека `nurtex` имеет определённые **фичи** (особенности). С каждым обновлением данной библиотеки мы стараемся добавить новые возможности, которые упростят работу с ней. Здесь мы рассмотрим некоторые из доступных фич.


## Плагины бота

Бот имеет встроенные плагины, которые служат для **автоматического выполнения** определённых действий.

Давай рассмотрим пример использования плагина `AutoReconnect`:

```rust
use nurtex::Bot;
use nurtex::bot::plugins::{AutoReconnectPlugin, BotPlugins};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём нашего бота с плагином `AutoReconnect`
  let mut bot = Bot::create("nurtex_bot")
    .set_plugins(BotPlugins {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true, // Включаем плагин,
        reconnect_delay: 2000, // Задержка переподключения в мс
        max_attempts: 5, // Максимальное кол-во попыток переподключения
      },
      ..Default::default() // Остальное оставим так как есть
    });

  // Подключаем бота к локальному серверу
  bot.connect("localhost", 25565);

  // Ожидаем завершения хэндла
  bot.wait_handle().await
}
```

Проверить работоспособность плагина можно написав в чат команду:

```
/kick nurtex_bot test
```


## Задержка подключения роя

Я решил не останавливаться на простой статичной задержке подключения (`JoinDelay`) и сделал её максимально гибкой.

Изначально рассмотрим несколько методов создания задержки подключения:

- `JoinDelay::fixed(delay)`: Фиксированная задержка.
- `JoinDelay::progressive_linear(delay, max_delay)`: Прогрессивная линейная задержка.
- `JoinDelay::random(min_delay, max_delay)`: Случайная задержка в заданном диапозоне.
- `JoinDelay::custom(func)`: Своя функция создания задержки.
- ...

Давай создадим рой с использованием прогрессивной линейной задержкой подключения:

```rust
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой
  let mut swarm = Swarm::create();

  // Добавляем 6 ботов в рой
  for i in 0..6 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Запускаем рой с прогрессивной линейной задержкой
  swarm.launch("localhost", 25565, JoinDelay::progressive_linear(500, 4000)).await;

  // Ожидаем завершения хэндлов
  swarm.wait_handles().await;

  Ok(())
}
```

Как это будет выглядеть:

```
nurtex_bot_0 подключен
Ожидание 500 мс...
nurtex_bot_1 подключен
Ожидание 1000 мс...
nurtex_bot_2 подключен
Ожидание 1500 мс...
nurtex_bot_3 подключен
Ожидание 2000 мс...
nurtex_bot_4 подключен
Ожидание 2500 мс...
nurtex_bot_5 подключен
```


## Рой со спидометром

Спидометр явялется ещё одной особенностью библиотеки `nurtex`. Он позволяет замерять скорость запуска ботов из роя.

Давай создадим рой со спидометром и получим скорость запуска (скорость измеряется в b/s, или же ботов в секунду):

```rust
use std::sync::Arc;

use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Speedometer, SpeedometerEvent, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём спидометр
  let speedometer = Arc::new(Speedometer::new(100));

  // Создаём рой со спидометром
  let mut swarm = Swarm::create_with_speedometer(Arc::clone(&speedometer));

  // Добавляем 20 ботов в рой чтобы увидеть изменения скорости
  for i in 0..20 {
    // Создаём бота со спидометром
    let speedometer = Arc::clone(&speedometer);
    let bot = Bot::create_with_speedometer(format!("nurtex_bot_{}", i), speedometer);

    // Добавляем бота в рой
    swarm.add_bot(bot);
  }

  // Запускаем ботов на сервер с регрессивной линейной задержкой
  swarm.quiet_launch("localhost", 25565, JoinDelay::regressive_linear(3000, 25));

  // Подписываемся на события спидометра
  let mut speedometer_rx = speedometer.subscribe();

  // Создаём бесконечный цикл где обрабатываем события спидометра
  loop {
    if let Ok(event) = speedometer_rx.recv().await {
      match event {
        SpeedometerEvent::TimerTick { speed, boost } => {
          println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
        }
        SpeedometerEvent::UpdatePeakSpeed(speed) => {
          println!("Новая пиковая скорость: {} b/s", speed);
        }
        _ => {}
      }
    }
  }
}
```

После запуска программы получится примерно такой вывод:

```
Новая пиковая скорость: 1 b/s
Фиксированная скорость: 1 b/s (буст: 1)
Фиксированная скорость: 0 b/s (буст: -1)
Фиксированная скорость: 0 b/s (буст: 0)
Фиксированная скорость: 1 b/s (буст: 1)
Фиксированная скорость: 1 b/s (буст: 0)
Фиксированная скорость: 1 b/s (буст: 0)
Новая пиковая скорость: 2 b/s
Фиксированная скорость: 2 b/s (буст: 1)
Фиксированная скорость: 2 b/s (буст: 0)
Новая пиковая скорость: 3 b/s
Фиксированная скорость: 3 b/s (буст: 1)
Новая пиковая скорость: 4 b/s
Фиксированная скорость: 4 b/s (буст: 1)
Новая пиковая скорость: 5 b/s
Фиксированная скорость: 5 b/s (буст: 1)
```


# Дополнительно

## Чтение и запись пакетов

Библиотека `nurtex` позволяет читать (`ClientsidePacket`) и записывать (`ServersidePlayPacket`) пакеты.

Давай создадим бота, который при каждом полученном пакете `PlayerChat` будет проверять наличие слова `swing` в сообщение и отправлять на сервер пакет `SwingArm`. Так мы в одном примере сразу сможем посмотреть как работает чтение и запись пакетов:

```rust
use nurtex::bot::Bot;
use nurtex_protocol::connection::ClientsidePacket;
use nurtex_protocol::packets::play::{ClientsidePlayPacket, ServersidePlayPacket, ServersideSwingArm};
use nurtex_protocol::types::RelativeHand;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к серверу
  bot.connect("localhost", 25565);

  // Получаем и подписываемся на читателя пакетов
  let mut packet_rx = bot.get_reader().subscribe();

  // Запускаем бесконечный цикл
  loop {
    // Читаем пакеты только в Play состоянии
    if let Ok(ClientsidePacket::Play(packet)) = packet_rx.recv().await {
      match packet {
        ClientsidePlayPacket::PlayerChat(p) => {
          // Проверяем что сообщение содержит слово `swing`
          if p.message.contains("swing") {
            // Отправляем пакет `SwingArm`
            bot.send_packet(ServersidePlayPacket::SwingArm(ServersideSwingArm {
              hand: RelativeHand::MainHand,
            }));

            // Так же можно отправлять пакеты таким способом:
            // let _ = bot.get_writer().send(ServersidePlayPacket::SwingArm(ServersideSwingArm {
            //  hand: RelativeHand::MainHand,
            // }));
          }
        }
        _ => {}
      }
    }
  }
}
```