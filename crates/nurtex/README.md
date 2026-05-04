# Nurtex

This is a library written in Rust that allows you to create Minecraft bots and manage them, including connection and packet processing. This library focuses on an asynchronous environment, maximum speed and optimization, and ease of coding.

Supported Minecraft version: `1.21.11` (or protocol version - `774`).


# Features

- **Speed:** All operations in this library work efficiently and are limited by timeouts.
- **Lightweight:** Library is quite lightweight and does not have any unnecessary dependencies.
- **Swarm architecture:** Allows you to work effectively with large groups of bots while consuming little RAM.
- **Cluster architecture:** Allows you to simultaneously run swarms of bots on different servers (multi-target).
- **Flexibility:** Library has flexible settings (relative to current capabilities).
- **Proxy support:** Library supports connecting bots via SOCKS5 / SOCKS4 proxies.
- **Asynchrony:** Library relies on an asynchronous environment.


# Documentation

[**Русская**](https://github.com/NurtexMC/nurtex/tree/main/docs/RU.md) | [**English**](https://github.com/NurtexMC/nurtex/tree/main/docs/EN.md)


# Examples

All current examples can be found here: [browse](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex/examples)


## Create a bot

This is one of the simplest examples of creating and connecting a bot.

```rust
use nurtex::Bot;
use nurtex::bot::BotChatExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота
  let mut bot = Bot::create("nurtex_bot");

  // Подключаем бота к серверу
  bot.connect("localhost", 25565);

  // Ждём немножко
  tokio::time::sleep(std::time::Duration::from_secs(3)).await;

  // Отправляем сообщение в чат
  bot.chat_message("Привет, мир!").await?;

  // Ожидаем окончания хэндла подключения
  bot.wait_handle().await
}
```


## Create a swarm

In this example, you can see a simple implementation of a bot swarm.

```rust
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() {
  // Создаём список ботов
  let mut bots = Vec::new();

  // Добавляем ботов в наш список
  for i in 0..6 {
    bots.push(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Создаём рой и запускаем его на сервер
  Swarm::create()
    .with_bots(bots)
    .цшер_join_delay(JoinDelay::fixed(500))
    .bind("localhost", 25565)
    .launch_and_wait()
    .await
}
```


## Create a cluster

Here you can see a minimal example of creating a cluster.

```rust
use nurtex::{Bot, Cluster, JoinDelay};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём кластер
  let mut cluster = Cluster::create();

  // Создаём 3 роя
  for s_ind in 0..3 {
    let mut bots = Vec::new();

    // Создаём 2 бота
    for b_ind in 0..2 {
      // Создаём бота и добавляем его в список
      bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
    }

    cluster.add_swarm(bots, JoinDelay::fixed(1000), "localhost", 25565);
  }

  // Запускаем кластер и ожидаем завершения всех хэндлов
  cluster.launch_and_wait().await
}
```