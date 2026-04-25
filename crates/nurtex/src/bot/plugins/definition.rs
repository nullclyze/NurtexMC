use crate::bot::plugins::AutoRespawnPlugin;

/// Структура плагинов бота
#[derive(Clone)]
pub struct BotPlugins {
  pub auto_respawn: AutoRespawnPlugin,
}

impl Default for BotPlugins {
  fn default() -> Self {
    Self {
      auto_respawn: AutoRespawnPlugin::default(),
    }
  }
}
