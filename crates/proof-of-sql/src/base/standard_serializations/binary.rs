use alloc::vec::Vec;
use bincode::{
    config::Config,
    error::{DecodeError, EncodeError},
};
use serde::{Deserialize, Serialize};

fn standard_binary_config() -> impl Config {
    bincode::config::legacy()
        .with_fixed_int_encoding()
        .with_big_endian()
}

/// The standard serialization we use for our proof types
pub fn try_standard_binary_serialization(
    value_to_be_serialized: impl Serialize,
) -> Result<Vec<u8>, EncodeError> {
    bincode::serde::encode_to_vec(value_to_be_serialized, standard_binary_config())
}

/// The standard deserialization we use for our proof types
pub fn try_standard_binary_deserialization<D: for<'a> Deserialize<'a>>(
    value_to_be_deserialized: &[u8],
) -> Result<(D, usize), DecodeError> {
    bincode::serde::decode_from_slice(value_to_be_deserialized, standard_binary_config())
}

#[cfg(test)]
mod tests {
    use super::{try_standard_binary_deserialization, try_standard_binary_serialization};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct SerdeTestType {
        a: String,
        b: bool,
        c: i32,
    }

    #[test]
    fn round_trip() {
        let obj = SerdeTestType {
            a: "test".to_string(),
            b: false,
            c: 123,
        };
        let serialized = try_standard_binary_serialization(obj.clone()).unwrap();
        let (deserialized, _): (SerdeTestType, _) =
            try_standard_binary_deserialization(&serialized).unwrap();
        assert_eq!(obj, deserialized);
        let reserialized = try_standard_binary_serialization(deserialized).unwrap();
        assert_eq!(serialized, reserialized);
    }

    #[test]
    fn serialization_uses_fixed_width_big_endian_encoding() {
        let obj = SerdeTestType {
            a: "ok".to_string(),
            b: true,
            c: 0x0102_0304,
        };

        let serialized = try_standard_binary_serialization(obj).unwrap();
        assert_eq!(
            serialized,
            vec![
                // String length as a fixed-width big-endian u64, followed by bytes.
                0, 0, 0, 0, 0, 0, 0, 2, b'o', b'k',
                // bool and i32 as fixed-width big-endian bytes.
                1, 1, 2, 3, 4,
            ]
        );
    }

    #[test]
    fn deserialization_reports_consumed_bytes_and_leaves_trailing_data() {
        let obj = SerdeTestType {
            a: "tail".to_string(),
            b: false,
            c: -42,
        };
        let mut serialized = try_standard_binary_serialization(obj.clone()).unwrap();
        let object_len = serialized.len();
        serialized.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let (deserialized, consumed): (SerdeTestType, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, obj);
        assert_eq!(consumed, object_len);
        assert_eq!(&serialized[consumed..], &[0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn deserialization_rejects_invalid_bool_byte() {
        let mut encoded_invalid_bool = Vec::new();
        // Empty string length.
        encoded_invalid_bool.extend_from_slice(&0_u64.to_be_bytes());
        // bincode accepts only 0 or 1 for bool.
        encoded_invalid_bool.push(2);
        // i32 payload that should not make the invalid bool acceptable.
        encoded_invalid_bool.extend_from_slice(&7_i32.to_be_bytes());

        let error = try_standard_binary_deserialization::<SerdeTestType>(&encoded_invalid_bool)
            .expect_err("invalid bool discriminant must fail to deserialize");
        assert!(error.to_string().contains("InvalidBooleanValue"));
    }

    #[test]
    fn deserialization_rejects_truncated_fixed_width_integer() {
        let mut truncated = Vec::new();
        truncated.extend_from_slice(&0_u64.to_be_bytes());
        truncated.push(1);
        // Only three of the four big-endian i32 payload bytes are present.
        truncated.extend_from_slice(&[0x01, 0x02, 0x03]);

        let error = try_standard_binary_deserialization::<SerdeTestType>(&truncated)
            .expect_err("truncated fixed-width i32 must fail to deserialize");
        assert!(error.to_string().contains("UnexpectedEnd"));
    }

    #[test]
    fn deserialization_rejects_truncated_string_payload() {
        let mut truncated = Vec::new();
        // Claim a three-byte string but provide only two bytes.
        truncated.extend_from_slice(&3_u64.to_be_bytes());
        truncated.extend_from_slice(b"ab");

        let error = try_standard_binary_deserialization::<SerdeTestType>(&truncated)
            .expect_err("truncated string payload must fail to deserialize");
        assert!(error.to_string().contains("UnexpectedEnd"));
    }
}
