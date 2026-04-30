use std::sync::Arc;

use tokio::sync::RwLock;

use crate::protocol::connection::NurtexConnection;
use crate::bot::BotComponents;
use crate::storage::Storage;

/// Функция временного захвата подключения
pub async fn capture_connection<F>(connection: &Arc<RwLock<Option<NurtexConnection>>>, f: F) -> std::io::Result<()>
where
  F: AsyncFnOnce(&NurtexConnection) -> std::io::Result<()>,
{
  let guard = connection.read().await;
  let Some(conn) = guard.as_ref() else {
    return Ok(());
  };

  f(conn).await
}

/// Функция временного захвата компонентов
pub async fn capture_components<F>(components: &Arc<RwLock<BotComponents>>, f: F)
where
  F: AsyncFnOnce(&mut BotComponents),
{
  let mut guard = components.write().await;
  f(&mut *guard).await
}

/// Функция временного захвата хранилища
pub async fn capture_storage<F>(storage: &Arc<RwLock<Storage>>, f: F)
where
  F: AsyncFnOnce(&mut Storage),
{
  let mut guard = storage.write().await;
  f(&mut *guard).await
}
