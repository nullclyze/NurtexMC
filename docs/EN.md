# Basics

## How to include the `nurtex` library

You need to add the following to your `Cargo.toml`:

```toml
[dependencies]
nurtex = "1.0.0" # May be a different version
```

Or type in the terminal:

```bash
cargo add nurtex
```

## First program

Once the `nurtex` library has been successfully installed and included, you can move on to writing the **first** program.

Here we'll write a simple **Minecraft** bot whose task will be **simply connecting** to a server (in this example, we'll connect to a local server using version `1.21.11`).

**In the `main.rs` file:**
```rust
use nurtex::Bot;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create our bot
  let mut bot = Bot::create("nurtex_bot");

  // Connect the bot to the local server
  bot.connect("localhost", 25565);

  // Simply wait for the bot's handle (connection process) to complete
  bot.wait_handle().await
}
```

This code can also be simplified:

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create our bot and immediately connect to the server and get the
  // connection handle, then simply wait for it to complete
  nurtex::Bot::create("nurtex_bot")
    .connect_with_handle("localhost", 25565)
    .await?
}
```

## Sending messages to chat

Let's write a bot whose task is to connect to the server and send a specific message to chat 5 times.

**In the `main.rs` file:**
```rust
use std::time::Duration;

use nurtex::Bot;
use nurtex::bot::BotChatExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create our bot
  let mut bot = Bot::create("nurtex_bot");

  // Connect the bot to the local server
  bot.connect("localhost", 25565);

  // It is recommended to wait a bit after using the
  // `connect` method, as it does not block the current thread
  // while connecting (meaning we can start interacting
  // with the bot before it is fully connected to the server,
  // and this can lead to unexpected problems/errors)
  tokio::time::sleep(Duration::from_secs(3)).await;

  // Create a 5-item loop
  for _ in 0..5 {
    // Send a message to the chat
    bot.chat_message(format!("Hello, I'm {}!", bot.username())).await?;

    // Wait a bit
    tokio::time::sleep(Duration::from_secs(2)).await;
  }

  // Don't exit, wait for the handle to complete
  bot.wait_handle().await
}
```

## Creating a swarm

The `nurtex` library allows you to create a swarm (army, group - it doesn't matter) of bots, allowing you to effectively work with several (2 or more) bots simultaneously. One of the features of a swarm is shared storage of world data: instead of using unique storage for each bot, a swarm consolidates all data in one place, thereby avoiding situations with insane RAM consumption.

Here we'll write a minimal program using a swarm. Its task will be to launch several bots on the server at once and simply wait for them to close.

**In the `main.rs` file:**
```rust
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create our swarm
  let mut swarm = Swarm::create();

  // Create 5 bots and add them to the swarm
  for i in 0..5 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Connect the swarm to the server with a 500ms connection interval
  swarm.launch("localhost", 25565, JoinDelay::fixed(500)).await;

  // Wait for handles to complete
  swarm.wait_handles().await;

  Ok(())
}
```

# Features

The `nurtex` library has certain **features**. With each update of this library, we try to add new capabilities that will make working with it easier. Here we will look at some of the available features.

## Bot plugins

The bot has built-in plugins that are used to **automatically perform** certain actions.

Let's look at an example of using the `AutoReconnect` plugin:

```rust
use nurtex::Bot;
use nurtex::bot::plugins::{AutoReconnectPlugin, BotPlugins};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create our bot with the `AutoReconnect` plugin
  let mut bot = Bot::create("nurtex_bot")
    .set_plugins(BotPlugins {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true, // Enable the plugin,
        reconnect_delay: 2000, // Reconnection delay in ms
        max_attempts: 5, // Maximum number of reconnection attempts
      },
      ..Default::default() // Leave the rest as is
    });

  // Connect the bot to the local server
  bot.connect("localhost", 25565);

  // Wait for handle completion
  bot.wait_handle().await
}
```

You can test the plugin's functionality by typing the following command in the chat:

```
/kick nurtex_bot test
```

## Swarm join delay

I decided not to stop at a simple static join delay (`JoinDelay`) and made it as flexible as possible.

Let's first look at several methods for creating a join delay:

- `JoinDelay::fixed(delay)`: Fixed delay.
- `JoinDelay::progressive_linear(delay, max_delay)`: Progressive linear delay.
- `JoinDelay::random(min_delay, max_delay)`: Random delay in the specified range.
- `JoinDelay::custom(func)`: Custom delay creation function.
- ...

Let's create a swarm using progressive linear join delay:

```rust
use nurtex::{Bot, JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create a swarm
  let mut swarm = Swarm::create();

  // Add 6 bots to the swarm
  for i in 0..6 {
  swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Launch a swarm with progressive linear delay
  swarm.launch("localhost", 25565, JoinDelay::progressive_linear(500, 4000)).await;

  // Wait for handles to complete
  swarm.wait_handles().await;

  Ok(())
}
```

What it will look like:

```
nurtex_bot_0 connected
Waiting for 500ms...
nurtex_bot_1 connected
Waiting for 1000ms...
nurtex_bot_2 connected
Waiting for 1500ms...
nurtex_bot_3 connected
Waiting for 2000ms...
nurtex_bot_4 connected
Waiting for 2500ms...
nurtex_bot_5 connected
```

## Swarm with a speedometer

The speedometer is another feature of the `nurtex` library. It allows you to measure the speed at which bots from a swarm are launched.

Let's create a swarm with a speedometer and get the startup speed (speed is measured in bps, or bots per second):

```rust
use std::sync::Arc;

use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Speedometer, SpeedometerEvent, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Create a speedometer
  let speedometer = Arc::new(Speedometer::new(100));

  // Create a swarm with a speedometer
  let mut swarm = Swarm::create_with_speedometer(Arc::clone(&speedometer));

  // Add 20 bots to the swarm to see speed changes
  for i in 0..20 {
    // Create a bot with a speedometer
    let speedometer = Arc::clone(&speedometer);
    let bot = Bot::create_with_speedometer(format!("nurtex_bot_{}", i), speedometer);

    // Add the bot to the swarm
    swarm.add_bot(bot);
  }

  // Launch bots on the server with regressive linear delay
  swarm.quiet_launch("localhost", 25565, JoinDelay::regressive_linear(3000, 25));

  // Subscribe to speedometer events
  let mut speedometer_rx = speedometer.subscribe();

  // Create an infinite loop where we process speedometer events
  loop {
    if let Ok(event) = speedometer_rx.recv().await {
      match event {
        SpeedometerEvent::TimerTick { speed, boost } => {
          println!("Fixed speed: {} b/s (boost: {})", speed, boost);
        }
        SpeedometerEvent::UpdatePeakSpeed(speed) => {
          println!("New peak speed: {} b/s", speed);
        }
        _ => {}
      }
    } 
  }
}
```

After running the program, you will get an output similar to this:

```
New peak speed: 1 b/s
Fixed speed: 1 b/s (boost: 1)
Fixed speed: 0 b/s (boost: -1)
Fixed speed: 0 b/s (boost: 0)
Fixed speed: 1 b/s (boost: 1)
Fixed speed: 1 b/s (boost: 0)
Fixed speed: 1 b/s (boost: 0)
New peak speed: 2 b/s
Fixed speed: 2 b/s (boost: 1)
Fixed speed: 2 b/s (boost: 0)
New peak speed: 3 b/s
Fixed speed: 3 b/s (boost: 1)
New peak speed: 4 b/s
Fixed speed: 4 b/s (boost: 1)
New peak speed: 5 b/s
Fixed speed: 5 b/s (boost: 1)
```