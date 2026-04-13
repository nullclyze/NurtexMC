use azalea_inventory::ItemStack;
use hashbrown::HashMap;

/// Структура данных инвентаря
#[derive(Debug, Clone)]
pub struct Inventory {
  pub containers: HashMap<i32, InventoryContainer>,
}

/// Структура данных контейнера инвентаря
#[derive(Debug, Clone)]
pub struct InventoryContainer {
  pub carried_item: ItemStack,
  pub items: Vec<InventoryItem>,
}

/// Структура данных предмета инвентаря
#[derive(Debug, Clone)]
pub struct InventoryItem {
  pub name: String,
  pub count: i32,
  pub slot: u32,
}

impl Default for Inventory {
  fn default() -> Self {
    Self { containers: HashMap::new() }
  }
}

impl Inventory {
  /// Метод получения ссылки контейнера
  pub fn get_container(&self, id: i32) -> Option<&InventoryContainer> {
    self.containers.get(&id)
  }

  /// Метод получения мутабельной ссылки контейнера
  pub fn get_mut_container(&mut self, id: i32) -> Option<&mut InventoryContainer> {
    self.containers.get_mut(&id)
  }

  /// Метод добавления нового конейтенра
  pub fn add_container(&mut self, id: i32) {
    self.containers.insert(
      id,
      InventoryContainer {
        carried_item: ItemStack::Empty,
        items: Vec::new(),
      },
    );
  }
}
