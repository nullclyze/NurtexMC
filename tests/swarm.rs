#[cfg(test)]
mod tests {
  use std::io;

  use nurtex::core::bot::BotCommand;
  use nurtex::core::swarm::SwarmObject;
  use nurtex::utils::sleep;
  use nurtex::{create_shared_swarm, launch_shared_swarm};

  #[tokio::test]
  async fn launch_swarm() -> io::Result<()> {
    let mut objects = Vec::new();

    for i in 0..=50 {
      let object = SwarmObject::new(format!("bot_{}", i));
      objects.push(object);
    }

    let swarm = create_shared_swarm(objects);

    launch_shared_swarm(swarm.clone(), "localhost".to_string(), 25565, 25);

    sleep(5000).await;
    swarm
      .read()
      .await
      .send(BotCommand::Chat("Test".to_string()))
      .await;

    sleep(5000).await;

    swarm.write().await.destroy().await;

    Ok(())
  }
}
