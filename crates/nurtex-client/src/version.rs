/// Метод получения протокола по версии Minecraft
pub fn get_protocol_from_version(version: &str) -> i32 {
  match version {
    "1.21.11" => 774,
    "1.21.10" => 773,
    "1.21.9" => 772,
    "1.21.8" => 771,
    _ => 774,
  }
}
