use tokio::task::JoinHandle;

/// Структура хэндла бота
pub struct BotHandle {
  pub connection_handle: Option<JoinHandle<core::result::Result<(), std::io::Error>>>,
  pub reader_handle: Option<JoinHandle<()>>,
  pub writer_handle: Option<JoinHandle<()>>,
}

impl Default for BotHandle {
  fn default() -> Self {
    Self {
      connection_handle: None,
      reader_handle: None,
      writer_handle: None,
    }
  }
}

impl BotHandle {
  /// Метод отмены всех хэндлов
  pub fn abort(&self) {
    if let Some(h) = &self.reader_handle {
      h.abort();
    }

    if let Some(h) = &self.writer_handle {
      h.abort();
    }

    if let Some(h) = &self.connection_handle {
      h.abort();
    }
  }
}
