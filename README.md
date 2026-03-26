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

use nurtex::create_bot;
use nurtex::core::bot::BotPlugins;
use nurtex::core::terminal::Command;
use nurtex::utils::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Create a bot and its terminal.
  let (mut bot, terminal) = create_bot("NurtexBot", BotPlugins::default());

  // Connecting the bot to the server.
  tokio::spawn(async move {
    let _ = bot.connect_to("localhost", 25565).await;
  });

  sleep(3000).await; // Wait for the bot to connect.

  // Send a message to the chat from the bot through the terminal.
  terminal.send(Command::Chat("Hello, world!".to_string())).await;

  sleep(5000).await; // Wait a little.

  // Disconnect the bot.
  terminal.send(Command::Disconnect).await;

  Ok(())
}
```

## Creating a swarm

```rust
use std::io;

use nurtex::{create_shared_swarm, launch_shared_swarm};
use nurtex::core::swarm::SwarmObject;
use nurtex::core::terminal::Command;
use nurtex::utils::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating swarm objects.
  let mut objects = Vec::new();

  for i in 0..5 {
    let object = SwarmObject::new(format!("bot_{}", i));
    objects.push(object);
  }

  // Creating a swarm and bots.
  let (swarm, bots) = create_shared_swarm(objects);

  // Launch the swarm.
  launch_shared_swarm(swarm.clone(), bots, "localhost".to_string(), 25565, 500);

  sleep(4000).await; // Waiting for all the bots to connect.

  // Send a message from all bots to the chat.
  swarm
    .read()
    .await
    .send(Command::Chat("Hello, world!".to_string()))
    .await;

  sleep(5000).await; // Wait a little.

  // Clean and destroy the swarm.
  swarm.write().await.destroy().await;

  Ok(())
}
``` 