use std::time::{SystemTime, UNIX_EPOCH};

/// Вспомогательная функция для создания временного ожидания в мс
pub async fn sleep(ms: u64) {
  tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

/// Вспомогательная функция получения UNIX timestamp
pub fn timestamp() -> u64 {
  match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(d) => d.as_secs(),
    Err(_) => 0,
  }
}
