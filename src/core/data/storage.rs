use std::sync::Arc;

use azalea_core::position::Vec3;
use azalea_entity::LookDirection;
use hashbrown::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Storage {
  /// Список всех сущностей, в ключе (i32) указывается ID сущности
  pub entities: HashMap<i32, Entity>,
}

impl Storage {
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),
    }
  }
}

pub type StorageLock = Arc<RwLock<Storage>>;

#[derive(Debug)]
pub struct Entity {
  /// Тип сущности
  pub entity_type: String,

  /// UUID сущнности
  pub uuid: Uuid,

  /// Позиция сущности (x, y, z)
  pub position: Vec3,

  /// Скорость сущности (x, y, z)
  pub velocity: Vec3,

  /// Направление взгляда сущности (y_rot, x_rot)
  pub look_direction: LookDirection,

  /// Физическое состояние `on_ground` сущности
  pub on_ground: bool,

  /// Информация игрока, если сущность **НЕ является** игроком - None
  pub player_info: Option<PlayerInfo>,
}

#[derive(Debug)]
pub struct PlayerInfo {
  /// Юзернейм игрока
  pub username: String,

  /// Режим игры игрока, например, "creative"
  pub game_mode: String,

  /// Пинг игрока
  pub ping: i32,
}
