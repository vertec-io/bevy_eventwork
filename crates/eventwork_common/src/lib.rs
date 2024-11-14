pub mod messages;
pub use messages::*;

pub mod codec;

pub mod error;

use serde::{Serialize, Deserialize};

use std::fmt::Debug;
use std::fmt::Display;


#[derive(Serialize, Deserialize)]
/// [`NetworkPacket`]s are untyped packets to be sent over the wire
pub struct NetworkPacket {
    /// Typically the NetworkMessage::NAME
    pub kind: String,
    /// The serialized message from bincode
    pub data: Vec<u8>,
}

impl Debug for NetworkPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkPacket")
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
/// A [`ConnectionId`] denotes a single connection
pub struct ConnectionId {
    /// The key of the connection.
    pub id: u32,
}

impl Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Connection with ID={0}", self.id))
    }
}
