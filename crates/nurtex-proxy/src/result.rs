use crate::error::ProxyError;

/// Вспомогательный тип для результата операций
pub type ProxyResult<T> = Result<T, ProxyError>;
