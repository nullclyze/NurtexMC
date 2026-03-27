#[derive(Debug)]
pub struct Profile {
  pub entity_id: Option<i32>,
  pub game_mode: String,
  pub ping: i32,
}

impl Default for Profile {
  fn default() -> Self {
    Self {
      entity_id: None,
      game_mode: "unknown".to_string(),
      ping: 0,
    }
  }
}
