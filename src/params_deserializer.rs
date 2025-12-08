use std::collections::HashMap;

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, Visitor};
use serde::Deserialize;

/// Error type for deserialization
#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for Error {}

impl de::Error for Error {
  fn custom<T: std::fmt::Display>(msg: T) -> Self {
    Error(msg.to_string())
  }
}

/// Deserializer for HashMap<String, String>
pub struct HashMapDeserializer<'de> {
  iter: std::collections::hash_map::Iter<'de, String, String>,
}

impl<'de> HashMapDeserializer<'de> {
  pub fn new(map: &'de HashMap<String, String>) -> Self {
    HashMapDeserializer { iter: map.iter() }
  }
}

/// Deserialize a HashMap<String, String> into any type T
pub fn from_hashmap<'de, T>(map: &'de HashMap<String, String>) -> Result<T, Error>
where
  T: Deserialize<'de>,
{
  let deserializer = HashMapDeserializer::new(map);
  T::deserialize(deserializer)
}

impl<'de> de::Deserializer<'de> for HashMapDeserializer<'de> {
  type Error = Error;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_map(visitor)
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_map(HashMapAccess {
      iter: self.iter,
      current_value: None,
    })
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.deserialize_map(visitor)
  }

  // Forward all other types to deserialize_any
  serde::forward_to_deserialize_any! {
    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
    bytes byte_buf option unit unit_struct newtype_struct seq tuple
    tuple_struct enum identifier ignored_any
  }
}

/// MapAccess implementation for iterating over the HashMap
struct HashMapAccess<'de> {
  iter: std::collections::hash_map::Iter<'de, String, String>,
  current_value: Option<&'de str>,
}

impl<'de> MapAccess<'de> for HashMapAccess<'de> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
  where
    K: DeserializeSeed<'de>,
  {
    match self.iter.next() {
      Some((key, value)) => {
        self.current_value = Some(value.as_str());
        seed.deserialize(key.as_str().into_deserializer()).map(Some)
      }
      None => Ok(None),
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
  where
    V: DeserializeSeed<'de>,
  {
    let value = self
      .current_value
      .take()
      .ok_or_else(|| de::Error::custom("value is missing"))?;
    seed.deserialize(StrDeserializer(value))
  }
}

/// Deserializer for individual string values that handles type conversion
struct StrDeserializer<'de>(&'de str);

impl<'de> de::Deserializer<'de> for StrDeserializer<'de> {
  type Error = Error;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_borrowed_str(self.0)
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    match self.0 {
      "true" | "1" => visitor.visit_bool(true),
      "false" | "0" => visitor.visit_bool(false),
      _ => Err(de::Error::custom(format!("cannot parse '{}' as bool", self.0))),
    }
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: i8 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_i8(value)
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: i16 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_i16(value)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: i32 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_i32(value)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: i64 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_i64(value)
  }

  fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: i128 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_i128(value)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: u8 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_u8(value)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: u16 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_u16(value)
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: u32 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_u32(value)
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: u64 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_u64(value)
  }

  fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: u128 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_u128(value)
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: f32 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_f32(value)
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let value: f64 = self.0.parse().map_err(de::Error::custom)?;
    visitor.visit_f64(value)
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let mut chars = self.0.chars();
    match (chars.next(), chars.next()) {
      (Some(c), None) => visitor.visit_char(c),
      _ => Err(de::Error::custom(format!("cannot parse '{}' as char", self.0))),
    }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_borrowed_str(self.0)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_string(self.0.to_owned())
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    // Empty string is treated as None, otherwise Some
    if self.0.is_empty() {
      visitor.visit_none()
    } else {
      visitor.visit_some(self)
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_newtype_struct(self)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_borrowed_bytes(self.0.as_bytes())
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_byte_buf(self.0.as_bytes().to_vec())
  }

  fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    serde_json::Deserializer::from_str(self.0)
      .deserialize_seq(_visitor)
      .map_err(de::Error::custom)
  }

  fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    Err(de::Error::custom("tuples are not supported"))
  }

  fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    Err(de::Error::custom("tuple structs are not supported"))
  }

  fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    Err(de::Error::custom("nested maps are not supported"))
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    Err(de::Error::custom("nested structs are not supported"))
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_enum(self.0.into_deserializer())
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_borrowed_str(self.0)
  }

  fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Deserialize, PartialEq)]
  struct Person {
    name: String,
    age: u32,
    score: f64,
    active: bool,
  }

  #[test]
  fn test_basic_deserialization() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Alice".to_string());
    map.insert("age".to_string(), "30".to_string());
    map.insert("score".to_string(), "95.5".to_string());
    map.insert("active".to_string(), "true".to_string());

    let person: Person = from_hashmap(&map).unwrap();

    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 30);
    assert_eq!(person.score, 95.5);
    assert!(person.active);
  }

  #[derive(Debug, Deserialize, PartialEq)]
  struct WithOptional {
    required: String,
    optional: Option<i32>,
  }

  #[test]
  fn test_optional_some() {
    let mut map = HashMap::new();
    map.insert("required".to_string(), "hello".to_string());
    map.insert("optional".to_string(), "42".to_string());

    let result: WithOptional = from_hashmap(&map).unwrap();

    assert_eq!(result.required, "hello");
    assert_eq!(result.optional, Some(42));
  }

  #[test]
  fn test_optional_none() {
    let mut map = HashMap::new();
    map.insert("required".to_string(), "hello".to_string());
    map.insert("optional".to_string(), "".to_string());

    let result: WithOptional = from_hashmap(&map).unwrap();

    assert_eq!(result.required, "hello");
    assert_eq!(result.optional, None);
  }

  #[derive(Debug, Deserialize, PartialEq)]
  enum Status {
    Active,
    Inactive,
    Pending,
  }

  #[derive(Debug, Deserialize, PartialEq)]
  struct WithEnum {
    status: Status,
  }

  #[test]
  fn test_enum_deserialization() {
    let mut map = HashMap::new();
    map.insert("status".to_string(), "Active".to_string());

    let result: WithEnum = from_hashmap(&map).unwrap();

    assert_eq!(result.status, Status::Active);
  }
}
