use nurtex_codec::Buffer;

/// Текстовый компонент
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TextComponent(String);

impl Buffer for TextComponent {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self(String::read_buf(buffer)?))
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.0.write_buf(buffer)
  }
}

impl TextComponent {
  /// Метод получения текста
  pub fn text(&self) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.0) {
      self.extract_text_from_json(&json).unwrap_or(self.clear_text(&self.0))
    } else {
      self.clear_text(&self.0)
    }
  }

  /// Метод извлечения текста из JSON формата
  fn extract_text_from_json(&self, value: &serde_json::Value) -> Option<String> {
    match value {
      serde_json::Value::String(s) => Some(self.clear_text(s)),
      serde_json::Value::Object(obj) => {
        let mut result = String::new();

        if let Some(text) = obj.get("text") {
          if let serde_json::Value::String(s) = text {
            result.push_str(&self.clear_text(s));
          }
        }

        if let Some(extra) = obj.get("extra") {
          if let serde_json::Value::Array(arr) = extra {
            for item in arr {
              result.push_str(&self.extract_text_from_json(item).unwrap_or(String::new()));
            }
          }
        }

        for (key, val) in obj {
          if key != "text" && key != "extra" && key != "color" && key != "bold" && key != "italic" {
            if let serde_json::Value::String(s) = val {
              if !s.is_empty() && key != "clickEvent" && key != "hoverEvent" {
                result.push_str(&self.clear_text(s));
              }
            }
          }
        }

        Some(result)
      }
      serde_json::Value::Array(arr) => {
        let mut result = String::new();

        for item in arr {
          result.push_str(&self.extract_text_from_json(item).unwrap_or(String::new()));
        }

        Some(result)
      }
      _ => None,
    }
  }

  /// Метод очистки текста
  fn clear_text(&self, text: &str) -> String {
    text.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\t').collect::<String>()
  }
}
