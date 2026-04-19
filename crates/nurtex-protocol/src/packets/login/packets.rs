use std::io::{self, Cursor, Write};

use nurtex_codec::{Buffer, VarInt};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideLoginDisconnect {
  pub reason: String,
}

impl ClientsideLoginDisconnect {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      reason: String::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.reason.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideEncryptionRequest {
  pub server_id: String,
  pub public_key: Vec<u8>,
  pub verify_token: Vec<u8>,
  pub should_authenticate: bool,
}

impl ClientsideEncryptionRequest {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let server_id = String::read_buf(buffer)?;
    let public_key_len = i32::read_varint(buffer)? as usize;
    let mut public_key = vec![0u8; public_key_len];

    for byte in &mut public_key {
      *byte = u8::read_buf(buffer)?;
    }

    let verify_token_len = i32::read_varint(buffer)? as usize;
    let mut verify_token = vec![0u8; verify_token_len];

    for byte in &mut verify_token {
      *byte = u8::read_buf(buffer)?;
    }

    let should_authenticate = bool::read_buf(buffer)?;

    Some(Self {
      server_id,
      public_key,
      verify_token,
      should_authenticate,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.server_id.write_buf(buffer)?;
    (self.public_key.len() as i32).write_varint(buffer)?;

    for byte in &self.public_key {
      byte.write_buf(buffer)?;
    }

    (self.verify_token.len() as i32).write_varint(buffer)?;

    for byte in &self.verify_token {
      byte.write_buf(buffer)?;
    }

    self.should_authenticate.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideLoginSuccess {
  pub uuid: Uuid,
  pub username: String,
  pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
  pub name: String,
  pub value: String,
  pub signature: Option<String>,
}

impl ClientsideLoginSuccess {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let uuid = Uuid::read_buf(buffer)?;
    let username = String::read_buf(buffer)?;
    let properties_len = i32::read_varint(buffer)? as usize;
    let mut properties = Vec::with_capacity(properties_len);

    for _ in 0..properties_len {
      let name = String::read_buf(buffer)?;
      let value = String::read_buf(buffer)?;
      let has_signature = bool::read_buf(buffer)?;

      let signature = if has_signature { Some(String::read_buf(buffer)?) } else { None };

      properties.push(Property { name, value, signature });
    }

    Some(Self { uuid, username, properties })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.uuid.write_buf(buffer)?;
    self.username.write_buf(buffer)?;
    (self.properties.len() as i32).write_varint(buffer)?;

    for prop in &self.properties {
      prop.name.write_buf(buffer)?;
      prop.value.write_buf(buffer)?;
      prop.signature.is_some().write_buf(buffer)?;

      if let Some(sig) = &prop.signature {
        sig.write_buf(buffer)?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideSetCompression {
  pub compression_threshold: i32,
}

impl ClientsideSetCompression {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      compression_threshold: i32::read_varint(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.compression_threshold.write_varint(buffer)
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsidePluginRequest {
  pub message_id: i32,
  pub channel: String,
  pub data: Vec<u8>,
}

impl ClientsidePluginRequest {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let message_id = i32::read_varint(buffer)?;
    let channel = String::read_buf(buffer)?;

    let mut data = Vec::new();

    while let Some(byte) = u8::read_buf(buffer) {
      data.push(byte);
    }

    Some(Self { message_id, channel, data })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.message_id.write_varint(buffer)?;
    self.channel.write_buf(buffer)?;

    for byte in &self.data {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientsideCookieRequest {
  pub key: String,
}

impl ClientsideCookieRequest {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self { key: String::read_buf(buffer)? })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.key.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideLoginStart {
  pub username: String,
  pub uuid: Uuid,
}

impl ServersideLoginStart {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      username: String::read_buf(buffer)?,
      uuid: Uuid::read_buf(buffer)?,
    })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.username.write_buf(buffer)?;
    self.uuid.write_buf(buffer)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideEncryptionResponse {
  pub shared_secret: Vec<u8>,
  pub verify_token: Vec<u8>,
}

impl ServersideEncryptionResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let shared_secret_len = i32::read_varint(buffer)? as usize;
    let mut shared_secret = vec![0u8; shared_secret_len];

    for byte in &mut shared_secret {
      *byte = u8::read_buf(buffer)?;
    }

    let verify_token_len = i32::read_varint(buffer)? as usize;
    let mut verify_token = vec![0u8; verify_token_len];

    for byte in &mut verify_token {
      *byte = u8::read_buf(buffer)?;
    }

    Some(Self { shared_secret, verify_token })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    (self.shared_secret.len() as i32).write_varint(buffer)?;

    for byte in &self.shared_secret {
      byte.write_buf(buffer)?;
    }

    (self.verify_token.len() as i32).write_varint(buffer)?;

    for byte in &self.verify_token {
      byte.write_buf(buffer)?;
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersidePluginResponse {
  pub message_id: i32,
  pub data: Option<Vec<u8>>,
}

impl ServersidePluginResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let message_id = i32::read_varint(buffer)?;
    let has_data = bool::read_buf(buffer)?;

    let data = if has_data {
      let mut bytes = Vec::new();

      while let Some(byte) = u8::read_buf(buffer) {
        bytes.push(byte);
      }

      Some(bytes)
    } else {
      None
    };

    Some(Self { message_id, data })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.message_id.write_varint(buffer)?;
    self.data.is_some().write_buf(buffer)?;

    if let Some(data) = &self.data {
      for byte in data {
        byte.write_buf(buffer)?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideLoginAcknowledged;

impl ServersideLoginAcknowledged {
  pub fn read(_buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self)
  }

  pub fn write(&self, _buffer: &mut impl Write) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServersideCookieResponse {
  pub key: String,
  pub payload: Option<Vec<u8>>,
}

impl ServersideCookieResponse {
  pub fn read(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let key = String::read_buf(buffer)?;
    let has_payload = bool::read_buf(buffer)?;

    let payload = if has_payload {
      let len = i32::read_varint(buffer)? as usize;
      let mut bytes = vec![0u8; len];

      for byte in &mut bytes {
        *byte = u8::read_buf(buffer)?;
      }

      Some(bytes)
    } else {
      None
    };

    Some(Self { key, payload })
  }

  pub fn write(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.key.write_buf(buffer)?;
    self.payload.is_some().write_buf(buffer)?;

    if let Some(payload) = &self.payload {
      (payload.len() as i32).write_varint(buffer)?;

      for byte in payload {
        byte.write_buf(buffer)?;
      }
    }

    Ok(())
  }
}
