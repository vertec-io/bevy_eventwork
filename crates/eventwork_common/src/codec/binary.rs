use codee::{Decoder, Encoder};
use serde::{Serialize, de::DeserializeOwned};
// use std::convert::TryInto;

use crate::error::NetworkError;

/// A custom codec that encodes data using bincode and adds a length prefix.
pub struct EventworkBincodeCodec;

impl<T: Serialize> Encoder<T> for EventworkBincodeCodec {
    type Error = NetworkError;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        // Serialize the data using bincode WITHOUT length prefix
        // WebSocket frames already provide message boundaries
        let serialized_data =
            bincode::serialize(val).map_err(|_err| NetworkError::Serialization)?;

        Ok(serialized_data)
    }
    // #[inline(always)]
    // fn encode(message: &T) -> Result<Self::Encoded, NetworkError> {

    //     // Serialize the message to bytes
    //     let serialized_msg = match bincode::serialize(&message) {
    //         Ok(msg) => msg,
    //         Err(_) => return Err(NetworkError::Serialization)
    //     };

    //     // Wrap the message in a network packet
    //     let packet = NetworkPacket {
    //         kind: T::NAME.to_string(),
    //         data: serialized_msg
    //     };

    //     // Serialize the packet
    //     let serialized_packet = match bincode::serialize(&packet) {
    //         Ok(pack) => pack,
    //         Err(_) => return Err(NetworkError::Serialization)
    //     };

    //     // Get the length of the packet, and write a
    //     // buffer with a length prefix and the serialized packet
    //     // buffer(bytes) = <len(bytes)><packet(bytes)>
    //     let len = serialized_packet.len() as u64;
    //     let mut buffer = Vec::new();
    //     buffer.extend_from_slice(&len.to_le_bytes());
    //     buffer.extend(&serialized_packet);

    //     Ok(buffer)
    // }
}

impl<T: DeserializeOwned> Decoder<T> for EventworkBincodeCodec {
    type Error = NetworkError;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        // Deserialize the data using bincode WITHOUT length prefix
        // WebSocket frames already provide message boundaries
        bincode::deserialize(val)
            .map_err(|_err| NetworkError::Serialization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bincode_codec() {
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Test {
            s: String,
            i: i32,
        }
        let t = Test {
            s: String::from("party time 🎉"),
            i: 42,
        };

        // Test encoding
        let enc = EventworkBincodeCodec::encode(&t).unwrap();
        assert!(enc.len() > 0); // Ensure there's data

        // Test decoding
        let dec: Test = EventworkBincodeCodec::decode(&enc).unwrap();
        assert_eq!(dec, t);
    }
}
