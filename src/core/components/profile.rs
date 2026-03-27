#[derive(Debug)]
pub struct Profile {
  pub entity_id: Option<i32>,
  pub game_mode: String,
  pub display_name: Option<String>,
  pub ping: i32,
}

impl Default for Profile {
  fn default() -> Self {
    Self {
      entity_id: None,
      display_name: None,
      game_mode: "unknown".to_string(),
      ping: 0,
    }
  }
}
