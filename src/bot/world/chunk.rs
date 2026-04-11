use azalea_core::position::ChunkPos;

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
  // Метод создания нового чанка
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
  // Метод создания пустой секции чанка
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
