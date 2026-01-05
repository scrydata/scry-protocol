//! Parameter value types for prepared statement parameters.

use serde::{Deserialize, Serialize};

/// Represents a typed parameter value from PostgreSQL's Bind message.
///
/// Covers core PostgreSQL types with an `Unknown` escape hatch for
/// extension types or unrecognized OIDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ParamValue {
    /// SQL NULL
    Null,

    /// Boolean (OID 16)
    Bool(bool),

    /// 16-bit integer (OID 21)
    Int16(i16),

    /// 32-bit integer (OID 23)
    Int32(i32),

    /// 64-bit integer (OID 20)
    Int64(i64),

    /// 32-bit float (OID 700)
    Float32(f32),

    /// 64-bit float (OID 701)
    Float64(f64),

    /// Arbitrary precision numeric as string (OID 1700)
    Numeric(String),

    /// Text string (OID 25, 1043 varchar, etc.)
    Text(String),

    /// Binary data (OID 17 bytea)
    #[serde(with = "base64_serde")]
    Bytes(Vec<u8>),

    /// Date as days since 2000-01-01 (OID 1082)
    Date(i32),

    /// Time as microseconds since midnight (OID 1083)
    Time(i64),

    /// Timestamp as microseconds since 2000-01-01 (OID 1114)
    Timestamp(i64),

    /// Timestamp with timezone as microseconds since 2000-01-01 UTC (OID 1184)
    TimestampTz(i64),

    /// Interval (OID 1186)
    Interval {
        months: i32,
        days: i32,
        microseconds: i64,
    },

    /// UUID as 16 bytes (OID 2950)
    #[serde(with = "uuid_serde")]
    Uuid([u8; 16]),

    /// JSON/JSONB as string (OID 114, 3802)
    Json(String),

    /// Array of values (OID varies)
    Array {
        elements: Vec<ParamValue>,
        dimensions: Vec<i32>,
    },

    /// Range type (OID varies)
    Range {
        lower: Option<Box<ParamValue>>,
        upper: Option<Box<ParamValue>>,
        lower_inc: bool,
        upper_inc: bool,
    },

    /// Composite/record type (OID varies)
    Composite { fields: Vec<ParamValue> },

    /// Unknown or extension type - escape hatch
    Unknown {
        oid: u32,
        #[serde(with = "base64_serde")]
        data: Vec<u8>,
    },
}

/// Base64 serialization for byte arrays
mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use base64::Engine;
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

/// UUID serialization as hex string
mod uuid_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 16], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        );
        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 16], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let hex: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        if hex.len() != 32 {
            return Err(serde::de::Error::custom("UUID must be 32 hex chars"));
        }
        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(serde::de::Error::custom)?;
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_value_null_roundtrip() {
        let val = ParamValue::Null;
        let json = serde_json::to_string(&val).unwrap();
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_int32_roundtrip() {
        let val = ParamValue::Int32(42);
        let json = serde_json::to_string(&val).unwrap();
        assert!(json.contains("\"type\":\"Int32\""));
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_text_roundtrip() {
        let val = ParamValue::Text("hello world".to_string());
        let json = serde_json::to_string(&val).unwrap();
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_bytes_base64() {
        let val = ParamValue::Bytes(vec![0x01, 0x02, 0x03]);
        let json = serde_json::to_string(&val).unwrap();
        assert!(json.contains("AQID")); // base64 of [1,2,3]
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_uuid_hex() {
        let val = ParamValue::Uuid([
            0x55, 0x06, 0x7d, 0xc5, 0xb9, 0x1c, 0x40, 0x78,
            0x90, 0x5b, 0x8a, 0x7f, 0xdd, 0x00, 0x83, 0x0c,
        ]);
        let json = serde_json::to_string(&val).unwrap();
        assert!(json.contains("55067dc5-b91c-4078-905b-8a7fdd00830c"));
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_array_roundtrip() {
        let val = ParamValue::Array {
            elements: vec![ParamValue::Int32(1), ParamValue::Int32(2)],
            dimensions: vec![2],
        };
        let json = serde_json::to_string(&val).unwrap();
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn test_param_value_unknown_roundtrip() {
        let val = ParamValue::Unknown {
            oid: 12345,
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        let json = serde_json::to_string(&val).unwrap();
        let parsed: ParamValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, parsed);
    }
}
