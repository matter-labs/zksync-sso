use alloy::primitives::U256;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize_u256_as_integer_string<S>(
    value: &U256,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u256_from_integer_string<'de, D>(
    deserializer: D,
) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<U256>().map_err(serde::de::Error::custom)
}
