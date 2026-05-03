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
- [x] SOCKS4 proxy support
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
- [nurtex-proxy](https://github.com/NurtexMC/nurtex/tree/main/crates/nurtex-proxy): A crate for creating connections via SOCKS5 / SOCKS4 proxy.


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
    .with_join_delay(JoinDelay::fixed(500))
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

  for s_ind in 0..3 {
    // Создаём список ботов
    let mut bots = Vec::new();

    for b_ind in 0..2 {
      // Создаём бота и добавляем его в список
      bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
    }

    // Добавляем рой в кластер
    cluster.add_swarm(bots, JoinDelay::fixed(1000), "localhost", 25565);
  }

  // Запускаем кластер и ожидаем завершения всех хэндлов
  cluster.launch_and_wait().await
}
```