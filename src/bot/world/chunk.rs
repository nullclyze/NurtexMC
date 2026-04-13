use azalea_block::BlockState;
use azalea_buf::AzaleaRead;
use azalea_core::position::{ChunkPos, ChunkSectionBlockPos};
use std::io::Cursor;

/// Чанк мира
#[derive(Debug, Clone)]
pub struct Chunk {
  /// Позиция чанка
  pub pos: ChunkPos,

  /// Секции чанка
  pub sections: Vec<azalea_world::Section>,
}

impl Chunk {
  // Метод создания нового чанка
  pub fn new(pos: ChunkPos, raw_data: Vec<u8>) -> Self {
    let mut chunk = Self { pos, sections: vec![] };

    chunk.parse_chunk_data(&raw_data);

    chunk
  }

  /// Метод парсинга данных чанка
  fn parse_chunk_data(&mut self, raw_data: &[u8]) {
    let mut cursor = Cursor::new(raw_data);

    for _ in 0..24 {
      match azalea_world::Section::azalea_read(&mut cursor) {
        Ok(section) => {
          self.sections.push(section);
        }
        Err(_e) => {
          break;
        }
      }
    }
  }

  /// Метод получения блока из чанка
  pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<u32> {
    if x >= 16 || y >= 384 || z >= 16 {
      return None;
    }

    let section_index = y / 16;
    let section_y = y % 16;

    let section = self.sections.get(section_index)?;
    let pos = ChunkSectionBlockPos::new(x as u8, section_y as u8, z as u8);
    let block_state = section.get_block_state(pos);

    Some(block_state.id() as u32)
  }

  /// Метод установки блока в чанке
  pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_state_id: u32) {
    if x >= 16 || y >= 384 || z >= 16 {
      return;
    }

    let section_index = y / 16;
    let section_y = y % 16;

    while self.sections.len() <= section_index {
      self.sections.push(azalea_world::Section::default());
    }

    if let Some(section) = self.sections.get_mut(section_index) {
      let pos = ChunkSectionBlockPos::new(x as u8, section_y as u8, z as u8);

      if let Ok(block_state) = BlockState::try_from(block_state_id as u16) {
        section.get_and_set_block_state(pos, block_state);
      }
    }
  }
}
