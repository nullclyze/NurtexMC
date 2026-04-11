use std::sync::Arc;

use azalea_core::position::{BlockPos, ChunkPos, Vec3};
use hashbrown::HashMap;
use tokio::sync::RwLock;

use crate::bot::world::{chunk::Chunk, entity::Entity};

/// Хранилище данных о мире
#[derive(Debug, Clone)]
pub struct Storage {
  /// Список всех сущностей, в ключе (i32) указывается ID сущности
  pub entities: HashMap<i32, Entity>,

  /// Хранилище чанков, в ключе (ChunkPos) указывается позиция чанка
  pub chunks: HashMap<ChunkPos, Chunk>,
}

/// Вспомогательная обёртка для Storage
pub type StorageLock = Arc<RwLock<Storage>>;

impl Storage {
  // Метод создания нового пустого хранилища
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),
      chunks: HashMap::new(),
    }
  }

  // Метод полной очистки хранилища
  pub fn clear(&mut self) {
    self.entities.clear();
    self.chunks.clear();
  }

  /// Метод получения блока по позиции
  pub fn get_block(&self, pos: &BlockPos) -> Option<u32> {
    let chunk_pos = ChunkPos::from(pos);
    let chunk = self.chunks.get(&chunk_pos)?;

    let x = pos.x.rem_euclid(16) as usize;
    let y = (pos.y + 64) as usize;
    let z = pos.z.rem_euclid(16) as usize;

    chunk.get_block(x, y, z)
  }

  /// Метод установки блока на определённую позицию
  pub fn set_block(&mut self, pos: &BlockPos, state: u32) {
    let chunk_pos = ChunkPos::from(pos);

    let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(chunk_pos, Vec::new()));

    let x = pos.x.rem_euclid(16) as usize;
    let y = (pos.y + 64) as usize;
    let z = pos.z.rem_euclid(16) as usize;

    chunk.set_block(x, y, z, state);
  }

  /// Метод установки секции блоков
  pub fn set_block_section(&mut self, blocks: HashMap<BlockPos, u32>) {
    for (pos, state) in blocks {
      let chunk_pos = ChunkPos::from(pos);

      let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(chunk_pos, Vec::new()));

      let x = pos.x.rem_euclid(16) as usize;
      let y = (pos.y + 64) as usize;
      let z = pos.z.rem_euclid(16) as usize;

      chunk.set_block(x, y, z, state);
    }
  }

  /// Метод загрузки чанка
  pub fn load_chunk(&mut self, x: i32, z: i32, data: Vec<u8>) {
    let chunk_pos = ChunkPos::new(x, z);
    let chunk = Chunk::new(chunk_pos, data);
    self.chunks.insert(chunk_pos, chunk);
  }

  /// Метод удаления чанка
  pub fn remove_chunk(&mut self, chunk_pos: &ChunkPos) {
    self.chunks.remove(chunk_pos);
  }

  /// Метод проверки на существование чанка
  pub fn is_chunk_loaded(&self, chunk_pos: &ChunkPos) -> bool {
    self.chunks.contains_key(chunk_pos)
  }

  /// Метод проверки, находится ли бот на земле
  pub fn is_on_ground(&self, pos: &Vec3) -> bool {
    let y = (pos.y - 0.1).floor() as i32;

    let check_positions = [
      BlockPos::new(pos.x.floor() as i32, y, pos.z.floor() as i32),
      BlockPos::new((pos.x + 0.3).floor() as i32, y, pos.z.floor() as i32),
      BlockPos::new(pos.x.floor() as i32, y, (pos.z + 0.3).floor() as i32),
      BlockPos::new((pos.x + 0.3).floor() as i32, y, (pos.z + 0.3).floor() as i32),
    ];

    for ground_pos in &check_positions {
      match self.get_block(ground_pos) {
        Some(block_id) if block_id != 0 => return true,
        _ => {}
      }
    }

    false
  }
}
