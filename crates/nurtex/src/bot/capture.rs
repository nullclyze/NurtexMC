use std::io;
use std::sync::Arc;

use nurtex_protocol::connection::NurtexConnection;
use tokio::sync::RwLock;

use crate::bot::BotComponents;

/// Функция временного захвата подключения
pub async fn capture_connection<F>(connection: &Arc<RwLock<Option<NurtexConnection>>>, f: F) -> io::Result<()>
where
  F: AsyncFnOnce(&NurtexConnection) -> io::Result<()>,
{
  let guard = connection.read().await;
  let Some(conn) = guard.as_ref() else {
    return Ok(());
  };

  f(conn).await
}

/// Функция временного захвата компонентов
pub async fn capture_components<F>(components: &Arc<RwLock<BotComponents>>, f: F) -> io::Result<()>
where
  F: AsyncFnOnce(&mut BotComponents) -> io::Result<()>,
{
  let mut guard = components.write().await;
  f(&mut *guard).await
}
