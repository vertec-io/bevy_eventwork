use codee::{Decoder, Encoder};
use serde::de::DeserializeOwned;

use crate::{error::NetworkError, EventworkMessage, NetworkPacket};

/// Primary codec for eventwork WebSocket communication (multi-message mode).
///
/// This codec handles the eventwork WebSocket protocol:
/// - 8-byte little-endian length prefix
/// - Bincode-serialized NetworkPacket
///
/// This is the **recommended codec** for production applications as it supports
/// multiple message types over a single WebSocket connection.
///
/// ## Usage
///
/// ```rust,ignore
/// use eventwork_common::codec::EventworkBincodeCodec;
/// use eventwork_common::NetworkPacket;
///
/// let ws = use_websocket_with_options::<NetworkPacket, NetworkPacket, EventworkBincodeCodec>(
///     "ws://127.0.0.1:8081",
///     options
/// );
/// ```
///
/// The application layer is responsible for:
/// - Wrapping messages in NetworkPacket before sending
/// - Unwrapping NetworkPacket after receiving
/// - Routing messages based on NetworkPacket.type_name
pub struct EventworkBincodeCodec;

// Multi-message encoder: accepts NetworkPacket directly (already wrapped by application)
impl Encoder<NetworkPacket> for EventworkBincodeCodec {
    type Error = NetworkError;
    type Encoded = Vec<u8>;

    fn encode(val: &NetworkPacket) -> Result<Self::Encoded, Self::Error> {
        // The NetworkPacket is already created by the application layer
        // We just need to encode it with bincode and add the length prefix
        let encoded_packet = bincode::serialize(val)
            .map_err(|_| NetworkError::Serialization)?;

        let len = encoded_packet.len() as u64;
        let mut buffer = Vec::with_capacity(8 + encoded_packet.len());
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(&encoded_packet);

        Ok(buffer)
    }
}

// Multi-message decoder: returns NetworkPacket directly (application handles routing)
impl Decoder<NetworkPacket> for EventworkBincodeCodec {
    type Error = NetworkError;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<NetworkPacket, Self::Error> {
        if val.len() < 8 {
            return Err(NetworkError::Serialization);
        }

        let length_bytes: [u8; 8] = val[..8]
            .try_into()
            .map_err(|_| NetworkError::Serialization)?;
        let _length = u64::from_le_bytes(length_bytes);

        // Decode directly to NetworkPacket
        // The application layer will handle unwrapping and routing
        bincode::deserialize(&val[8..])
            .map_err(|_| NetworkError::Serialization)
    }
}

/// Convenience codec for single message-type connections.
///
/// This codec automatically wraps/unwraps NetworkPacket, making it convenient
/// for simple examples with dedicated connections per message type.
///
/// ## Usage
///
/// ```rust,ignore
/// use eventwork_common::codec::EventworkBincodeSingleMsgCodec;
///
/// let ws = use_websocket_with_options::<UserChatMessage, NewChatMessage, EventworkBincodeSingleMsgCodec>(
///     "ws://127.0.0.1:8081",
///     options
/// );
/// ```
///
/// **Note:** For production applications with multiple message types, use `EventworkBincodeCodec` instead.
pub struct EventworkBincodeSingleMsgCodec;

// Single-message type encoder: wraps T in NetworkPacket
impl<T: EventworkMessage> Encoder<T> for EventworkBincodeSingleMsgCodec {
    type Error = NetworkError;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        // Wrap the message in NetworkPacket
        let packet = NetworkPacket {
            type_name: T::type_name().to_string(),
            schema_hash: T::schema_hash(),
            data: bincode::serialize(val).map_err(|_| NetworkError::Serialization)?,
        };

        // Encode the NetworkPacket with bincode
        let encoded_packet = bincode::serialize(&packet)
            .map_err(|_| NetworkError::Serialization)?;

        // Prepend the 8-byte length prefix (REQUIRED for WebSocket protocol)
        let len = encoded_packet.len() as u64;
        let mut buffer = Vec::with_capacity(8 + encoded_packet.len());
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(&encoded_packet);

        Ok(buffer)
    }
}

// Single-message type decoder: unwraps NetworkPacket to get T
impl<T: DeserializeOwned> Decoder<T> for EventworkBincodeSingleMsgCodec {
    type Error = NetworkError;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        // Read the 8-byte length prefix
        if val.len() < 8 {
            return Err(NetworkError::Serialization);
        }

        let length_bytes: [u8; 8] = val[..8]
            .try_into()
            .map_err(|_| NetworkError::Serialization)?;
        let _length = u64::from_le_bytes(length_bytes);

        // Decode the NetworkPacket
        let packet: NetworkPacket = bincode::deserialize(&val[8..])
            .map_err(|_| NetworkError::Serialization)?;

        // Decode the message from the packet's data
        bincode::deserialize(&packet.data)
            .map_err(|_| NetworkError::Serialization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_message_codec() {
        // Test the multi-message codec (EventworkBincodeCodec)
        // This codec works with NetworkPacket directly

        let packet = NetworkPacket {
            type_name: "TestMessage".to_string(),
            schema_hash: 0x1234567890abcdef,
            data: vec![1, 2, 3, 4, 5],
        };

        // Test encoding
        let enc = EventworkBincodeCodec::encode(&packet).unwrap();

        // Should have 8-byte length prefix + encoded packet
        assert!(enc.len() > 8);

        // First 8 bytes should be the length
        let length_bytes: [u8; 8] = enc[..8].try_into().unwrap();
        let length = u64::from_le_bytes(length_bytes);
        assert_eq!(length as usize, enc.len() - 8);

        // Test decoding
        let dec: NetworkPacket = EventworkBincodeCodec::decode(&enc).unwrap();
        assert_eq!(dec.type_name, packet.type_name);
        assert_eq!(dec.schema_hash, packet.schema_hash);
        assert_eq!(dec.data, packet.data);
    }

    #[test]
    fn test_single_message_codec() {
        // Test the single-message codec (EventworkBincodeSingleMsgCodec)
        // This codec automatically wraps/unwraps NetworkPacket

        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct TestMessage {
            s: String,
            i: i32,
        }

        let msg = TestMessage {
            s: String::from("party time 🎉"),
            i: 42,
        };

        // Test encoding (wraps in NetworkPacket automatically)
        let enc = EventworkBincodeSingleMsgCodec::encode(&msg).unwrap();

        // Should have 8-byte length prefix + encoded NetworkPacket
        assert!(enc.len() > 8);

        // First 8 bytes should be the length
        let length_bytes: [u8; 8] = enc[..8].try_into().unwrap();
        let length = u64::from_le_bytes(length_bytes);
        assert_eq!(length as usize, enc.len() - 8);

        // Test decoding (unwraps NetworkPacket automatically)
        let dec: TestMessage = EventworkBincodeSingleMsgCodec::decode(&enc).unwrap();
        assert_eq!(dec, msg);
    }
}
