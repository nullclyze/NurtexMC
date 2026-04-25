/// Структура плагина `AutoRespawn`
#[derive(Clone)]
pub struct AutoRespawnPlugin {
  pub enabled: bool,
  pub respawn_delay: u64,
}

impl Default for AutoRespawnPlugin {
  fn default() -> Self {
    Self { enabled: true, respawn_delay: 0 }
  }
}
