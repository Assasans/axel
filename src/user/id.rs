use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use jwt_simple::prelude::{Deserialize, Serialize};
use postgres_types::private::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

impl<'a> FromSql<'a> for UserId {
  fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
    let val = <i64 as FromSql>::from_sql(_ty, raw)?;
    Ok(UserId(val))
  }

  fn accepts(ty: &Type) -> bool {
    <i64 as ToSql>::accepts(ty)
  }
}

impl ToSql for UserId {
  fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    let val = self.0;
    <i64 as ToSql>::to_sql(&val, ty, out)
  }

  fn accepts(ty: &Type) -> bool {
    <i64 as ToSql>::accepts(ty)
  }

  fn to_sql_checked(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
    ToSql::to_sql(self, ty, out)
  }
}

impl UserId {
  pub fn new(id: i64) -> Self {
    Self(id)
  }
}

impl Display for UserId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Deref for UserId {
  type Target = i64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
