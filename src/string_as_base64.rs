use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&BASE64_STANDARD.encode(value))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  let encoded = String::deserialize(deserializer)?;
  match BASE64_STANDARD.decode(&encoded) {
    Ok(bytes) => match String::from_utf8(bytes) {
      Ok(decoded) => Ok(decoded),
      Err(_) => Err(serde::de::Error::custom("Failed to convert base64 to UTF-8 string")),
    },
    Err(_) => Err(serde::de::Error::custom("Failed to decode base64 string")),
  }
}
