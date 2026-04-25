use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;

/// Спидометр, отвечающий за измерение скорости (бот / сек)
pub struct Speedometer {
  event_tx: Arc<broadcast::Sender<SpeedometerEvent>>,
  total_joined: Arc<AtomicU32>,
  last_bots_per_second: Arc<AtomicU32>,
  peak_bots_per_second: Arc<AtomicU32>,
  update_handle: Option<JoinHandle<()>>,
}

/// События спидометра
#[derive(Clone, Debug)]
pub enum SpeedometerEvent {
  /// Событие о подключении бота
  BotJoined(String),

  /// Событие об обновлении скорости (бот / сек)
  UpdateSpeed(u32),

  /// Событие таймера (отправляется каждую секунду)
  TimerTick { speed: u32, boost: i32 },

  /// Событие об обновлении пиковой скорости (бот / сек)
  UpdatePeakSpeed(u32),
}

impl Speedometer {
  /// Метод создания нового спидометра
  pub fn new(channel_capacity: usize) -> Self {
    let (event_tx, _) = broadcast::channel(channel_capacity);

    let mut speedometer = Self {
      event_tx: Arc::new(event_tx),
      total_joined: Arc::new(AtomicU32::new(0)),
      last_bots_per_second: Arc::new(AtomicU32::new(0)),
      peak_bots_per_second: Arc::new(AtomicU32::new(0)),
      update_handle: None,
    };

    speedometer.run_timer();

    speedometer
  }

  /// Метод получения отправителя событий
  pub fn get_event_sender(&self) -> Arc<broadcast::Sender<SpeedometerEvent>> {
    Arc::clone(&self.event_tx)
  }

  /// Метод подписки на события спидометра
  pub fn subscribe(&self) -> broadcast::Receiver<SpeedometerEvent> {
    self.event_tx.subscribe()
  }

  /// Метод отправки события о подключении бота
  pub fn bot_joined(&self, username: String) {
    self.total_joined.fetch_add(1, Ordering::SeqCst);
    let _ = self.event_tx.send(SpeedometerEvent::BotJoined(username));

    let bps = self.total_joined.load(Ordering::SeqCst);

    let _ = self.event_tx.send(SpeedometerEvent::UpdateSpeed(bps));

    if bps > self.peak_bots_per_second.load(Ordering::SeqCst) {
      let _ = self.event_tx.send(SpeedometerEvent::UpdatePeakSpeed(bps));
      self.peak_bots_per_second.store(bps, Ordering::SeqCst);
    }
  }

  /// Метод получения пиковой скорости (бот / сек)
  pub fn get_peak_speed(&self) -> u32 {
    self.peak_bots_per_second.load(Ordering::SeqCst)
  }

  /// Метод запуска таймера, который сбрасывает статистику каждую секунду
  pub fn run_timer(&mut self) {
    let event_tx = Arc::clone(&self.event_tx);
    let last_bots_per_second = Arc::clone(&self.last_bots_per_second);
    let total_joined = Arc::clone(&self.total_joined);
    let retainer_duration = Duration::from_millis(1000);

    let handle = tokio::spawn(async move {
      loop {
        let bps = total_joined.load(Ordering::SeqCst);
        let last_bps = last_bots_per_second.load(Ordering::SeqCst);

        let _ = event_tx.send(SpeedometerEvent::TimerTick {
          speed: bps,
          boost: bps as i32 - last_bps as i32,
        });

        if bps != last_bps {
          last_bots_per_second.store(bps, Ordering::SeqCst);
        }

        total_joined.store(0, Ordering::SeqCst);

        tokio::time::sleep(retainer_duration).await;
      }
    });

    self.update_handle = Some(handle);
  }

  /// Метод остановки и сброса спидометра
  pub fn stop(&self) {
    if let Some(handle) = &self.update_handle {
      handle.abort();
    }

    self.last_bots_per_second.store(0, Ordering::SeqCst);
    self.peak_bots_per_second.store(0, Ordering::SeqCst);
    self.total_joined.store(0, Ordering::SeqCst);
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::sync::Arc;
  use std::time::Duration;

  use crate::{
    bot::Bot,
    swarm::{JoinDelay, Speedometer, SpeedometerEvent, Swarm},
  };

  #[tokio::test]
  async fn test_instant() -> io::Result<()> {
    let speedometer = Arc::new(Speedometer::new(100));

    let mut swarm = Swarm::create_with_speedometer(Arc::clone(&speedometer));

    for i in 0..300 {
      swarm.add_bot(Bot::create_with_speedometer(format!("bot_{}", i), Arc::clone(&speedometer)));
    }

    let mut speedometer_events = speedometer.subscribe();

    tokio::spawn(async move {
      loop {
        if let Ok(event) = speedometer_events.recv().await {
          match event {
            SpeedometerEvent::UpdateSpeed(speed) => {
              println!("Скорость: {} b/s", speed);
            }
            SpeedometerEvent::TimerTick { speed, boost } => {
              println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
            }
            SpeedometerEvent::UpdatePeakSpeed(speed) => {
              println!("Новая пиковая скорость: {} b/s", speed);
            }
            _ => {}
          }
        }
      }
    });

    swarm.instant_launch("localhost", 25565);

    tokio::time::sleep(Duration::from_secs(20)).await;

    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_normal() -> io::Result<()> {
    let speedometer = Arc::new(Speedometer::new(100));

    let mut swarm = Swarm::create_with_speedometer(Arc::clone(&speedometer));

    for i in 0..100 {
      swarm.add_bot(Bot::create_with_speedometer(format!("bot_{}", i), Arc::clone(&speedometer)));
    }

    let mut speedometer_events = speedometer.subscribe();

    tokio::spawn(async move {
      loop {
        if let Ok(event) = speedometer_events.recv().await {
          match event {
            SpeedometerEvent::TimerTick { speed, boost } => {
              println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
            }
            SpeedometerEvent::UpdatePeakSpeed(speed) => {
              println!("Новая пиковая скорость: {} b/s", speed);
            }
            _ => {}
          }
        }
      }
    });

    swarm.launch("localhost", 25565, JoinDelay::fixed(100)).await;

    tokio::time::sleep(Duration::from_secs(20)).await;

    swarm.shutdown().await?;

    Ok(())
  }
}
