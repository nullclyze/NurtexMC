use std::sync::Arc;

use azalea_core::position::{BlockPos, ChunkPos, Vec3};
use hashbrown::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::bot::components::{position::Position, rotation::Rotation, velocity::Velocity};

/// Хранилище данных о мире
#[derive(Debug, Clone)]
pub struct Storage {
  /// Список всех сущностей, в ключе (i32) указывается ID сущности
  pub entities: HashMap<i32, Entity>,

  /// Хранилище чанков, в ключе (ChunkPos) указывается позиция чанка
  pub chunks: HashMap<ChunkPos, Chunk>,
}

impl Storage {
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),
      chunks: HashMap::new(),
    }
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
  pub fn set_block(&mut self, pos: &BlockPos, block_state: u32) {
    let chunk_pos = ChunkPos::from(pos);

    let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| Chunk::new(chunk_pos, Vec::new()));

    let x = pos.x.rem_euclid(16) as usize;
    let y = (pos.y + 64) as usize;
    let z = pos.z.rem_euclid(16) as usize;

    chunk.set_block(x, y, z, block_state);
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

pub type StorageLock = Arc<RwLock<Storage>>;

/// Сущность мира
#[derive(Debug, Clone)]
pub struct Entity {
  /// Тип сущности
  pub entity_type: String,

  /// UUID сущнности
  pub uuid: Uuid,

  /// Позиция сущности (x, y, z)
  pub position: Position,

  /// Скорость сущности (x, y, z)
  pub velocity: Velocity,

  /// Ротация сущности (y, x)
  pub rotation: Rotation,

  /// Физическое состояние `on_ground` сущности
  pub on_ground: bool,

  /// Информация игрока, если сущность **НЕ является** игроком - None
  pub player_info: Option<PlayerInfo>,
}

/// Информация об игроке
#[derive(Debug, Clone)]
pub struct PlayerInfo {
  /// Юзернейм игрока
  pub username: String,

  /// Режим игры игрока, например, "creative"
  pub game_mode: String,

  /// Пинг игрока
  pub ping: i32,
}

/// Чанк мира
#[derive(Debug, Clone)]
pub struct Chunk {
  /// Позиция чанка
  pub pos: ChunkPos,

  /// Секции чанка
  pub sections: Vec<ChunkSection>,

  /// Raw-данные чанка
  pub raw_data: Vec<u8>,
}

impl Chunk {
  pub fn new(pos: ChunkPos, raw_data: Vec<u8>) -> Self {
    Self {
      pos,
      sections: vec![ChunkSection::empty(); 24],
      raw_data,
    }
  }

  /// Метод получения блока из чанка
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<u32> {
    if x >= 16 || y >= 384 || z >= 16 {
      return None;
    }

    let section_index = y / 16;
    let section_y = y % 16;

    self.sections.get(section_index)?.get_block(x, section_y, z)
  }

  /// Метод установки блока в чанке
  pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_state: u32) {
    if x >= 16 || y >= 384 || z >= 16 {
      return;
    }

    let section_index = y / 16;
    let section_y = y % 16;

    if let Some(section) = self.sections.get_mut(section_index) {
      section.set_block(x, section_y, z, block_state);
    }
  }
}

/// Секция чанка
#[derive(Debug, Clone)]
pub struct ChunkSection {
  /// Палитра блоков
  pub palette: Vec<u32>,

  /// Индексы блоков в палитре (4096 штук)
  pub blocks: Vec<u16>,

  /// Количество непустых блоков
  pub non_air_blocks: u16,
}

impl ChunkSection {
  pub fn empty() -> Self {
    Self {
      palette: vec![0],
      blocks: vec![0; 4096],
      non_air_blocks: 0,
    }
  }

  /// Метод получения блока из секции
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<u32> {
    if x >= 16 || y >= 16 || z >= 16 {
      return None;
    }

    let index = (y * 16 * 16) + (z * 16) + x;
    let palette_index = *self.blocks.get(index)? as usize;
    self.palette.get(palette_index).copied()
  }

  /// Метод установки блока в секции
  pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_state: u32) {
    if x >= 16 || y >= 16 || z >= 16 {
      return;
    }

    let index = (y * 16 * 16) + (z * 16) + x;

    let palette_index = if let Some(idx) = self.palette.iter().position(|&b| b == block_state) {
      idx
    } else {
      self.palette.push(block_state);
      self.palette.len() - 1
    };

    let old_state = self.blocks[index];
    if old_state == 0 && block_state != 0 {
      self.non_air_blocks += 1;
    } else if old_state != 0 && block_state == 0 {
      self.non_air_blocks = self.non_air_blocks.saturating_sub(1);
    }

    self.blocks[index] = palette_index as u16;
  }
}
