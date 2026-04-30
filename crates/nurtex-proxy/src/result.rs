use crate::error::{ErrorName, ProxyError};

/// Вспомогательное перечисление результата
pub enum ProxyResult<T> {
  Success(T),
  Failed(ProxyError),
}

impl<T> From<core::result::Result<T, ProxyError>> for ProxyResult<T> {
  fn from(result: core::result::Result<T, ProxyError>) -> Self {
    match result {
      Ok(t) => return Self::Success(t),
      Err(e) => return Self::Failed(e),
    }
  }
}

impl<T> From<std::io::Result<T>> for ProxyResult<T> {
  fn from(result: std::io::Result<T>) -> Self {
    match result {
      Ok(t) => return Self::Success(t),
      Err(e) => return Self::Failed(e.into()),
    }
  }
}

impl<T> From<std::option::Option<T>> for ProxyResult<T> {
  fn from(value: std::option::Option<T>) -> Self {
    match value {
      Some(v) => ProxyResult::Success(v),
      None => ProxyResult::Failed(ProxyError::new(ErrorName::InvalidData, "option does not contain any data")),
    }
  }
}
