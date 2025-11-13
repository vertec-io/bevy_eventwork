use codee::{Decoder, Encoder};
use serde::de::DeserializeOwned;
use eventwork_common::{NetworkPacket, EventworkMessage};

/// Wrapper codec for use with leptos_use that matches eventwork's WebSocket protocol.
///
/// The eventwork WebSocket protocol uses:
/// 1. An 8-byte length prefix (u64, little-endian)
/// 2. A bincode-encoded NetworkPacket containing the message
///
/// This is different from the EventworkBincodeCodec which only handles the message encoding.
/// The WebSocket transport layer adds the length prefix.
pub struct EventworkBincodeCodec;

impl<T: EventworkMessage> Encoder<T> for EventworkBincodeCodec {
    type Error = String;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        // First, wrap the message in a NetworkPacket
        let packet = NetworkPacket {
            type_name: T::type_name().to_string(),
            schema_hash: T::schema_hash(),
            data: bincode::serialize(val).map_err(|e| format!("Failed to serialize message: {}", e))?,
        };

        // Encode the NetworkPacket with bincode
        let encoded_packet = bincode::serialize(&packet)
            .map_err(|e| format!("Failed to serialize NetworkPacket: {}", e))?;

        // Prepend the 8-byte length prefix
        let len = encoded_packet.len() as u64;
        let mut buffer = Vec::with_capacity(8 + encoded_packet.len());
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(&encoded_packet);

        log::debug!("Encoded {} total bytes (8-byte prefix + {} packet bytes) for type: {} (hash: 0x{:016x})",
                    buffer.len(), encoded_packet.len(), T::type_name(), T::schema_hash());

        Ok(buffer)
    }
}

impl<T: DeserializeOwned> Decoder<T> for EventworkBincodeCodec {
    type Error = String;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        log::debug!("Decoding {} bytes", val.len());

        // The incoming data should have an 8-byte length prefix
        if val.len() < 8 {
            return Err(format!("Message too short: {} bytes", val.len()));
        }

        // Read the length prefix
        let length_bytes: [u8; 8] = val[..8].try_into()
            .map_err(|_| "Failed to read length prefix".to_string())?;
        let length = u64::from_le_bytes(length_bytes) as usize;

        log::debug!("Length prefix: {}, actual data length: {}", length, val.len() - 8);

        // Decode the NetworkPacket
        let packet: NetworkPacket = bincode::deserialize(&val[8..])
            .map_err(|e| format!("Failed to deserialize NetworkPacket: {}", e))?;

        // Decode the message from the packet's data
        bincode::deserialize(&packet.data)
            .map_err(|e| format!("Failed to deserialize message: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_roundtrip() {
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct TestMessage {
            text: String,
            count: i32,
        }

        let msg = TestMessage {
            text: "Hello, WebSocket! 🚀".to_string(),
            count: 42,
        };

        let encoded = EventworkBincodeCodec::encode(&msg).unwrap();
        let decoded: TestMessage = EventworkBincodeCodec::decode(&encoded).unwrap();

        assert_eq!(decoded, msg);
    }
}

