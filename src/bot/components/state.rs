/// Структура данных состояния
#[derive(Debug)]
pub struct State {
  pub health: f32,
  pub satiety: u32,
  pub saturation: f32,
}

impl Default for State {
  fn default() -> Self {
    Self {
      health: 0.0,
      satiety: 0,
      saturation: 0.0,
    }
  }
}
