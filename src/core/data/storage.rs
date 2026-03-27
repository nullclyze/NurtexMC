use azalea_core::position::Vec3;
use azalea_entity::LookDirection;
use hashbrown::HashMap;
use uuid::Uuid;

pub struct Storage {
  pub entities: HashMap<i32, Entity>
}

#[derive(Debug)]
pub struct Entity {
  pub entity_type: String,
  pub uuid: Uuid,
  pub position: Vec3,
  pub velocity: Vec3,
  pub look_direction: LookDirection,
  pub on_ground: bool,
  pub player_info: Option<PlayerInfo>
}

#[derive(Debug)]
pub struct PlayerInfo {
  pub username: String,
  pub game_mode: String,
  pub ping: i32
}

impl Default for Storage {
  fn default() -> Self {
    Self {
      entities: HashMap::new()
    }
  }
}