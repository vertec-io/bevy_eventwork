use codee::{Decoder, Encoder};
use serde::{de::{DeserializeOwned, Error}, Serialize};
use std::convert::TryInto;

/// A custom codec that encodes data using bincode and adds a length prefix.
pub struct EventworkBincodeCodec;

impl<T: Serialize> Encoder<T> for EventworkBincodeCodec {
    type Error = bincode::Error;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        // Serialize the data using bincode
        let serialized_data = bincode::serialize(val)?;

        // Get the length of the serialized data
        let len = serialized_data.len() as u64;

        // Create a buffer with the length prefix and the serialized data
        let mut buffer = Vec::with_capacity(8 + serialized_data.len());
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(&serialized_data);

        Ok(buffer)
    }
}

impl<T: DeserializeOwned> Decoder<T> for EventworkBincodeCodec {
    type Error = bincode::Error;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        if val.len() < 8 {
            // Not enough data to read the length prefix
            return Err(bincode::Error::custom("Data is too short to contain length prefix"));
        }

        // Read the length prefix (first 8 bytes)
        let length_prefix = u64::from_le_bytes(val[..8].try_into().expect("Invalid length prefix"));

        // Check that the length of the remaining data matches the length prefix
        if val.len() < 8 + length_prefix as usize {
            return Err(bincode::Error::custom("Data length does not match length prefix"));
        }

        // Deserialize the data using bincode
        bincode::deserialize(&val[8..8 + length_prefix as usize])
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
