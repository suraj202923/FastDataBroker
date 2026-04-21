//! Binary serialization module using Bincode for optimal performance
//! 
//! Bincode provides 2-5x faster serialization than JSON for binary data.
//! This module wraps Bincode to provide simple encode/decode APIs.

use serde::{Deserialize, Serialize};

/// Encode a value to binary format using Bincode
/// 
/// # Arguments
/// * `value` - The value to encode
/// 
/// # Returns
/// * `Ok(Vec<u8>)` - The encoded bytes
/// * `Err(String)` - Error message if encoding fails
#[inline]
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    bincode::serialize(value).map_err(|e| format!("Bincode encode error: {}", e))
}

/// Decode a value from binary format using Bincode
/// 
/// # Arguments
/// * `data` - The bytes to decode
/// 
/// # Returns
/// * `Ok(T)` - The decoded value
/// * `Err(String)` - Error message if decoding fails
#[inline]
pub fn decode<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T, String> {
    bincode::deserialize(data).map_err(|e| format!("Bincode decode error: {}", e))
}

/// Get the approximate size in bytes of a serialized value
/// 
/// This is useful for monitoring message sizes and for quota enforcement.
#[inline]
pub fn approximate_size<T: Serialize>(value: &T) -> Result<usize, String> {
    encode(value).map(|bytes| bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        id: u64,
        name: String,
        data: Vec<u8>,
    }

    #[test]
    fn test_encode_decode() {
        let msg = TestMessage {
            id: 42,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let encoded = encode(&msg).expect("Encoding failed");
        let decoded: TestMessage = decode(&encoded).expect("Decoding failed");

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_size_calculation() {
        let msg = TestMessage {
            id: 42,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let size = approximate_size(&msg).expect("Size calculation failed");
        assert!(size > 0);
        assert!(size < 1000); // Should be reasonably small
    }

    #[test]
    fn test_bincode_vs_json_size() {
        // Bincode should be significantly smaller than JSON for binary data
        use serde_json;

        let msg = TestMessage {
            id: 42,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let bincode_size = encode(&msg).expect("Bincode encoding failed").len();
        let json_size = serde_json::to_string(&msg)
            .expect("JSON encoding failed")
            .len();

        // Bincode should be smaller for this data
        println!(
            "Bincode size: {}, JSON size: {}, Ratio: {:.2}%",
            bincode_size,
            json_size,
            (bincode_size as f64 / json_size as f64) * 100.0
        );

        assert!(bincode_size < json_size);
    }
}
