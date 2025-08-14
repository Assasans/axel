// https://github.com/rnag/serde-this-or-that
// Copyright (c) 2022, Ritvik Nag. Licensed under the MIT Licence.
// See the LICENCE-MIT file in the repository root for full licence text.

use std::fmt;

use serde::de::Unexpected;
use serde::{de, Deserializer};

struct DeserializeI64WithVisitor;

impl de::Visitor<'_> for DeserializeI64WithVisitor {
  type Value = i64;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a signed integer or a string")
  }

  fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v)
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match i64::try_from(v) {
      Ok(v) => Ok(v),
      Err(_) => Err(E::custom(format!(
        "overflow: Unable to convert unsigned value `{v:?}` to i64"
      ))),
    }
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v.round() as i64)
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    if let Ok(n) = v.parse::<i64>() {
      Ok(n)
    } else if v.is_empty() {
      Ok(0)
    } else if let Ok(f) = v.parse::<f64>() {
      Ok(f.round() as i64)
    } else {
      Err(E::invalid_value(Unexpected::Str(v), &self))
    }
  }

  /// We encounter a `null` value; this default implementation returns a
  /// "zero" value.
  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(0)
  }
}

pub fn as_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(DeserializeI64WithVisitor)
}

struct DeserializeI32WithVisitor;

impl de::Visitor<'_> for DeserializeI32WithVisitor {
  type Value = i32;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a signed integer or a string")
  }

  fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v)
  }

  fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match i32::try_from(v) {
      Ok(v) => Ok(v),
      Err(_) => Err(E::custom(format!(
        "overflow: Unable to convert unsigned value `{v:?}` to i32"
      ))),
    }
  }

  fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(v.round() as i32)
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    if let Ok(n) = v.parse::<i32>() {
      Ok(n)
    } else if v.is_empty() {
      Ok(0)
    } else if let Ok(f) = v.parse::<f64>() {
      Ok(f.round() as i32)
    } else {
      Err(E::invalid_value(Unexpected::Str(v), &self))
    }
  }

  /// We encounter a `null` value; this default implementation returns a
  /// "zero" value.
  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(0)
  }
}

pub fn as_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(DeserializeI32WithVisitor)
}
