use std::sync::Arc;

use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Speedometer, SpeedometerEvent, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём спидометр 
  let speedometer = Arc::new(Speedometer::new(100));

  // Создаём рой со спидометром
  let mut swarm = Swarm::create_with_speedometer(Arc::clone(&speedometer));

  // Добавляем ботов в рой
  for i in 0..50 {
    // Добавляем бота со спидометром
    swarm.add_bot(Bot::create_with_speedometer(format!("nurtex_bot_{}", i), Arc::clone(&speedometer)));
  }

  // Запускаем ботов на сервер с регрессивной линейной задержкой
  swarm.quiet_launch("localhost", 25565, JoinDelay::regressive_linear(5000, 50));

  // Подписываемся на события спидометра
  let mut speedometer_rx = speedometer.subscribe();

  // Создаём бесконечный цикл
  loop {
    if let Ok(event) = speedometer_rx.recv().await {
      match event {
        SpeedometerEvent::TimerTick { speed, boost } => {
          // Обрабатываем тик таймера
          println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
        }
        SpeedometerEvent::UpdatePeakSpeed(speed) => {
          // Обрабатываем пиковую скорость
          println!("Новая пиковая скорость: {} b/s", speed);
        }
        _ => {}
      }
    }
  }
}
