use std::collections::HashMap;
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

fn encrypt(data: &[u8], user_key: Option<&[u8]>) -> Vec<u8> {
  let encrypted =
    Aes128CbcEnc::new(AES_KEY.into(), user_key.unwrap_or(AES_IV).into()).encrypt_padded_vec_mut::<Pkcs7>(data);
  let hash = md5::compute(&encrypted);
  debug!("hash: {:?}", hash);

  encrypted
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

  let (response, user_key, jwt_blob) = match &*method {
    "idlink_confirm_google" => {
      let response = r#"{"islink":0,"status":0,"time":1722619270,"remotedata":[],"notificationdata":[]}"#;
      (response.as_bytes(), None, "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjcyI6ImY0OGY1Njc1MGRiZmQ0ZTAxMjg2M2UyMDBiYzJjMjY0In0.sL2BqIiQcvo_mu6JbiLH2t0zVvIpp1Kcx0sSI9T0PP3oQIwpnd0EgplcwgkbgNAYD4yaTpf2CpVDd7v5j6nTxNCKzQCBNbzEepV-kzDzUooyE1bSxDi7eCcSdVQ6YVYxLuGho9KqrW9xUDgr1XT_07Nts78EDNVNz8xONwcy5jc")
    }
    "masterlist" => {
      let response = include_str!("masterlist.json").trim();
      (response.as_bytes(), None, "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjcyI6IjZlNzE0ZDk5NzRiMzJiZTRlNjNkZTc3NzA4YTIyOWJiIn0.RRCjZkiF-i46cCVcOxo6o8d3kK74yGL6kOpSiZE23SJTLysvykwUb0u66hrSwO9XPAkUnKLcGUsUSKTASBRcQNhTiHrG9F3U_nWIsyTa5DFKWkK_RMjxraTOWPADLBr7LieL04WVq42n9reTcOAReCW6vw1xLGNcGIWRHiCzWYU")
    }
    "login" => {
      let response = include_str!("login.json").trim();
      (response.as_bytes(), Some(USER_KEY), "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjcyI6IjM2ZTNlOTVjMzMyOGRmZDkxNDNkM2ExZGQyZmMwYTAxIiwidWsiOiI1YTRkYTQ3ZGViYjI0YWU5ZTY4OTU1NzVkZmRmOTI5MSJ9.F3sL1szo8mHoU_Tcuw403PeWdopBLsIQ7-6zm0bdq69phJwkhqCZE3EdToPxy581Ja24OP5aNjej1JtO1aI-_fy2dSiiRMuctc4xnUCdZzwsA1oR5l-xKPaLcl8HzZYSkcKo4B_TxVV6KW7-X7Dl05nIO_6s6unkuen921lxM1s")
    }
    "capturesend" => {
      let response = r#"{"status":0,"time":1722620389,"remotedata":[],"notificationdata":[]}"#;
      (response.as_bytes(), Some(USER_KEY), "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjcyI6IjlhY2VhYWQxMDAwMjg4ZjYxYTg1MGU5ZWFlYWY4ZDQ2IiwidWsiOiI1YTRkYTQ3ZGViYjI0YWU5ZTY4OTU1NzVkZmRmOTI5MSJ9.Jw-qyWq3uXWpG3LSh5q1gJZVo3q8QPlRSmjBBgY7Cqb_Q3YU-XcrHkO1kVUsrXxXqPTpe_Nydzm3wVyRbRn-3bFdGcwtkmjlt9d6ebhjFuS9aeQhk1mZmV9qqeXDjXI0kJ3kGwQunqjY8pDvQxpeRdyOeYlFA1nzDLgL0pCMRYg")
    }
    "masterall" => {
      let response = include_str!("masterall.json").trim();
      (response.as_bytes(), None, "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJjcyI6IjFiZTA5YmJiOWZhMTJkMDBhMzMyOWQxM2ZiMGMzNTk1In0.LVfRdKefX_XesFUq_b7QbhkCFXlXB8mtksW6STYK-rX0N4BmZe1lLhIE_dsmYeoKBmKV5Z32tcqr1uxgX8LpD-KSICBuOSnNOwms-NfoC9mwPXnKqunMGYGb9Y72E7hT8s__cxsRYz7uK8ft2zK8aef9dPbCbpkEDInLxjt2uXU")
    }
    _ => todo!(),
  };

  Ok(([(JWT_HEADER, jwt_blob)], encrypt(response, user_key)))
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
