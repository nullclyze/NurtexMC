use crate::protocol::types::{Experience, Rotation, Vector3};

/// Структура компонентов бота
pub struct BotComponents {
  pub entity_id: i32,
  pub position: Vector3,
  pub velocity: Vector3,
  pub rotation: Rotation,
  pub health: f32,
  pub food: i32,
  pub experience: Experience,
}

impl Default for BotComponents {
  fn default() -> Self {
    Self {
      entity_id: -1,
      position: Vector3::zero(),
      velocity: Vector3::zero(),
      rotation: Rotation::zero(),
      health: -1.0,
      food: -1,
      experience: Experience::default(),
    }
  }
}
