use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};

/// Тип потокобезопасного подключения
pub type Connection = Arc<RwLock<Option<crate::protocol::connection::NurtexConnection>>>;

/// Тип потокобезопасного `reader`
pub type PacketReader = Arc<broadcast::Sender<crate::protocol::connection::ClientsidePacket>>;

/// Тип потокобезопасного `writer`
pub type PacketWriter = Arc<broadcast::Sender<crate::protocol::packets::play::ServersidePlayPacket>>;
