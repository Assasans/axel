use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Mutex;

use aes::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use jwt_simple::prelude::{Deserialize, Serialize};
use postgres_types::private::BytesMut;
use postgres_types::{IsNull, Type};
use rand::random;
use tokio_postgres::types::{FromSql, ToSql};
use tracing::{debug, info};

use crate::api_server::{Aes128CbcEnc, AES_IV, AES_KEY};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

impl<'a> FromSql<'a> for UserId {
  fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
    let val = <i64 as FromSql>::from_sql(_ty, raw)?;
    Ok(UserId(val))
  }

  fn accepts(ty: &Type) -> bool {
    <i64 as FromSql>::accepts(ty)
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

/// Represents a logged in player.
pub struct Session {
  pub user_id: UserId,
  pub user_key: Mutex<Option<[u8; 16]>>,
  pub device_token: Option<String>,
}

impl Session {
  pub fn new(user_id: UserId, device_token: Option<String>) -> Self {
    Self {
      user_id,
      user_key: Mutex::new(None),
      device_token,
    }
  }

  pub fn rotate_user_key(&self) {
    let mut key = self.user_key.lock().unwrap();
    let key = key.insert(random());
    info!("updated user key: {}", const_hex::encode(key));
  }

  pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
    Aes128CbcEnc::new(
      AES_KEY.into(),
      self.user_key.lock().unwrap().as_ref().unwrap_or(&AES_IV).into(),
    )
    .encrypt_padded_vec_mut::<Pkcs7>(data)
  }

  pub fn encrypt_and_sign(&self, data: &[u8]) -> Result<(Vec<u8>, String), jwt_simple::Error> {
    let encrypted = self.encrypt(data);
    let digest = md5::compute(&encrypted);
    debug!("digest: {:?}", digest);

    let key_pair = RS256KeyPair::from_pem(include_str!("../key.pem")).unwrap();
    let mut custom = BTreeMap::new();
    custom.insert("cs".to_owned(), const_hex::encode(*digest));
    if let Some(user_key) = &*self.user_key.lock().unwrap() {
      custom.insert("uk".to_owned(), const_hex::encode(user_key));
    }

    let claims = JWTClaims {
      issued_at: None,
      expires_at: None,
      invalid_before: None,
      issuer: None,
      subject: None,
      audiences: None,
      jwt_id: None,
      nonce: None,
      custom,
    };
    let token = key_pair.sign(claims)?;
    debug!("response jwt: {}", token);

    Ok((encrypted, token))
  }
}
