/// Структура данных авторизации прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyAuth {
  username: String,
  password: String,
}

impl ProxyAuth {
  /// Метод создания нового экземпляра `ProxyAuth`
  pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
    Self {
      username: username.into(),
      password: password.into(),
    }
  }

  /// Метод получения юзернейма
  pub fn username(&self) -> &str {
    &self.username
  }

  /// Метод получения пароля
  pub fn password(&self) -> &str {
    &self.password
  }
}
