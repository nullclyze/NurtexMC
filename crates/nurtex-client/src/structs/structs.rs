use nurtex_protocol::packets::configuration::ServersideClientInformation;

/// Структура информации клиента
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClientInfo {
  pub locale: String,
  pub view_distance: i8,
  pub chat_mode: ChatMode,
  pub chat_colors: bool,
  pub displayed_skin_parts: DisplayedSkinParts,
  pub main_hand: ClientHand,
  pub enable_text_filtering: bool,
  pub allow_server_listings: bool,
  pub particle_status: ParticleStatus
}

impl Default for ClientInfo {
  fn default() -> Self {
    Self {
      locale: "en_US".to_string(),
      view_distance: 8,
      chat_mode: ChatMode::Enabled,
      chat_colors: true,
      displayed_skin_parts: DisplayedSkinParts::default(),
      main_hand: ClientHand::Right,
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
      chat_mode: self.chat_mode.id(),
      chat_colors: self.chat_colors,
      displayed_skin_parts: self.displayed_skin_parts.to_mask(),
      main_hand: self.main_hand.id(),
      enable_text_filtering: self.enable_text_filtering,
      allow_server_listings: self.allow_server_listings,
      particle_status: self.particle_status.id(),
    }
  }
}

/// Режим чата
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ChatMode {
  Enabled,
  CommandsOnly,
  Hidden
}

impl ChatMode {
  /// Метод получения идентификатора состояния видимости чата
  pub fn id(&self) -> i32 {
    match self {
      ChatMode::Enabled => 0,
      ChatMode::CommandsOnly => 1,
      ChatMode::Hidden => 2,
    }
  }
}

/// Отображаемые части скина
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DisplayedSkinParts {
  pub cape: bool,
  pub jacket: bool,
  pub left_sleeve: bool,
  pub right_sleeve: bool,
  pub left_pants_leg: bool,
  pub right_pants_leg: bool,
  pub hat: bool
}

impl Default for DisplayedSkinParts {
  fn default() -> Self {
    Self {
      cape: true,
      jacket: true,
      left_sleeve: true,
      right_sleeve: true,
      left_pants_leg: true,
      right_pants_leg: true,
      hat: true
    }
  }
}

impl DisplayedSkinParts {
  /// Метод получения битовой маски из `DisplayedSkinParts`
  pub fn to_mask(&self) -> u8 {
    let mut mask = 0u8;
    
    if self.cape { mask |= 0x01; }
    if self.jacket { mask |= 0x02; }
    if self.left_sleeve { mask |= 0x04; }
    if self.right_sleeve { mask |= 0x08; }
    if self.left_pants_leg { mask |= 0x10; }
    if self.right_pants_leg { mask |= 0x20; }
    if self.hat { mask |= 0x40; }
    
    mask
  }

  /// Метод получения `DisplayedSkinParts` из битовой маски
  pub fn from_mask(mask: u8) -> Self {
    Self {
      cape: (mask & 0x01) != 0,
      jacket: (mask & 0x02) != 0,
      left_sleeve: (mask & 0x04) != 0,
      right_sleeve: (mask & 0x08) != 0,
      left_pants_leg: (mask & 0x10) != 0,
      right_pants_leg: (mask & 0x20) != 0,
      hat: (mask & 0x40) != 0,
    }
  }
}

/// Основная (ведущая) рука
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ClientHand {
  Left,
  Right
}

impl ClientHand {
  /// Метод получения идентификатора ведущей руки
  pub fn id(&self) -> i32 {
    match self {
      ClientHand::Left => 0,
      ClientHand::Right => 1,
    }
  }
}

/// Статус видимости партиклов
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ParticleStatus {
  All,
  Decreased,
  Minimal
}

impl ParticleStatus {
  /// Метод получения идентификатора состояния видимости партиклов
  pub fn id(&self) -> i32 {
    match self {
      ParticleStatus::All => 0,
      ParticleStatus::Decreased => 1,
      ParticleStatus::Minimal => 2,
    }
  }
}