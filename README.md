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

use nurtex::core::bot::Bot;
use nurtex::utils::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating a bot and its terminal.
  let (mut bot, terminal) = Bot::new("NurtexBot");

  // Spawn an asynchronous task.
  tokio::spawn(async move {
    sleep(3000).await; // Wait for the bot to connect.

    // Send a message to the chat.
    terminal.chat("Hello, world!").await; 

    sleep(5000).await; // Wait a little.

    // Disconnect bot.
    terminal.disconnect().await; 
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
use nurtex::core::swarm::SwarmObject;
use nurtex::utils::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating swarm objects.
  let mut objects = Vec::new();

  for i in 0..4 {
    let object = SwarmObject::new(format!("bot_{}", i));
    objects.push(object);
  }

  // Creating a shared-swarm of bots.
  let swarm = create_shared_swarm(objects);

  // Starting the swarm without blocking the thread.
  launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 500);

  sleep(4000).await; // Waiting for all the bots to connect.

  // Send a message to the chat from all bots.
  swarm.read().await.send(BotCommand::Chat("Hello, world!".to_string())).await; 

  sleep(5000).await; // Wait a little.

  // Clear and destroy swarm.
  swarm.write().await.destroy().await; 

  Ok(())
}
``` 