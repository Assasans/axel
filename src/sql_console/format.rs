use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use postgres_types::Type;
use serde_json::Value;
use tokio_postgres::Row;

pub fn format_cell(row: &Row, idx: usize) -> Value {
  let col = &row.columns()[idx];
  let ty = col.type_();

  match *ty {
    Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
      opt(row.try_get::<usize, Option<String>>(idx)).map_or(Value::Null, Value::String)
    }

    Type::INT2 => opt(row.try_get::<usize, Option<i16>>(idx)).map_or(Value::Null, |v| Value::Number(v.into())),
    Type::INT4 => opt(row.try_get::<usize, Option<i32>>(idx)).map_or(Value::Null, |v| Value::Number(v.into())),
    Type::INT8 => opt(row.try_get::<usize, Option<i64>>(idx)).map_or(Value::Null, |v| Value::Number(v.into())),

    Type::FLOAT4 => float(row.try_get::<usize, Option<f32>>(idx)),
    Type::FLOAT8 => float(row.try_get::<usize, Option<f64>>(idx)),

    Type::BOOL => opt(row.try_get::<usize, Option<bool>>(idx)).map_or(Value::Null, Value::Bool),

    Type::JSON | Type::JSONB => {
      row.try_get::<usize, Option<Value>>(idx).unwrap_or(None).unwrap_or(Value::Null)
    }

    // Type::UUID => opt(row.try_get::<usize, Option<Uuid>>(idx))
    //   .map_or(Value::Null, |v| Value::String(v.to_string())),

    Type::DATE => opt(row.try_get::<usize, Option<NaiveDate>>(idx))
      .map_or(Value::Null, |v| Value::String(v.to_string())),

    Type::TIMESTAMP => opt(row.try_get::<usize, Option<NaiveDateTime>>(idx))
      .map_or(Value::Null, |v| Value::String(v.to_string())),

    Type::TIMESTAMPTZ => opt(row.try_get::<usize, Option<DateTime<Utc>>>(idx))
      .map_or(Value::Null, |v| Value::String(v.to_rfc3339())),

    // Type::BYTEA => opt(row.try_get::<usize, Option<Bytes>>(idx))
    //   .map_or(Value::Null, |v| Value::String(base64::encode(v))),

    Type::INT4_ARRAY => array(row.try_get::<usize, Option<Vec<i32>>>(idx)),
    Type::TEXT_ARRAY => array(row.try_get::<usize, Option<Vec<String>>>(idx)),
    // Type::UUID_ARRAY => array(row.try_get::<usize, Option<Vec<Uuid>>>(idx)),

    _ => {
      Value::String(format!("<unsupported:{}>", ty.name()))
    }
  }
}

fn opt<T>(r: Result<Option<T>, tokio_postgres::Error>) -> Option<T> {
  r.ok().flatten()
}

fn float<T: Into<f64>>(r: Result<Option<T>, tokio_postgres::Error>) -> Value {
  r.ok()
    .flatten()
    .and_then(|v| serde_json::Number::from_f64(v.into()))
    .map(Value::Number)
    .unwrap_or(Value::Null)
}

fn array<T: Into<Value>>(r: Result<Option<Vec<T>>, tokio_postgres::Error>) -> Value {
  r.ok()
    .flatten()
    .map(|v| Value::Array(v.into_iter().map(Into::into).collect()))
    .unwrap_or(Value::Null)
}
