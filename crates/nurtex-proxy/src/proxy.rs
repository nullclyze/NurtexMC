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

const PROXY_VERSION: u8 = 0x05;
const CMD_CONNECT: u8 = 0x01;
const ATYP_IPV4: u8 = 0x01;
const ATYP_IPV6: u8 = 0x04;
const ATYP_DOMAIN: u8 = 0x03;

/// Структура SOCKS5 прокси
#[derive(Debug, Clone)]
pub struct Proxy {
  proxy_address: String,
  target: Arc<RwLock<TargetServer>>,
  timeout: u64,
  auth: Option<ProxyAuth>,
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

impl Proxy {
  /// Метод создания нового прокси
  pub fn new(proxy_address: impl Into<String>) -> Self {
    Self {
      proxy_address: proxy_address.into(),
      target: Arc::new(RwLock::new(TargetServer::default())),
      timeout: 20000,
      auth: None,
    }
  }

  /// Метод создания нового прокси с авторизацией
  pub fn new_with_auth(proxy_address: impl Into<String>, auth: ProxyAuth) -> Self {
    Self {
      proxy_address: proxy_address.into(),
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

  /// Метод создания подключения с SOCKS5 прокси
  pub async fn connect(&self) -> ProxyResult<TcpStream> {
    let (target_host, target_port) = {
      let guard = self.target.read().await;

      let Some(host) = guard.host.clone() else {
        return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidData, "target server host not specified"));
      };

      let Some(port) = guard.port else {
        return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidData, "target server port not specified"));
      };

      (host, port)
    };

    let mut stream = match timeout(Duration::from_millis(self.timeout), TcpStream::connect(&self.proxy_address)).await {
      Ok(result) => match result {
        Ok(s) => s,
        Err(_) => return ProxyResult::Failed(ProxyError::new(ErrorName::NotConnected, "could not connect to the specified server")),
      },
      Err(_) => return ProxyResult::Failed(ProxyError::new(ErrorName::Timeout, "Failed to connect to the server within the specified time")),
    };

    let mut greet = vec![PROXY_VERSION, if self.auth.is_some() { 2 } else { 1 }, 0x00];

    if self.auth.is_some() {
      greet = vec![PROXY_VERSION, 2, 0x00, 0x02];
    }

    stream.write_all(&greet).await.err();

    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await.err();

    if resp[0] != PROXY_VERSION {
      return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidVersion, "incompatible proxy version"));
    }

    match resp[1] {
      0x00 => {}
      0x02 => {
        if let Some(auth) = &self.auth {
          self.authorize(&mut stream, auth).await.err();
        } else {
          return ProxyResult::Failed(ProxyError::new(ErrorName::AuthFailed, "proxy requires authorization (username, password)"));
        }
      }
      _ => return ProxyResult::Failed(ProxyError::new(ErrorName::Unsupported, "unsupported authorization method")),
    }

    let mut req = BytesMut::with_capacity(512);
    req.put_u8(PROXY_VERSION);
    req.put_u8(CMD_CONNECT);
    req.put_u8(0x00);

    if let Ok(ipv4) = target_host.parse::<std::net::Ipv4Addr>() {
      req.put_u8(ATYP_IPV4);
      req.put_slice(&ipv4.octets());
    } else if let Ok(ipv6) = target_host.parse::<std::net::Ipv6Addr>() {
      req.put_u8(ATYP_IPV6);
      req.put_slice(&ipv6.octets());
    } else {
      req.put_u8(ATYP_DOMAIN);
      let host_bytes = target_host.as_bytes();

      if host_bytes.len() > 255 {
        return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidData, "target host is too long"));
      }

      req.put_u8(host_bytes.len() as u8);
      req.put_slice(host_bytes);
    }

    req.put_u16(target_port);

    stream.write_all(&req).await.err();

    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await.err();

    if header[0] != PROXY_VERSION {
      return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidVersion, "incompatible proxy version"));
    }

    let rep = header[1];

    if rep != 0x00 {
      return ProxyResult::Failed(ProxyError::new(ErrorName::NotConnected, format!("proxy connection error (rep: 0x{:02x})", rep)));
    }

    let atyp = header[3];

    match atyp {
      ATYP_IPV4 => {
        let mut addr = [0u8; 4 + 2];
        stream.read_exact(&mut addr).await.err();
      }
      ATYP_IPV6 => {
        let mut addr = [0u8; 16 + 2];
        stream.read_exact(&mut addr).await.err();
      }
      ATYP_DOMAIN => {
        let mut len = [0u8; 1];
        stream.read_exact(&mut len).await.err();
        let mut rest = vec![0u8; len[0] as usize + 2];
        stream.read_exact(&mut rest).await.err();
      }
      _ => return ProxyResult::Failed(ProxyError::new(ErrorName::InvalidData, format!("unknown ATYP in reply: 0x{:02x}", atyp))),
    }

    ProxyResult::Success(stream)
  }

  /// Метод выполнения авторизации
  async fn authorize(&self, stream: &mut TcpStream, auth: &ProxyAuth) -> Result<(), ProxyError> {
    let username = auth.username();
    let password = auth.password();

    if username.len() > 255 || password.len() > 255 {
      return Err(ProxyError::new(ErrorName::InvalidData, "username or password is too long"));
    }

    let mut buf = BytesMut::with_capacity(2 + username.len() + password.len());
    buf.put_u8(0x01); // 0x02 может быть
    buf.put_u8(username.len() as u8);
    buf.put_slice(username.as_bytes());
    buf.put_u8(password.len() as u8);
    buf.put_slice(password.as_bytes());

    stream.write_all(&buf).await?;

    let mut resp = [0u8; 2];

    stream.read_exact(&mut resp).await?;

    if resp[0] != 0x01 {
      return Err(ProxyError::new(ErrorName::AuthFailed, "invalid authorization version"));
    }

    if resp[1] != 0x00 {
      return Err(ProxyError::new(ErrorName::AuthFailed, "authorization failed (possibly incorrect password or username)"));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Error, ErrorKind};

  use tokio::io::{AsyncReadExt, AsyncWriteExt};

  use crate::Proxy;
  use crate::result::ProxyResult;

  #[tokio::test]
  async fn test_proxy_connect() -> std::io::Result<()> {
    let proxy = Proxy::new("212.58.132.5:1080"); // Это публичный прокси
    proxy.bind("ipinfo.io".to_string(), 80);

    let mut conn = match proxy.connect().await {
      ProxyResult::Success(s) => s,
      ProxyResult::Failed(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    conn.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await?;

    let mut buf = Vec::new();
    conn.read_to_end(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(&buf));

    Ok(())
  }
}
