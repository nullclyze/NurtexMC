use std::sync::Arc;
use tokio::sync::broadcast;

use crate::bot::Bot;
use crate::bot::components::position::Position;
use crate::bot::components::rotation::Rotation;
use crate::bot::options::BotStatus;
use crate::bot::world::StorageLock;

/// Трейт пакета данных (package) бота.
///
/// Пример создания своего пакета:
/// ```rust, ignore
/// // Создаём структуру данных пакета
/// #[derive(Clone, Debug)]
/// struct CustomPackage {
///     position: Position,
///     health: f32,
/// }
///
/// // Реализуем метод описания пакета
/// impl BotPackage for CustomPackage {
///     fn describe(bot: &Bot) -> Self {
///         Self {
///             position: bot.components.position.clone(),
///             health: bot.components.state.health,
///         }
///     }
/// }
/// ```
/// 
/// Пример использования своего пакета:
/// ```rust, ignore
/// // Создаём бота и задаём ему конкретный тип
/// let account = BotAccount::new("NurtexBot");
/// let bot: Bot<CustomPackage> = Bot::create(account);
/// 
/// // Прочая логика...
/// ```
pub trait BotPackage: Clone + Send + 'static {
  /// Метод создания пакета из данных бота
  fn describe<P: BotPackage>(bot: &Bot<P>) -> Self;
}

/// Передатчик пакетов с данными бота
#[derive(Clone)]
pub struct BotTransmitter<B: BotPackage> {
  sender: Arc<broadcast::Sender<B>>,
}

impl<B: BotPackage> BotTransmitter<B> {
  /// Метод создания нового передатчика
  pub fn new(capacity: usize) -> Self {
    let (sender, _) = broadcast::channel(capacity);
    Self { sender: Arc::new(sender) }
  }

  /// Метод подписки к передатчику пакетов
  pub fn subscribe(&self) -> broadcast::Receiver<B> {
    self.sender.subscribe()
  }

  /// Метод отправки пакета всем получателям
  pub fn emit(&self, package: B) {
    let _ = self.sender.send(package);
  }

  /// Метод получения количества активных получателей
  pub fn receiver_count(&self) -> usize {
    self.sender.receiver_count()
  }
}

/// Пустой пакет данных бота
#[derive(Clone)]
pub struct NullPackage;

impl BotPackage for NullPackage {
  fn describe<B: BotPackage>(_bot: &Bot<B>) -> Self {
    Self
  }
}

/// Минимальный пакет данных бота
#[derive(Clone, Debug)]
pub struct MinimalPackage {
  pub position: Position,
  pub rotation: Rotation,
}

impl BotPackage for MinimalPackage {
  fn describe<B: BotPackage>(bot: &Bot<B>) -> Self {
    Self {
      position: bot.components.position.clone(),
      rotation: bot.components.rotation.clone(),
    }
  }
}

/// Стандартный пакет данных бота
#[derive(Clone, Debug)]
pub struct StandardPackage {
  pub status: BotStatus,
  pub position: Position,
  pub rotation: Rotation,
  pub health: f32,
  pub satiety: u32,
}

impl BotPackage for StandardPackage {
  fn describe<B: BotPackage>(bot: &Bot<B>) -> Self {
    Self {
      status: bot.status.clone(),
      position: bot.components.position.clone(),
      rotation: bot.components.rotation.clone(),
      health: bot.components.state.health,
      satiety: bot.components.state.satiety,
    }
  }
}

/// Полный пакет данных бота
#[derive(Clone, Debug)]
pub struct FullPackage {
  pub status: BotStatus,
  pub position: Position,
  pub rotation: Rotation,
  pub health: f32,
  pub satiety: u32,
  pub local_storage: StorageLock,
  pub shared_storage: Option<StorageLock>,
}

impl BotPackage for FullPackage {
  fn describe<B: BotPackage>(bot: &Bot<B>) -> Self {
    Self {
      status: bot.status.clone(),
      position: bot.components.position.clone(),
      rotation: bot.components.rotation.clone(),
      health: bot.components.state.health,
      satiety: bot.components.state.satiety,
      local_storage: bot.local_storage.clone(),
      shared_storage: bot.shared_storage.clone(),
    }
  }
}
