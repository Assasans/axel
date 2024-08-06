use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Mutex;

use aes::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use jwt_simple::prelude::{Deserialize, Serialize};
use rand::random;
use tracing::{debug, info};

use crate::{Aes128CbcEnc, AES_IV, AES_KEY};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(u64);

impl Deref for UserId {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// Represents a logged in player.
pub struct Session {
  pub user_id: UserId,
  pub user_key: Mutex<Option<[u8; 16]>>,
}

impl Session {
  pub fn new(user_id: UserId) -> Self {
    Self {
      user_id,
      user_key: Mutex::new(None),
    }
  }

  pub fn rotate_user_key(&self) {
    let mut key = self.user_key.lock().unwrap();
    let key = key.insert(random());
    info!("updated user key: {}", hex::encode(key));
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
    custom.insert("cs".to_owned(), hex::encode(&*digest));
    if let Some(user_key) = &*self.user_key.lock().unwrap() {
      custom.insert("uk".to_owned(), hex::encode(user_key));
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
