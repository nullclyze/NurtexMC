/// Структура данных опыта
#[derive(Debug, Clone)]
pub struct Experience {
  pub level: u32,
  pub progress: f32,
  pub total: u32,
}

impl Default for Experience {
  fn default() -> Self {
    Self {
      level: 0,
      progress: 0.0,
      total: 0,
    }
  }
}
