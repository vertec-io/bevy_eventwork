// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

pub mod network_message_file;
pub use network_message_file::*;
use serde::{Serialize, Deserialize};

use std::fmt::Debug;

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