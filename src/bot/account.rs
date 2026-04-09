use uuid::Uuid;

/// Аккаунт бота, содержит в себе `username` и `UUID` (по умолчанию нулевое)
#[derive(Debug)]
pub struct BotAccount {
  /// Юзернейм аккаунта (игровой ник)
  pub username: String,

  /// UUID аккаунта, по умолчанию нулевой
  pub uuid: Uuid,
}

impl BotAccount {
  /// Метод создания нового аккаунта
  pub fn new(username: impl Into<String>) -> Self {
    Self {
      username: username.into(),
      uuid: Uuid::nil(),
    }
  }

  /// Метод установки UUID для аккаунта
  pub fn set_uuid(mut self, uuid: Uuid) -> Self {
    self.uuid = uuid;
    self
  }
}
