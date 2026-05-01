use std::sync::Arc;
use std::time::Duration;

use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::ProxyAuth;
use crate::error::{ErrorName, ProxyError};
use crate::result::ProxyResult;

/// Структура SOCKS5 / SOCKS4 прокси.
///
/// ## Примеры
///
/// ```rust, ignore
/// use nurtex_proxy::{Proxy, ProxyAuth, ProxyType};
///
/// // Пример SOCKS5 прокси без авторизации
/// let proxy = Proxy::new("PROXY_IP:PROXY_PORT", ProxyType::Socks5);
///
/// // Пример SOCKS5 с авторизацией
/// let auth = ProxyAuth::new("USERNAME", "PASSWORD");
/// let proxy = Proxy::new_with_auth("PROXY_IP:PROXY_PORT", ProxyType::Socks5, auth);
///
/// // Пример SOCKS4 с авторизацией
/// let auth = ProxyAuth::new("USER_ID", ""); // В SOCKS4 не используется пароль
/// let proxy = Proxy::new_with_auth("PROXY_IP:PROXY_PORT", ProxyType::Socks4, auth);
/// ```
#[derive(Debug, Clone)]
pub struct Proxy {
  proxy_type: ProxyType,
  proxy_address: String,
  target: Arc<RwLock<TargetServer>>,
  timeout: u64,
  auth: Option<ProxyAuth>,
}

/// Тип прокси
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
  Socks5,
  Socks4,
}

/// Структура адреса целевого сервера
#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetServer {
  host: Option<String>,
  port: Option<u16>,
}

impl Default for TargetServer {
  fn default() -> Self {
    Self { host: None, port: None }
  }
}

/// Вспомогательная функция записи данных в поток
async fn write_all_to(stream: &mut TcpStream, buffer: Vec<u8>) -> ProxyResult<()> {
  match timeout(Duration::from_secs(10), stream.write_all(&buffer)).await {
    Ok(result) => match result {
      Ok(_) => ProxyResult::Ok(()),
      Err(e) => ProxyResult::Err(ProxyError::new(ErrorName::StreamError, e.to_string())),
    },
    Err(_) => ProxyResult::Err(ProxyError::new(ErrorName::StreamError, "failed to write buffer to stream")),
  }
}

/// Вспомогательная функция чтения данных из потока
async fn read_exact_from<'a>(stream: &mut TcpStream, buffer: &'a mut [u8]) -> ProxyResult<()> {
  match timeout(Duration::from_secs(10), stream.read_exact(buffer)).await {
    Ok(result) => match result {
      Ok(_) => ProxyResult::Ok(()),
      Err(e) => ProxyResult::Err(ProxyError::new(ErrorName::StreamError, e.to_string())),
    },
    Err(_) => ProxyResult::Err(ProxyError::new(ErrorName::StreamError, "failed to read buffer from stream")),
  }
}

impl From<String> for Proxy {
  fn from(value: String) -> Self {
    let split = value.split("://").collect::<Vec<&str>>();
    let (protocol, proxy) = (split.get(0).unwrap_or(&"socks5"), split.get(1).unwrap_or(&"127.0.0.1"));

    Self {
      proxy_address: (*proxy).to_string(),
      proxy_type: match *protocol {
        "socks5" => ProxyType::Socks5,
        "socks4" => ProxyType::Socks4,
        _ => ProxyType::Socks5,
      },
      target: Arc::new(RwLock::new(TargetServer::default())),
      timeout: 20000,
      auth: None,
    }
  }
}

impl From<&str> for Proxy {
  fn from(value: &str) -> Self {
    let split = value.split("://").collect::<Vec<&str>>();
    let (protocol, proxy) = (split.get(0).unwrap_or(&"socks5"), split.get(1).unwrap_or(&"127.0.0.1"));

    Self {
      proxy_address: (*proxy).to_string(),
      proxy_type: match *protocol {
        "socks5" => ProxyType::Socks5,
        "socks4" => ProxyType::Socks4,
        _ => ProxyType::Socks5,
      },
      target: Arc::new(RwLock::new(TargetServer::default())),
      timeout: 20000,
      auth: None,
    }
  }
}

impl Proxy {
  /// Метод создания нового прокси
  pub fn new(proxy_address: impl Into<String>, proxy_type: ProxyType) -> Self {
    Self {
      proxy_address: proxy_address.into(),
      proxy_type: proxy_type,
      target: Arc::new(RwLock::new(TargetServer::default())),
      timeout: 20000,
      auth: None,
    }
  }

  /// Метод создания нового прокси с авторизацией
  pub fn new_with_auth(proxy_address: impl Into<String>, proxy_type: ProxyType, auth: ProxyAuth) -> Self {
    Self {
      proxy_address: proxy_address.into(),
      proxy_type: proxy_type,
      target: Arc::new(RwLock::new(TargetServer::default())),
      timeout: 20000,
      auth: Some(auth),
    }
  }

  /// Метод установки адреса целевого сервера
  pub fn bind(&self, target_host: String, target_port: u16) {
    match self.target.try_write() {
      Ok(mut g) => {
        g.host = Some(target_host);
        g.port = Some(target_port);
      }
      Err(_) => {}
    }
  }

  /// Метод установки таймаута подключения к прокси
  pub fn set_timeout(mut self, timeout: u64) -> Self {
    self.timeout = timeout;
    self
  }

  /// Метод установки типа прокси
  pub fn set_proxy_type(mut self, proxy_type: ProxyType) -> Self {
    self.proxy_type = proxy_type;
    self
  }

  /// Метод попытки создания соединения с прокси
  pub async fn is_available(&self) -> bool {
    match timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.proxy_address)).await {
      Ok(result) => match result {
        Ok(_) => return true,
        Err(_) => return false,
      },
      Err(_) => return false,
    }
  }

  /// Метод получения IP прокси
  pub fn get_ip(&self) -> Option<String> {
    if let Some(ip) = self.proxy_address.split(":").collect::<Vec<&str>>().get(0) {
      Some(ip.to_string())
    } else {
      None
    }
  }

  /// Метод подключения к прокси
  pub async fn connect(&self) -> ProxyResult<TcpStream> {
    let (target_host, target_port) = {
      let guard = self.target.read().await;

      let Some(host) = guard.host.clone() else {
        return ProxyResult::Err(ProxyError::new(ErrorName::InvalidData, "target server host not specified"));
      };

      let Some(port) = guard.port else {
        return ProxyResult::Err(ProxyError::new(ErrorName::InvalidData, "target server port not specified"));
      };

      (host, port)
    };

    let mut stream = match timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.proxy_address)).await {
      Ok(result) => match result {
        Ok(s) => s,
        Err(_) => return ProxyResult::Err(ProxyError::new(ErrorName::NotConnected, "could not connect to specified server")),
      },
      Err(_) => return ProxyResult::Err(ProxyError::new(ErrorName::Timeout, "failed to connect to server within specified time")),
    };

    match self.proxy_type {
      ProxyType::Socks5 => self.connect_socks5(&mut stream, target_host, target_port).await?,
      ProxyType::Socks4 => self.connect_socks4(&mut stream, target_host, target_port).await?,
    }

    ProxyResult::Ok(stream)
  }

  /// Метод создания подключения с SOCKS5 прокси
  async fn connect_socks5(&self, stream: &mut TcpStream, target_host: String, target_port: u16) -> ProxyResult<()> {
    let greet = if self.auth.is_some() { vec![0x05, 0x02, 0x00, 0x02] } else { vec![0x05, 0x01, 0x00] };

    write_all_to(stream, greet).await?;

    let mut response = [0u8; 2];

    read_exact_from(stream, &mut response).await?;

    if response[0] != 0x05 {
      return ProxyResult::Err(ProxyError::new(ErrorName::InvalidVersion, "invalid response version"));
    }

    match response[1] {
      0x00 => {}
      0x02 => {
        if let Some(auth) = &self.auth {
          let username = auth.username();
          let password = auth.password();

          if username.len() > 255 || password.len() > 255 {
            return Err(ProxyError::new(ErrorName::InvalidData, "username or password is too long"));
          }

          let mut buffer = BytesMut::with_capacity(2 + username.len() + password.len());
          buffer.put_u8(0x01);
          buffer.put_u8(username.len() as u8);
          buffer.put_slice(username.as_bytes());
          buffer.put_u8(password.len() as u8);
          buffer.put_slice(password.as_bytes());

          write_all_to(stream, buffer.into()).await?;

          let mut resp = [0u8; 2];

          read_exact_from(stream, &mut resp).await?;

          if resp[0] != 0x01 {
            return Err(ProxyError::new(ErrorName::AuthFailed, "invalid authorization version"));
          }

          if resp[1] != 0x00 {
            return Err(ProxyError::new(ErrorName::AuthFailed, "authorization failed (possibly incorrect password or username)"));
          }
        } else {
          return ProxyResult::Err(ProxyError::new(ErrorName::AuthFailed, "proxy requires authorization (username, password)"));
        }
      }
      _ => return ProxyResult::Err(ProxyError::new(ErrorName::Unsupported, "unsupported authorization method")),
    }

    let mut request = BytesMut::with_capacity(512);
    request.put_u8(0x05);
    request.put_u8(0x01);
    request.put_u8(0x00);

    if let Ok(ipv4) = target_host.parse::<std::net::Ipv4Addr>() {
      request.put_u8(0x01);
      request.put_slice(&ipv4.octets());
    } else if let Ok(ipv6) = target_host.parse::<std::net::Ipv6Addr>() {
      request.put_u8(0x04);
      request.put_slice(&ipv6.octets());
    } else {
      request.put_u8(0x03);
      let host_bytes = target_host.as_bytes();

      if host_bytes.len() > 255 {
        return ProxyResult::Err(ProxyError::new(ErrorName::InvalidData, "target host is too long"));
      }

      request.put_u8(host_bytes.len() as u8);
      request.put_slice(host_bytes);
    }

    request.put_u16(target_port);

    write_all_to(stream, request.into()).await?;

    let mut header = [0u8; 4];

    read_exact_from(stream, &mut header).await?;

    if header[0] != 0x05 {
      return ProxyResult::Err(ProxyError::new(ErrorName::InvalidVersion, "invalid response version"));
    }

    let rep = header[1];

    if rep != 0x00 {
      return ProxyResult::Err(ProxyError::new(ErrorName::NotConnected, format!("proxy connection error (rep: 0x{:02x})", rep)));
    }

    let atyp = header[3];

    match atyp {
      0x01 => {
        let mut addr = [0u8; 4 + 2];
        read_exact_from(stream, &mut addr).await?;
      }
      0x04 => {
        let mut addr = [0u8; 16 + 2];
        read_exact_from(stream, &mut addr).await?;
      }
      0x03 => {
        let mut len = [0u8; 1];
        read_exact_from(stream, &mut len).await?;
        let mut rest = vec![0u8; len[0] as usize + 2];
        read_exact_from(stream, &mut rest).await?;
      }
      _ => return ProxyResult::Err(ProxyError::new(ErrorName::InvalidData, format!("unknown address type in reply: 0x{:02x}", atyp))),
    }

    ProxyResult::Ok(())
  }

  /// Метод создания подключения с SOCKS4 прокси
  async fn connect_socks4(&self, stream: &mut TcpStream, target_host: String, target_port: u16) -> ProxyResult<()> {
    let mut request = BytesMut::with_capacity(512);
    request.put_u8(0x04);
    request.put_u8(0x01);
    request.put_u16(target_port);

    if let Ok(ipv4) = target_host.parse::<std::net::Ipv4Addr>() {
      request.put_slice(&ipv4.octets());

      if let Some(auth) = &self.auth {
        request.put_slice(auth.username().as_bytes());
      } else {
        request.put_u8(0x00);
      }
    } else {
      request.put_slice(&[0x00, 0x00, 0x00, 0x01]);

      if let Some(auth) = &self.auth {
        request.put_slice(auth.username().as_bytes());
      } else {
        request.put_u8(0x00);
      }

      if target_host.len() > 255 {
        return Err(ProxyError::new(ErrorName::InvalidData, "target host is too long"));
      }

      request.put_slice(target_host.as_bytes());
      request.put_u8(0x00);
    }

    write_all_to(stream, request.into()).await?;

    let mut response = [0u8; 8];
    read_exact_from(stream, &mut response).await?;

    if response[0] != 0x00 {
      return Err(ProxyError::new(ErrorName::InvalidVersion, "invalid response version"));
    }

    match response[1] {
      0x5a => Ok(()),
      0x5b => Err(ProxyError::new(ErrorName::NotConnected, "request rejected or failed")),
      0x5c => Err(ProxyError::new(ErrorName::AuthFailed, "client not identd-authenticated")),
      0x5d => Err(ProxyError::new(ErrorName::AuthFailed, "client identd-user mismatch")),
      _ => Err(ProxyError::new(ErrorName::Unsupported, format!("unknown response code 0x{:02x}", response[1]))),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Error, ErrorKind};

  use tokio::io::{AsyncReadExt, AsyncWriteExt};

  use crate::result::ProxyResult;
  use crate::{Proxy, ProxyType};

  #[tokio::test]
  async fn test_socks5_proxy() -> std::io::Result<()> {
    let proxy = Proxy::new("212.58.132.5:1080", ProxyType::Socks5);
    proxy.bind("ipinfo.io".to_string(), 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }

  #[tokio::test]
  async fn test_socks4_proxy() -> std::io::Result<()> {
    let proxy = Proxy::new("68.71.242.118:4145", ProxyType::Socks4);
    proxy.bind("ipinfo.io".to_string(), 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }
}
