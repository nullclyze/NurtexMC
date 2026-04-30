/// Список имён ошибок прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorName {
  Timeout,
  NotConnected,
  InvalidData,
  InvalidVersion,
  AuthFailed,
  Unsupported,
  UnknownError,
}

/// Структура ошибки прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyError {
  /// Имя ошибки
  name: ErrorName,

  /// Текст ошибки
  text: String,
}

impl ProxyError {
  /// Метод создания новой ошибки
  pub fn new(name: ErrorName, text: impl Into<String>) -> Self {
    Self { name: name, text: text.into() }
  }

  /// Метод получения имени ошибки
  pub fn name(&self) -> ErrorName {
    self.name.clone()
  }

  /// Метод получения текста ошибки
  pub fn text(&self) -> String {
    self.text.clone()
  }
}

impl From<std::io::Error> for ProxyError {
  fn from(error: std::io::Error) -> Self {
    Self {
      name: ErrorName::UnknownError,
      text: error.to_string(),
    }
  }
}
