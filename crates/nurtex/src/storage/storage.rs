use hashbrown::HashMap;

use crate::world::Entity;

/// Хранилище данных
#[derive(Debug, Clone)]
pub struct Storage {
  /// Список всех сущностей
  pub entities: HashMap<i32, Entity>,
}

impl Storage {
  /// Метод создания пустого хранилища
  pub fn null() -> Self {
    Self { entities: HashMap::new() }
  }

  /// Метод добавления сущности в хранилище
  pub fn add_entity(&mut self, id: i32, entity: Entity) {
    self.entities.insert(id, entity);
  }

  /// Метод получения ссылки на сущность
  pub fn get_entity(&self, id: &i32) -> Option<&Entity> {
    self.entities.get(id)
  }

  /// Метод получения мутабельной ссылки на сущность
  pub fn get_entity_mut(&mut self, id: &i32) -> Option<&mut Entity> {
    self.entities.get_mut(id)
  }

  /// Метод удаления сущности из хранилища
  pub fn remove_entity(&mut self, id: &i32) {
    self.entities.remove(id);
  }

  /// Метод очитски хранилища
  pub fn clear(&mut self) {
    self.entities.clear();
  }
}
