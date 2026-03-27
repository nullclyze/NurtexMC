#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::{create_shared_swarm, launch_shared_swarm};
  use nurtex::core::bot::BotCommand;
  use nurtex::core::swarm::BotConfig;
  use nurtex::utils::sleep;

  #[tokio::test]
  async fn launch_swarm() -> io::Result<()> {
    let mut configs = Vec::new();

    for i in 0..4 {
      let config = BotConfig::new(format!("bot_{}", i));
      configs.push(config);
    }

    let swarm = create_shared_swarm(configs);

    launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 500);

    sleep(4000).await;
    swarm.read().await.send(BotCommand::Chat("Test".to_string())).await;
    sleep(5000).await; 
    swarm.read().await.send(BotCommand::Disconnect).await;

    Ok(())
  }
}