use std::fmt::Debug;
use std::io::{self, Cursor};
use std::sync::Arc;

use nurtex_encrypt::{AesDecryptor, AesEncryptor};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{Mutex, RwLock};

use crate::connection::address::NurtexAddr;
use crate::connection::reader::{deserialize_packet, read_raw_packet, try_read_raw_packet};
use crate::connection::writer::{serialize_packet, write_raw_packet};
use crate::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideConfigurationPacket},
  handshake::{ClientsideHandshakePacket, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginPacket},
  play::{ClientsidePlayPacket, ServersidePlayPacket},
  status::{ClientsideStatusPacket, ServersideStatusPacket},
};

/// Состояние подключения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
  Handshake,
  Status,
  Login,
  Configuration,
  Play,
}

/// Универсальное перечисление `Clientside` пакетов
#[derive(Debug, Clone)]
pub enum ClientsidePacket {
  Handshake(ClientsideHandshakePacket),
  Status(ClientsideStatusPacket),
  Login(ClientsideLoginPacket),
  Configuration(ClientsideConfigurationPacket),
  Play(ClientsidePlayPacket),
}

/// Универсальное перечисление `Serverside` пакетов
#[derive(Debug, Clone)]
pub enum ServersidePacket {
  Handshake(ServersideHandshakePacket),
  Status(ServersideStatusPacket),
  Login(ServersideLoginPacket),
  Configuration(ServersideConfigurationPacket),
  Play(ServersidePlayPacket),
}

impl ServersidePacket {
  /// Вспомогательный метод создания `handshake` пакета
  pub fn handshake(packet: ServersideHandshakePacket) -> Self {
    ServersidePacket::Handshake(packet)
  }

  /// Вспомогательный метод создания `status` пакета
  pub fn status(packet: ServersideStatusPacket) -> Self {
    ServersidePacket::Status(packet)
  }

  //// Вспомогательный метод создания `login` пакета
  pub fn login(packet: ServersideLoginPacket) -> Self {
    ServersidePacket::Login(packet)
  }

  /// Вспомогательный метод создания `configuration` пакета
  pub fn configuration(packet: ServersideConfigurationPacket) -> Self {
    ServersidePacket::Configuration(packet)
  }

  /// Вспомогательный метод создания `play` пакета
  pub fn play(packet: ServersidePlayPacket) -> Self {
    ServersidePacket::Play(packet)
  }
}

/// Структура для чтения пакетов
pub struct ConnectionReader {
  read_stream: OwnedReadHalf,
  buffer: Cursor<Vec<u8>>,
  compression_threshold: Arc<RwLock<Option<u32>>>,
  decryptor: Arc<Mutex<Option<AesDecryptor>>>,
  state: Arc<RwLock<ConnectionState>>,
}

/// Структура для записи пакетов
pub struct ConnectionWriter {
  write_stream: OwnedWriteHalf,
  compression_threshold: Arc<RwLock<Option<u32>>>,
  encryptor: Arc<Mutex<Option<AesEncryptor>>>,
}

/// Основная структура подключения
pub struct NurtexConnection {
  reader: Arc<Mutex<ConnectionReader>>,
  writer: Arc<Mutex<ConnectionWriter>>,
  state: Arc<RwLock<ConnectionState>>,
  compression_threshold: Arc<RwLock<Option<u32>>>,
}

impl ConnectionReader {
  /// Метод чтения пакета
  pub async fn read_packet(&mut self) -> Option<ClientsidePacket> {
    let compression_threshold = *self.compression_threshold.read().await;
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;

    let mut cursor = Cursor::new(raw_packet.as_ref());
    let state = *self.state.read().await;

    match state {
      ConnectionState::Handshake => deserialize_packet::<ClientsideHandshakePacket>(&mut cursor).map(ClientsidePacket::Handshake),
      ConnectionState::Status => deserialize_packet::<ClientsideStatusPacket>(&mut cursor).map(ClientsidePacket::Status),
      ConnectionState::Login => deserialize_packet::<ClientsideLoginPacket>(&mut cursor).map(ClientsidePacket::Login),
      ConnectionState::Configuration => deserialize_packet::<ClientsideConfigurationPacket>(&mut cursor).map(ClientsidePacket::Configuration),
      ConnectionState::Play => deserialize_packet::<ClientsidePlayPacket>(&mut cursor).map(ClientsidePacket::Play),
    }
  }

  /// Метод чтения пакета (неблокирующий)
  pub fn try_read_packet(&mut self) -> Result<Option<ClientsidePacket>, std::io::Error> {
    let compression_threshold = match self.compression_threshold.try_read() {
      Ok(threshold) => *threshold,
      Err(_) => return Ok(None),
    };

    let mut decryptor_guard = match self.decryptor.try_lock() {
      Ok(guard) => guard,
      Err(_) => return Ok(None),
    };

    let Some(raw_packet) = try_read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard)? else {
      return Ok(None);
    };

    let mut cursor = Cursor::new(raw_packet.as_ref());
    let state = match self.state.try_read() {
      Ok(state) => *state,
      Err(_) => return Ok(None),
    };

    let packet = match state {
      ConnectionState::Handshake => deserialize_packet::<ClientsideHandshakePacket>(&mut cursor).map(ClientsidePacket::Handshake),
      ConnectionState::Status => deserialize_packet::<ClientsideStatusPacket>(&mut cursor).map(ClientsidePacket::Status),
      ConnectionState::Login => deserialize_packet::<ClientsideLoginPacket>(&mut cursor).map(ClientsidePacket::Login),
      ConnectionState::Configuration => deserialize_packet::<ClientsideConfigurationPacket>(&mut cursor).map(ClientsidePacket::Configuration),
      ConnectionState::Play => deserialize_packet::<ClientsidePlayPacket>(&mut cursor).map(ClientsidePacket::Play),
    };

    Ok(packet)
  }

  /// Вспомогательный метод чтения `status` пакета
  pub async fn read_status_packet(&mut self) -> Option<ClientsideStatusPacket> {
    let compression_threshold = *self.compression_threshold.read().await;
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideStatusPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `login` пакета
  pub async fn read_login_packet(&mut self) -> Option<ClientsideLoginPacket> {
    let compression_threshold = *self.compression_threshold.read().await;
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideLoginPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `configuration` пакета
  pub async fn read_configuration_packet(&mut self) -> Option<ClientsideConfigurationPacket> {
    let compression_threshold = *self.compression_threshold.read().await;
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideConfigurationPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `play` пакета
  pub async fn read_play_packet(&mut self) -> Option<ClientsidePlayPacket> {
    let compression_threshold = *self.compression_threshold.read().await;
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsidePlayPacket>(&mut cursor)
  }
}

impl ConnectionWriter {
  /// Метод записи пакета
  pub async fn write_packet(&mut self, packet: ServersidePacket) -> io::Result<()> {
    let serialized = match packet {
      ServersidePacket::Handshake(p) => serialize_packet(&p),
      ServersidePacket::Status(p) => serialize_packet(&p),
      ServersidePacket::Login(p) => serialize_packet(&p),
      ServersidePacket::Configuration(p) => serialize_packet(&p),
      ServersidePacket::Play(p) => serialize_packet(&p),
    }
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to serialize packet"))?;

    let compression_threshold = *self.compression_threshold.read().await;
    let mut encryptor_guard = self.encryptor.lock().await;

    write_raw_packet(&serialized, &mut self.write_stream, compression_threshold, &mut *encryptor_guard).await
  }

  /// Вспомогательный метод записи `handshake` пакета
  pub async fn write_handshake_packet(&mut self, packet: ServersideHandshakePacket) -> io::Result<()> {
    self.write_packet(ServersidePacket::Handshake(packet)).await
  }

  /// Вспомогательный метод записи `status` пакета
  pub async fn write_status_packet(&mut self, packet: ServersideStatusPacket) -> io::Result<()> {
    self.write_packet(ServersidePacket::Status(packet)).await
  }

  /// Вспомогательный метод записи `login` пакета
  pub async fn write_login_packet(&mut self, packet: ServersideLoginPacket) -> io::Result<()> {
    self.write_packet(ServersidePacket::Login(packet)).await
  }

  /// Вспомогательный метод записи `configuration` пакета
  pub async fn write_configuration_packet(&mut self, packet: ServersideConfigurationPacket) -> io::Result<()> {
    self.write_packet(ServersidePacket::Configuration(packet)).await
  }

  /// Вспомогательный метод записи `play` пакета
  pub async fn write_play_packet(&mut self, packet: ServersidePlayPacket) -> io::Result<()> {
    self.write_packet(ServersidePacket::Play(packet)).await
  }

  /// Метод выключения потока записи
  pub async fn shutdown(&mut self) -> io::Result<()> {
    self.write_stream.shutdown().await
  }
}

impl NurtexConnection {
  /// Метод создания нового подключения
  pub async fn new(address: &NurtexAddr) -> io::Result<Self> {
    let stream = TcpStream::connect(address.unpack()).await?;
    stream.set_nodelay(true)?;
    Self::new_from_stream(stream).await
  }

  /// Метод создания нового подключения из TcpStream
  pub async fn new_from_stream(stream: TcpStream) -> io::Result<Self> {
    let (read_stream, write_stream) = stream.into_split();

    let state = Arc::new(RwLock::new(ConnectionState::Handshake));
    let compression_threshold = Arc::new(RwLock::new(None));

    let reader = ConnectionReader {
      read_stream,
      buffer: Cursor::new(Vec::new()),
      compression_threshold: Arc::clone(&compression_threshold),
      decryptor: Arc::new(Mutex::new(None)),
      state: Arc::clone(&state),
    };

    let writer = ConnectionWriter {
      write_stream,
      compression_threshold: Arc::clone(&compression_threshold),
      encryptor: Arc::new(Mutex::new(None)),
    };

    Ok(NurtexConnection {
      reader: Arc::new(Mutex::new(reader)),
      writer: Arc::new(Mutex::new(writer)),
      state,
      compression_threshold,
    })
  }

  /// Метод получения `reader`
  pub fn get_reader(&self) -> Arc<Mutex<ConnectionReader>> {
    self.reader.clone()
  }

  /// Метод получения `writer`
  pub fn get_writer(&self) -> Arc<Mutex<ConnectionWriter>> {
    self.writer.clone()
  }

  /// Метод получения текущего состояния подключения
  pub async fn get_state(&self) -> ConnectionState {
    *self.state.read().await
  }

  /// Метод изменения состояния подключения
  pub async fn set_state(&self, state: ConnectionState) {
    *self.state.write().await = state;
  }

  /// Вспомогательный метод чтения пакета
  pub async fn read_packet(&self) -> Option<ClientsidePacket> {
    let mut reader = self.reader.lock().await;
    reader.read_packet().await
  }

  /// Вспомогательный метод чтения пакета (неблокирующий)
  pub fn try_read_packet(&self) -> Result<Option<ClientsidePacket>, std::io::Error> {
    let mut reader = match self.reader.try_lock() {
      Ok(reader) => reader,
      Err(_) => return Ok(None),
    };

    reader.try_read_packet()
  }

  /// Вспомогательный метод чтения `status` пакета
  pub async fn read_status_packet(&self) -> Option<ClientsideStatusPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_status_packet().await
  }

  /// Вспомогательный метод чтения `login` пакета
  pub async fn read_login_packet(&self) -> Option<ClientsideLoginPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_login_packet().await
  }

  /// Вспомогательный метод чтения `configuration` пакета
  pub async fn read_configuration_packet(&self) -> Option<ClientsideConfigurationPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_configuration_packet().await
  }

  /// Вспомогательный метод чтения `play` пакета
  pub async fn read_play_packet(&self) -> Option<ClientsidePlayPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_play_packet().await
  }

  /// Вспомогательный метод записи пакета
  pub async fn write_packet(&self, packet: ServersidePacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(packet).await
  }

  /// Вспомогательный метод записи `handshake` пакета
  pub async fn write_handshake_packet(&self, packet: ServersideHandshakePacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_handshake_packet(packet).await
  }

  /// Вспомогательный метод записи `status` пакета
  pub async fn write_status_packet(&self, packet: ServersideStatusPacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_status_packet(packet).await
  }

  /// Вспомогательный метод записи `login` пакета
  pub async fn write_login_packet(&self, packet: ServersideLoginPacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_login_packet(packet).await
  }

  /// Вспомогательный метод записи `configuration` пакета
  pub async fn write_configuration_packet(&self, packet: ServersideConfigurationPacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_configuration_packet(packet).await
  }

  /// Вспомогательный метод записи `play` пакета
  pub async fn write_play_packet(&self, packet: ServersidePlayPacket) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_play_packet(packet).await
  }

  /// Метод выключения соединения
  pub async fn shutdown(&self) -> io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.shutdown().await
  }

  /// Метод установки порога сжатия
  pub async fn set_compression_threshold(&self, threshold: i32) {
    let new_threshold = if threshold >= 0 { Some(threshold as u32) } else { None };

    *self.compression_threshold.write().await = new_threshold;
  }

  /// Устанавливает шифрование на соединении используя секретный ключ.
  /// Этот метод должен быть вызван **после** отправки `EncryptionResponse` серверу
  pub async fn set_encryption_key(&self, secret_key: [u8; 16]) {
    let (encryptor, decryptor) = nurtex_encrypt::create_cipher(&secret_key);

    {
      let reader = self.reader.lock().await;
      *reader.decryptor.lock().await = Some(decryptor);
    }

    {
      let writer = self.writer.lock().await;
      *writer.encryptor.lock().await = Some(encryptor);
    }
  }
}
