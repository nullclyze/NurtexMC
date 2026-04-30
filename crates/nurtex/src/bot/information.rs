use crate::protocol::packets::configuration::ServersideClientInformation;
use crate::protocol::types::{AccurateHand, ChatMode, DisplayedSkinParts, ParticleStatus};

/// Структура информации клиента
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClientInfo {
  pub locale: String,
  pub view_distance: i8,
  pub chat_mode: ChatMode,
  pub chat_colors: bool,
  pub displayed_skin_parts: DisplayedSkinParts,
  pub main_hand: AccurateHand,
  pub enable_text_filtering: bool,
  pub allow_server_listings: bool,
  pub particle_status: ParticleStatus,
}

impl Default for ClientInfo {
  fn default() -> Self {
    Self {
      locale: "en_US".to_string(),
      view_distance: 8,
      chat_mode: ChatMode::Enabled,
      chat_colors: true,
      displayed_skin_parts: DisplayedSkinParts::default(),
      main_hand: AccurateHand::Right,
      enable_text_filtering: false,
      allow_server_listings: true,
      particle_status: ParticleStatus::Minimal,
    }
  }
}

impl ClientInfo {
  /// Метод конвертации информации клиента в `Serverside` пакет
  pub fn to_serverside_packet(&self) -> ServersideClientInformation {
    ServersideClientInformation {
      locale: self.locale.clone(),
      view_distance: self.view_distance,
      chat_mode: self.chat_mode.clone(),
      chat_colors: self.chat_colors,
      displayed_skin_parts: self.displayed_skin_parts,
      main_hand: self.main_hand,
      enable_text_filtering: self.enable_text_filtering,
      allow_server_listings: self.allow_server_listings,
      particle_status: self.particle_status.clone(),
    }
  }
}
