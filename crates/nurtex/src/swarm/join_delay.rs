use rand::Rng;

/// Тип функции для создания задержки подключения
pub type JoinDelayFn = dyn Fn(u64, u64) -> u64 + Send + Sync;

/// Гибкая задержка между подключением ботов
pub struct JoinDelay(Box<JoinDelayFn>);

impl JoinDelay {
  /// Метод активации функции создания задержки
  pub fn activate(&self, current: usize, total: usize) -> u64 {
    self.0(current as u64, total as u64)
  }

  /// Метод создания фиксированной задержки
  pub fn fixed(delay: u64) -> Self {
    Self(Box::new(move |_current, _total| delay))
  }

  /// Метод создания линейной прогрессивной задержки (увеличивается с каждым ботом до указанного предела)
  pub fn progressive_linear(delay: u64, max_delay: u64) -> Self {
    Self(Box::new(move |current, _total| {
      let result = delay * (current + 1);

      if result > max_delay { max_delay } else { result }
    }))
  }

  /// Метод создания линейной регрессивной задержки (уменьшается с каждым ботом до указанного предела)
  pub fn regressive_linear(delay: u64, min_delay: u64) -> Self {
    Self(Box::new(move |current, _total| {
      let result = delay / (current + 1);

      if result < min_delay { min_delay } else { result }
    }))
  }

  /// Метод создания кастомной функции создания задержки
  pub fn custom(func: Box<JoinDelayFn>) -> Self {
    Self(func)
  }

  /// Метод создания случайной задержки в заданном диапазоне
  pub fn random(min_delay: u64, max_delay: u64) -> Self {
    Self(Box::new(move |_current, _total| {
      let mut rng = rand::thread_rng();
      rng.gen_range(min_delay..=max_delay)
    }))
  }

  /// Метод создания промежуточной задержки (каждый бот не попадающий под условие
  /// `(current + 1) % group_size == 0` имеет задрежку `delay`, а бот, который попадает
  /// под это условие, имеет задержку `intermediate_delay`)
  pub fn intermediate(group_size: u64, delay: u64, intermediate_delay: u64) -> Self {
    Self(Box::new(move |current, _total| {
      // Тк используются индексы ботов (0 включительно) нужно добавлять по 1 к текущему индексу,
      // чтобы не было вылета первого бота из условий
      if (current + 1) % group_size == 0 { intermediate_delay } else { delay }
    }))
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use crate::bot::Bot;
  use crate::swarm::{JoinDelay, Swarm};

  #[tokio::test]
  async fn test_fixed_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);
    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::fixed(500)).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    swarm.shutdown().await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_progressive_linear_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::progressive_linear(500, 5000)).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_regressive_linear_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::regressive_linear(5000, 500)).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_random_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::random(100, 3000)).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_custom_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    let join_delay_fn = |current, total| (500 + total) * current;

    swarm.launch("localhost", 25565, JoinDelay::custom(Box::new(join_delay_fn))).await;

    tokio::time::sleep(Duration::from_secs(2)).await;
    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_intermediate_delay() -> io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch("localhost", 25565, JoinDelay::intermediate(2, 100, 2000)).await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    swarm.shutdown().await?;

    Ok(())
  }
}
