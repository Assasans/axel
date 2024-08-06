use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_u8(if *value { 1 } else { 0 })
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  let value = u8::deserialize(deserializer)?;
  match value {
    1 => Ok(true),
    0 => Ok(false),
    _ => Err(serde::de::Error::custom("Expected 1 or 0 for a boolean value")),
  }
}
