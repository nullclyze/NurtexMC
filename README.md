# Nurtex

A collection of lightweight Rust libraries for creating Minecraft bots. Async, optimized, ease of coding.

> [!WARNING]
> All crates are currently in **early** development. The API may change frequently, and may be unstable, have frequent errors, and have limited functionality. Bugs or features should be reported to the GitHub **issues**.


# Focusing

All crates focus on:

- **Speed:** All crates do not contain heavy, long-term operations, even if they do, they are optimized.
- **Lightweight:** All crates do not carry any extra dependencies or code.
- **Asynchrony:** Almost all crates rely on an asynchronous code environment.
- **Simplicity:** We try to make the logic of crates understandable for everyone.


# Tasks and Goals

- [x] Bot architecture
- [x] Swarm architecture
- [x] Cluster architecture
- [x] Connecting to servers
- [ ] SOCKS4 proxy support
- [x] SOCKS5 proxy support
- [ ] HTTP(S) proxy support
- [x] Login processing
- [x] Configuration processing
- [ ] Play processing (a partial implementation already exists)
- [x] Auxiliary functionality (plugins, speedometer, just functions and methods)
- [ ] Implementation of physics
- [ ] Interaction with inventory
- [ ] Interaction with entities
- [ ] Storing world data (a small part implemented)
- [x] Flexible settings (relative to the current position)
- [ ] NBT parsing
- [ ] Text component parsing (it's there now, but it doesn't work as it should)
- [ ] Basic bypass of client validity checks (planned to be implemented soon)
- [ ] Bypass primitive bot checks
- [ ] Bypass complex bot checks (complete imitation of a real player)
- [ ] Bypass captchas


# Crate map

- [nurtex](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex): A crate for high-level work with the bot / swarm API.
- [nurtex-codec](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-codec): A crate for serializing Minecraft types into Rust types.
- [nurtex-derive](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-derive): A crate for convenient parsing of network packets.
- [nurtex-encrypt](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-encrypt): A crate containing the Minecraft TCP-connection encryption.
- [nurtex-protocol](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-protocol): A crate for creating Minecraft TCP-connections and working with packets.
- [nurtex-proxy](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-proxy): A crate for creating connections via SOCKS5 proxy.


# Documentation

[**Русская**](https://github.com/NurtexMC/nurtex/tree/main/docs/RU.md) | [**English**](https://github.com/NurtexMC/nurtex/tree/main/docs/EN.md)


# Examples

All current examples can be found here: [browse](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex/examples)


## Create a bot

This is one of the simplest examples of creating and connecting a bot.

```rust
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
```


## Create a swarm

In this example, you can see a simple implementation of a bot swarm.

```rust
use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой
  let mut swarm = Swarm::create();

  // Добавляем ботов в рой
  for i in 0..6 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Запускаем ботов на сервер с фиксированной задержкой в 500мс
  swarm.launch("localhost", 25565, JoinDelay::fixed(500)).await;

  // Ждём завершения всех хэндлов ботов
  swarm.wait_handles().await;

  Ok(())
}
```


## Swarm and speedometer

Here you can see how to properly use the speedometer in a swarm and obtain its statistics / data.

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

  // Добавляем ботов в рой
  for i in 0..50 {
    // Добавляем бота со спидометром
    swarm.add_bot(Bot::create_with_speedometer(format!("nurtex_bot_{}", i), Arc::clone(&speedometer)));
  }

  // Запускаем ботов на сервер с регрессивной линейной задержкой
  swarm.quiet_launch("localhost", 25565, JoinDelay::regressive_linear(5000, 50));

  // Подписываемся на события спидометра
  let mut speedometer_rx = speedometer.subscribe();

  // Создаём бесконечный цикл
  loop {
    if let Ok(event) = speedometer_rx.recv().await {
      match event {
        SpeedometerEvent::TimerTick { speed, boost } => {
          // Обрабатываем тик таймера
          println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
        }
        SpeedometerEvent::UpdatePeakSpeed(speed) => {
          // Обрабатываем пиковую скорость
          println!("Новая пиковая скорость: {} b/s", speed);
        }
        _ => {}
      }
    }
  }
}
```