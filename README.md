# NurtexMC

**NurtexMC** is a library written in Rust that allows you to create Minecraft bots and manage them, including connection and packet processing. This library focuses on an asynchronous environment, maximum speed and optimization, and ease of coding.

Supported Minecraft version: `1.21.11` (or protocol version - `774`).

# Usage

To use this library in your code, add a dependency to the Cargo.toml:

```
nurtex = "0.2.0"
```

# Examples

## Creating a bot

```rust
use std::io;

use nurtex::bot::Bot;
use nurtex::events::EventInvoker;
use nurtex::common::BotCommand;

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating a bot
  let bot = Bot::new("NurtexBot");

  // Create an event invoker
  let mut event_invoker = EventInvoker::new();

  // Сreate a handler for the "spawn" event
  event_invoker.on_spawn(|terminal| async move {
    terminal.chat("Hello, world!").await;
  });

  // Сreate a handler for the "chat" event
  event_invoker.on_chat(|terminal, payload| async move {
    if payload.message.contains("disconnect") {
      // Disconnect bot
      terminal.send(BotCommand::Disconnect).await;
    }
  });

  bot
    .set_event_invoker(event_invoker) // Set event invoker
    .connect_to("localhost", 25565) // Connect bot to server
    .await?;

  Ok(())
}
```

## Creating a swarm

```rust
use std::io;
use std::time::Duration;

use nurtex::common::BotCommand;
use nurtex::swarm::SwarmObject;
use nurtex::time::sleep;
use nurtex::{create_shared_swarm, destroy_shared_swarm, launch_shared_swarm};

#[tokio::main]
async fn main() -> io::Result<()> {
  // Creating swarm objects
  let mut objects = Vec::new();

  for i in 0..=5 {
    let object = SwarmObject::new(format!("bot_{}", i));
    objects.push(object);
  }

  // Creating a shared-swarm of bots
  let swarm = create_shared_swarm(objects);

  // Starting the swarm without blocking the thread
  launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 500);

  sleep(8000).await; // Waiting for all the bots to connect

  {
    let guard = swarm.read().await;

    // Send a message to the chat from all bots
    guard.send(BotCommand::Chat("Hello, world!".to_string())).await;
  }

  sleep(5000).await; // Wait a little

  // Clear and destroy swarm
  destroy_shared_swarm(swarm, Duration::from_secs(5)).await?;

  Ok(())
}
``` 