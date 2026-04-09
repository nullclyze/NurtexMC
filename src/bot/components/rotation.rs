use azalea_entity::LookDirection;

/// Структура данных ротации
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rotation {
  pub yaw: f32,
  pub pitch: f32,
}

impl Rotation {
  /// Метод создания ротации с заданными значениями
  pub fn new(yaw: f32, pitch: f32) -> Self {
    Self { yaw, pitch }
  }

  /// Метод создания нулевой ротации
  pub fn zero() -> Self {
    Self { yaw: 0.0, pitch: 0.0 }
  }

  /// Метод получения дельты
  pub fn delta(&self, other: Self) -> Self {
    let d_yaw = self.yaw - other.yaw;
    let d_pitch = self.pitch - other.pitch;

    Self { yaw: d_yaw, pitch: d_pitch }
  }

  /// Метод конвертировки Rotation в LookDirection
  pub fn to_look_direction(&self) -> LookDirection {
    LookDirection::new(self.yaw, self.pitch)
  }

  /// Метод конвертировки LookDirection в Rotation
  pub fn from_look_direction(look_direction: LookDirection) -> Self {
    Self {
      yaw: look_direction.y_rot(),
      pitch: look_direction.x_rot(),
    }
  }
}
