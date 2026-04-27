use nurtex_protocol::types::{Rotation, Vector3};
use uuid::Uuid;

/// Сущность мира
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
  pub entity_type: i32,
  pub entity_uuid: Uuid,
  pub position: Vector3,
  pub rotation: Rotation,
  pub velocity: Vector3,
  pub on_ground: bool,
}

impl Default for Entity {
  fn default() -> Self {
    Self {
      entity_type: -1,
      entity_uuid: Uuid::nil(),
      position: Vector3::zero(),
      rotation: Rotation::zero(),
      velocity: Vector3::zero(),
      on_ground: false,
    }
  }
}
