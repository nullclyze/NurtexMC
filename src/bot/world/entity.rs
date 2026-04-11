use uuid::Uuid;

use crate::bot::components::{position::Position, rotation::Rotation, velocity::Velocity};

/// Сущность мира
#[derive(Debug, Clone)]
pub struct Entity {
  /// Тип сущности
  pub entity_type: String,

  /// UUID сущнности
  pub uuid: Uuid,

  /// Позиция сущности (x, y, z)
  pub position: Position,

  /// Скорость сущности (x, y, z)
  pub velocity: Velocity,

  /// Ротация сущности (y, x)
  pub rotation: Rotation,

  /// Физическое состояние `on_ground` сущности
  pub on_ground: bool,

  /// Информация игрока, если сущность **НЕ является** игроком - None
  pub player_info: Option<PlayerInfo>,
}

/// Информация об игроке
#[derive(Debug, Clone)]
pub struct PlayerInfo {
  /// Юзернейм игрока
  pub username: String,

  /// Режим игры игрока, например, "creative"
  pub game_mode: String,

  /// Пинг игрока
  pub ping: i32,
}
