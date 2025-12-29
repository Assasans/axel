use std::collections::BTreeMap;
use std::sync::Mutex;

use aes::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use rand::random;
use tracing::trace;

use crate::api_server::{Aes128CbcEnc, AES_IV, AES_KEY};
use crate::user::id::UserId;

/// Represents a logged in player.
pub struct Session {
  pub user_id: UserId,
  pub user_key: Mutex<Option<[u8; 16]>>,
  pub device_token: Option<String>,
  cached_username: Mutex<Option<String>>,
}

impl Session {
  pub fn new(user_id: UserId, device_token: Option<String>) -> Self {
    Self {
      user_id,
      user_key: Mutex::new(None),
      device_token,
      cached_username: Mutex::new(None),
    }
  }

  pub fn rotate_user_key(&self) {
    let mut key = self.user_key.lock().unwrap();
    let key = key.insert(random());
    trace!("rotated user key: {}", const_hex::encode(key));
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
    trace!("digest: {:?}", digest);

    let key_pair = RS256KeyPair::from_pem(include_str!("../../key.pem")).unwrap();
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
    trace!("response jwt: {}", token);

    Ok((encrypted, token))
  }

  /// Retrieves the cached username, if available.
  pub fn get_cached_username(&self) -> Option<String> {
    self.cached_username.lock().unwrap().clone()
  }

  pub fn set_cached_username(&self, username: Option<String>) {
    let mut cached_username = self.cached_username.lock().unwrap();
    *cached_username = username;
  }
}
