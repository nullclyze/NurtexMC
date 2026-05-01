use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::Proxy;
use crate::result::ProxyResult;

/// Структура информации об IP
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IpInfo {
  pub ip: String,
  pub hostname: String,
  pub city: String,
  pub region: String,
  pub country: String,
  pub loc: String,
  pub org: String,
  pub postal: String,
  pub timezone: String,
  pub readme: String,
}

/// Трейт прокси чекера
pub trait ProxyChecker {
  /// Метод проверки работоспособности прокси, используя `ipinfo.io`.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use nurtex_proxy::{Proxy, ProxyChecker};
  ///
  /// // Создаём прокси
  /// let proxy = Proxy::new("PROXY_IP:PROXY_PORT");
  ///
  /// // Проверяем доступность прокси
  /// if proxy.check_proxy().await {
  ///   println!("Прокси доступен");
  /// } else {
  ///   println!("Прокси недоступен");
  /// }
  /// ```
  fn check_proxy(&self) -> impl std::future::Future<Output = bool> + Send;

  /// Метод получения информации об IP с `ipinfo.io`.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use nurtex_proxy::{Proxy, ProxyChecker};
  ///
  /// // Создаём прокси и получаем информацию об IP
  /// let proxy = Proxy::new("PROXY_IP:PROXY_PORT");
  /// let ip_info = proxy.get_ip_info().await;
  ///
  /// println!("Имя хоста: {}", ip_info.hostname);
  /// println!("Страна: {}", ip_info.country);
  /// println!("Город: {}", ip_info.city);
  /// println!("Локация: {}", ip_info.loc);
  /// ```
  fn get_ip_info(&self) -> impl std::future::Future<Output = Option<IpInfo>> + Send;
}

impl ProxyChecker for Proxy {
  async fn check_proxy(&self) -> bool {
    if !self.is_available().await {
      return false;
    }

    let ip_info = match self.get_ip_info().await {
      Some(info) => info,
      None => return false,
    };

    if let Some(ip) = self.get_ip() {
      if ip != ip_info.ip {
        return false;
      }
    }

    true
  }

  async fn get_ip_info(&self) -> Option<IpInfo> {
    self.bind("ipinfo.io".to_string(), 80);

    let mut stream = match self.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(_) => return None,
    };

    let _ = stream.write_all(b"GET / HTTP/1.0\r\nHost: ipinfo.io\r\n\r\n").await;

    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await;

    let data = match String::from_utf8(buf) {
      Ok(s) => s,
      Err(_) => String::new(),
    };

    let split_data = data.split("\n").collect::<Vec<&str>>();
    let mut pretty_data = String::new();

    // Здесь можно просто пропустить заголовки
    for (i, item) in split_data.iter().enumerate() {
      if i < 7 {
        continue;
      }

      pretty_data.push_str(*item);
    }

    let ip_info: IpInfo = match serde_json::from_str(&pretty_data) {
      Ok(info) => info,
      Err(_) => return None,
    };

    Some(ip_info)
  }
}

#[cfg(test)]
mod tests {
  use crate::{Proxy, ProxyChecker, ProxyType};

  #[tokio::test]
  async fn test_proxy_check() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    if proxy.check_proxy().await {
      println!("Доступен");
    } else {
      println!("Недоступен");
    }
  }

  #[tokio::test]
  async fn test_get_ip_info() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);
    println!("Информация об IP: {:?}", proxy.get_ip_info().await);
  }
}
