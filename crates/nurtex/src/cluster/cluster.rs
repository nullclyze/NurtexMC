use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::Swarm;

/// Кластер роев из ботов
pub struct Cluster {
  /// Список роев
  swarms: Vec<Arc<Swarm>>,

  /// Список хэндлов
  handles: Vec<JoinHandle<()>>,
}

impl Cluster {
  /// Метод создания нового скопления
  pub fn create() -> Self {
    Self {
      swarms: Vec::new(),
      handles: Vec::new(),
    }
  }

  /// Метод создания нового скопление с заданной ёмкостью
  pub fn create_with_capacity(capacity: usize) -> Self {
    Self {
      swarms: Vec::with_capacity(capacity),
      handles: Vec::with_capacity(capacity),
    }
  }

  /// Метод добавления роя
  pub fn add_swarm(&mut self, swarm: Swarm) {
    self.swarms.push(Arc::new(swarm));
  }

  /// Метод добавления списка роев
  pub fn add_swarms(&mut self, swarms: Vec<Swarm>) {
    for swarm in swarms {
      self.swarms.push(Arc::new(swarm));
    }
  }

  /// Метод добавления роя (возвращает `Self`)
  pub fn with_swarm(mut self, swarm: Swarm) -> Self {
    self.swarms.push(Arc::new(swarm));
    self
  }

  /// Метод добавления списка роев (возвращает `Self`)
  pub fn with_swarms(mut self, swarms: Vec<Swarm>) -> Self {
    for swarm in swarms {
      self.swarms.push(Arc::new(swarm));
    }

    self
  }

  /// Метод получения роя по его индексу
  pub fn get_swarm(&self, swarm_id: usize) -> Option<Arc<Swarm>> {
    if let Some(swarm) = self.swarms.get(swarm_id) { Some(Arc::clone(swarm)) } else { None }
  }

  /// Метод получения всех роев
  pub fn get_all_swarms(&self) -> Vec<Arc<Swarm>> {
    let mut swarms = Vec::with_capacity(self.swarms.len());

    for swarm in &self.swarms {
      swarms.push(Arc::clone(swarm));
    }

    swarms
  }

  /// Метод запуска кластера
  pub fn launch(&mut self) {
    for swarm in &self.swarms {
      self.handles.push(swarm.quiet_launch());
    }
  }

  /// Метод запуска кластера и ожидания завершения хэндлов
  pub async fn launch_and_wait(&mut self) -> std::io::Result<()> {
    for swarm in &self.swarms {
      self.handles.push(swarm.quiet_launch());
    }

    self.wait_finish().await
  }

  /// Метод запуска опредлённого роя из кластера
  pub fn launch_swarm(&mut self, swarm_id: i32) {
    for (id, swarm) in self.swarms.iter().enumerate() {
      if id as i32 != swarm_id {
        continue;
      }

      self.handles.push(swarm.quiet_launch());
    }
  }

  /// Последовательный for-each
  pub async fn for_each_consistent<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<Swarm>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    for swarm in &self.swarms {
      f(Arc::clone(swarm)).await;
    }
  }

  /// Параллельный for-each
  pub fn for_each_parallel<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<Swarm>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
  {
    let f = Arc::new(f);

    for swarm in &self.swarms {
      let f_clone = Arc::clone(&f);
      let swarm_clone = Arc::clone(swarm);

      tokio::spawn(f_clone(swarm_clone));
    }
  }

  /// Метод ожидания завершения всех хэндлов
  pub async fn wait_finish(&mut self) -> std::io::Result<()> {
    for handle in &mut self.handles {
      handle.await?;
    }

    Ok(())
  }

  /// Метод полной очистки и остановки кластера
  pub async fn shutdown(&mut self) {
    self.abort_handles();
    self.swarms.clear();
    self.handles.clear();
  }

  /// Метод отмены хэндлов
  pub fn abort_handles(&self) {
    for handle in &self.handles {
      if !handle.is_finished() {
        handle.abort();
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use crate::{Bot, Cluster, JoinDelay, Swarm};

  #[tokio::test]
  async fn test_minimal_cluster() -> std::io::Result<()> {
    let mut cluster = Cluster::create();

    for si in 0..3 {
      let mut swarm = Swarm::create().set_join_delay(JoinDelay::fixed(5000)).bind("localhost", 25565);

      for bi in 0..2 {
        swarm.add_bot(Bot::create(format!("nurtex_{}_{}", si, bi)));
      }

      cluster.add_swarm(swarm);
    }

    cluster.launch();

    tokio::time::sleep(Duration::from_secs(5)).await;

    cluster.wait_finish().await
  }
}
