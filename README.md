# NurtexMC

**NurtexMC** is a library written in Rust that allows you to create Minecraft bots and manage them, including connection and packet processing. This library focuses on an asynchronous environment, maximum speed and optimization, and ease of coding.

Supported Minecraft version: `1.21.11` (or protocol version - `774`).

# Usage

To use this library in your code, add a dependency to the Cargo.toml:

```
nurtex = { git = "https://github.com/nullclyze/NurtexMC" }
```

# Examples

## Creating a bot

```rust
use std::io;

use nurtex::core::bot::{Bot, BotCommand};
use nurtex::utils::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating a bot and its terminal.
  let (mut bot, terminal) = Bot::new("NurtexBot", Uuid::nil());

  // Spawn an asynchronous task.
  tokio::spawn(async move {
    sleep(3000).await; // Wait for the bot to connect.
    terminal.send(BotCommand::Chat("Hello, world!".to_string())).await; // Send a message to the chat.
    sleep(5000).await; // Wait a little.
    terminal.send(BotCommand::Disconnect).await; // Disconnect bot.
  });

  // Connecting bot to the server.
  bot.connect_to("localhost", 25565).await?;

  Ok(())
}
```

## Creating a swarm

```rust
use std::io;

use nurtex::{create_shared_swarm, launch_shared_swarm};
use nurtex::core::bot::BotCommand;
use nurtex::core::swarm::BotConfig;
use nurtex::utils::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating bot configs.
  let mut configs = Vec::new();

  for i in 0..4 {
    let config = BotConfig::new(format!("bot_{}", i));
    configs.push(config);
  }

  // Creating a shared-swarm of bots.
  let swarm = create_shared_swarm(configs);

  // Starting the swarm without blocking the thread.
  launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 500);

  sleep(4000).await; // Waiting for all the bots to connect.
  swarm.read().await.send(BotCommand::Chat("Hello, world!".to_string())).await; // Send a message to the chat from all bots.
  sleep(5000).await; // Wait a little.
  swarm.write().await.destroy().await; // Clear and destroy swarm.

  Ok(())
}
``` 