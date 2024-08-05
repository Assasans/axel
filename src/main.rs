use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::str;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use const_decoder::Decoder;
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use md5::Digest;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub static AES_KEY: &[u8] = &Decoder::Base64.decode::<16>(b"0x9AHqGo1sHGl/nIvD+MhA==");
pub static AES_IV: &[u8] = &Decoder::Base64.decode::<16>(b"Ng84GF0J4+ahev99Wk/qMg==");

pub static USER_KEY: &[u8] = &Decoder::Hex.decode::<16>(b"5a4da47debb24ae9e6895575dfdf9291");

pub static JWT_HEADER: &str = "X-Application-Header";

#[derive(Debug, Serialize, Deserialize)]
struct CallMeta {
  cs: String,
  #[serde(default)]
  uk: Option<String>,
}

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  info!("May There Be a Blessing on This Wonderful Server");

  // let result = Aes128CbcDec::new(AES_KEY.into(), AES_IV.into())
  //   .decrypt_padded_vec_mut::<Pkcs7>(include_bytes!(
  //     "/run/media/assasans/60088D3818C6DD19/KonoSubaFDRE/masterall"
  //   ))
  //   .unwrap();
  // let result = str::from_utf8(&result).unwrap();
  // println!("{}", result);

  let app = Router::new().route("/api/*method", post(api_call));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:2020").await.unwrap();
  info!("api server started at {:?}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();

  Ok(())
}

fn encrypt(data: &[u8], user_key: Option<&[u8]>) -> (Vec<u8>, Digest) {
  let encrypted =
    Aes128CbcEnc::new(AES_KEY.into(), user_key.unwrap_or(AES_IV).into()).encrypt_padded_vec_mut::<Pkcs7>(data);
  let hash = md5::compute(&encrypted);
  debug!("hash: {:?}", hash);

  (encrypted, hash)
}

async fn api_call(
  Path(method): Path<String>,
  headers: HeaderMap,
  body: Bytes,
) -> axum::response::Result<impl IntoResponse, AppError> {
  debug!("api call: {}", method);

  let jwt = headers.get(JWT_HEADER).ok_or_else(|| anyhow!("no jwt header"))?;
  debug!("jwt header: {:?}", jwt);
  let jwt = jwt.to_str().unwrap();
  let [_header, data, _signature] = &jwt.splitn(3, '.').collect::<Vec<_>>()[..] else {
    todo!()
  };
  let data = BASE64_STANDARD_NO_PAD.decode(data).unwrap();
  let meta: CallMeta = serde_json::from_slice(&data).unwrap();
  debug!("api call meta: {:?}", meta);

  let iv = meta
    .uk
    .as_ref()
    .map(|uk| hex::decode(uk).expect(&format!("failed to parse user key: {:?}", uk)))
    .as_deref()
    .unwrap_or(AES_IV)
    .to_owned();
  let body = Aes128CbcDec::new(AES_KEY.into(), iv.as_slice().into())
    .decrypt_padded_vec_mut::<Pkcs7>(&body)
    .expect("failed to decrypt body");
  let body = str::from_utf8(&body).expect(&format!("failed to convert body to string: {:?}", body));
  debug!("api call body: {}", body);

  let body: HashMap<String, String> = serde_urlencoded::from_str(body).unwrap();
  debug!("api call body: {:?}", body);

  info!(?method, ?meta, ?body, "api call");

  let (response, user_key) = match &*method {
    "idlink_confirm_google" => {
      let response = r#"{"islink":0,"status":0,"time":1722619270,"remotedata":[],"notificationdata":[]}"#;
      (response.as_bytes(), None)
    }
    "masterlist" => {
      let response = include_str!("masterlist.json").trim();
      (response.as_bytes(), None)
    }
    "login" => {
      let response = include_str!("login.json").trim();
      (response.as_bytes(), Some(USER_KEY))
    }
    "capturesend" => {
      let response = r#"{"status":0,"time":1722620389,"remotedata":[],"notificationdata":[]}"#;
      (response.as_bytes(), Some(USER_KEY))
    }
    "masterall" => {
      let response = include_str!("masterall.json").trim();
      (response.as_bytes(), None)
    }
    _ => todo!(),
  };

  let (encrypted, hash) = encrypt(response, user_key);

  let key_pair = RS256KeyPair::from_pem(include_str!("../key.pem")).unwrap();
  let mut custom = BTreeMap::new();
  custom.insert("cs".to_owned(), hex::encode(&*hash));
  if let Some(user_key) = user_key {
    custom.insert("uk".to_owned(), hex::encode(&*user_key));
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

  Ok(([(JWT_HEADER, token)], encrypted))
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
  }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}
