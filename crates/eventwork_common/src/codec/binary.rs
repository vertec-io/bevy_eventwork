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
        // Serialize the data using bincode
        let serialized_data =
            bincode::serialize(val).map_err(|_err| NetworkError::Serialization)?;

        // Get the length of the serialized data
        let len = serialized_data.len() as u64;

        // Create a buffer with the length prefix and the serialized data
        let mut buffer = Vec::with_capacity(8 + serialized_data.len());
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(&serialized_data);

        Ok(buffer)
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
        // Add detailed logging to understand what we're receiving
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "[Codec] Decoding {} bytes. First 16 bytes (hex): {:02x?}",
                val.len(),
                &val[..val.len().min(16)]
            )));
        }

        if val.len() < 8 {
            // Not enough data to read the length prefix
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "[Codec] ERROR: Not enough bytes for length prefix. Got {} bytes",
                    val.len()
                )));
            }
            return Err(NetworkError::Serialization);
        }

        // Read the length prefix (first 8 bytes)
        let length_prefix = u64::from_le_bytes(
            val[..8].try_into().map_err(|_err| NetworkError::Serialization)?
        );

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "[Codec] Length prefix: {}, total bytes: {}, expected data bytes: {}",
                length_prefix, val.len(), 8 + length_prefix
            )));
        }

        // Check that the length of the remaining data matches the length prefix
        if val.len() < 8 + length_prefix as usize {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "[Codec] ERROR: Length mismatch. Expected {} bytes, got {} bytes",
                    8 + length_prefix, val.len()
                )));
            }
            return Err(NetworkError::Serialization);
        }

        // Deserialize the data using bincode (skip the 8-byte length prefix)
        let result = bincode::deserialize(&val[8..8 + length_prefix as usize])
            .map_err(|err| {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "[Codec] ERROR: Bincode deserialization failed: {:?}",
                        err
                    )));
                }
                NetworkError::Serialization
            });

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            if result.is_ok() {
                web_sys::console::log_1(&JsValue::from_str("[Codec] Successfully decoded message"));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_prefixed_bincode_codec() {
        #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Test {
            s: String,
            i: i32,
        }
        // impl NetworkMessage for Test{
        //     const NAME: &'static str = "eventwork:TestBinaryCodec";
        // }
        let t = Test {
            s: String::from("party time 🎉"),
            i: 42,
        };

        // Test encoding
        let enc = EventworkBincodeCodec::encode(&t).unwrap();
        assert!(enc.len() > 8); // Ensure there's enough data for a length prefix

        // Test decoding
        let dec: Test = EventworkBincodeCodec::decode(&enc).unwrap();
        assert_eq!(dec, t);
    }
}
