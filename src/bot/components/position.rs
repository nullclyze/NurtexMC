use azalea_core::position::Vec3;

use crate::bot::components::velocity::Velocity;

/// Структура данных позиции
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Position {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Position {
  /// Метод создания позиции с заданными координатами
  pub fn new(x: f64, y: f64, z: f64) -> Self {
    Self { x, y, z }
  }

  /// Метод создания нулевой позиции
  pub fn zero() -> Self {
    Self { x: 0.0, y: 0.0, z: 0.0 }
  }

  /// Метод получения дельты
  pub fn delta(&self, other: Self) -> Self {
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    let dz = self.z - other.z;

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

  /// Метод применения скорости к позиции
  pub fn apply_velocity(&mut self, velocity: Velocity) {
    self.x += velocity.x;
    self.y += velocity.y;
    self.z += velocity.z;
  }
}
