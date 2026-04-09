use azalea_core::position::Vec3;

/// Структура данных скорости
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Velocity {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Velocity {
  /// Метод создания скорости с заданными значениями
  pub fn new(x: f64, y: f64, z: f64) -> Self {
    Self { x, y, z }
  }

  /// Метод создания нулевой скорости
  pub fn zero() -> Self {
    Self { x: 0.0, y: 0.0, z: 0.0 }
  }

  /// Метод получения дельты
  pub fn delta(&self, other: Self) -> Self {
    let dx = self.x - other.x;
    let dy = self.x - other.y;
    let dz = self.x - other.z;

    Self { x: dx, y: dy, z: dz }
  }

  /// Метод конвертировки Velocity в Vec3
  pub fn to_vec3(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }

  /// Метод конвертировки Vec3 в Velocity
  pub fn from_vec3(vector: Vec3) -> Self {
    Self {
      x: vector.x,
      y: vector.y,
      z: vector.z,
    }
  }
}
