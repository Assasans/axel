use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, str};

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::{MatchedPath, Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use clap::Parser;
use const_decoder::Decoder;
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use md5::Digest;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
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

#[derive(Parser, Debug)]
struct Args {
  #[arg(long, default_value_t = false)]
  proxy: bool,
}

#[derive(Clone)]
struct AppState {
  proxy: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  info!("May There Be a Blessing on This Wonderful Server");

  let args = Args::parse();
  if args.proxy {
    info!("proxy mode is enabled");
  }

  // let result = Aes128CbcDec::new(AES_KEY.into(), AES_IV.into())
  //   .decrypt_padded_vec_mut::<Pkcs7>(include_bytes!(
  //     "/run/media/assasans/60088D3818C6DD19/KonoSubaFDRE/masterall"
  //   ))
  //   .unwrap();
  // let result = str::from_utf8(&result).unwrap();
  // println!("{}", result);

  let state = AppState { proxy: args.proxy };

  let app = Router::new()
    .route("/api/*method", post(api_call))
    .layer(
      TraceLayer::new_for_http()
        // Create our own span for the request and include the matched path. The matched
        // path is useful for figuring out which handler the request was routed to.
        .make_span_with(|req: &Request| {
          let method = req.method();
          let uri = req.uri();

          // axum automatically adds this extension.
          let matched_path = req
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());

          tracing::info_span!("request", %method, %uri, matched_path)
        })
        // By default, `TraceLayer` will log 5xx responses but we're doing our specific
        // logging of errors so disable that
        .on_failure(()),
    )
    .with_state(state);

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
  State(state): State<AppState>,
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
  let encrypted_body = body.clone();
  let body = Aes128CbcDec::new(AES_KEY.into(), iv.as_slice().into())
    .decrypt_padded_vec_mut::<Pkcs7>(&body)
    .expect("failed to decrypt body");
  let body = str::from_utf8(&body).expect(&format!("failed to convert body to string: {:?}", body));
  let body_raw = body;
  debug!("api call body: {}", body);

  let body: HashMap<String, String> = serde_urlencoded::from_str(body).unwrap();
  debug!("api call body: {:?}", body);

  info!(?method, ?meta, ?body, "api call");

  let (response, user_key) = if state.proxy {
    let client = reqwest::Client::new();
    let response = client
      .post(format!("https://web-prd-wonder.sesisoft.com/api/{}", method))
      .header(JWT_HEADER, jwt)
      .header(CONTENT_TYPE, "application/octet-stream")
      .header(
        "User-Agent",
        "UnityPlayer/2021.3.36f1 (UnityWebRequest/1.0, libcurl/8.5.0-DEV)",
      )
      .header("X-Unity-Version", "2021.3.36f1")
      .body(encrypted_body.clone())
      .send()
      .await
      .unwrap();

    let upstream_jwt = response.headers().get(JWT_HEADER).cloned();

    // info!("upstream request: {:?}", encrypted_body);
    let response_data = response.bytes().await.unwrap();
    info!("upstream response: {:?}", response_data);

    let upstream_jwt = upstream_jwt.ok_or_else(|| anyhow!("no upstream jwt header"))?;
    debug!("jwt header: {:?}", upstream_jwt);
    let upstream_jwt = upstream_jwt.to_str().unwrap().to_owned();
    let [_header, data, _signature] = &upstream_jwt.splitn(3, '.').collect::<Vec<_>>()[..] else {
      todo!()
    };
    let data = BASE64_STANDARD_NO_PAD.decode(data).unwrap();
    let upstream_meta: CallMeta = serde_json::from_slice(&data).unwrap();
    debug!("api call upstream meta: {:?}", meta);

    let upstream_uk = upstream_meta
      .uk
      .as_ref()
      .map(|uk| hex::decode(uk).expect(&format!("failed to parse user key: {:?}", uk)));
    let upstream_iv = upstream_uk.as_deref().unwrap_or(AES_IV).to_owned();

    let response = Aes128CbcDec::new(AES_KEY.into(), upstream_iv.as_slice().into())
      .decrypt_padded_vec_mut::<Pkcs7>(&response_data)
      .expect("failed to decrypt body");
    info!("upstream decrypted response: {}", String::from_utf8_lossy(&response));
    let response =
      str::from_utf8(&response).expect(&format!("failed to convert upstream body to string: {:?}", response));

    fs::write(format!("proxied/{}-{method}.http", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()), format!("method: {method}\nrequest jwt: {jwt}\nresponse jwt: {upstream_jwt}\n\n---REQUEST---\n{body_raw}\n\n---RESPONSE---\n{response}")).unwrap();

    // info!("upstream response: {}", response);
    (response.to_owned(), upstream_uk)
  } else {
    match &*method {
      "idlink_confirm_google" => {
        let response = r#"{"islink":0,"status":0,"time":1722619270,"remotedata":[],"notificationdata":[]}"#;
        (response.to_owned(), None)
      }
      "masterlist" => {
        let response = include_str!("masterlist.json").trim();
        (response.to_owned(), None)
      }
      "login" => {
        let response = include_str!("login.json").trim();
        (response.to_owned(), Some(USER_KEY.to_vec()))
      }
      "capturesend" => {
        let response = r#"{"status":0,"time":1722620389,"remotedata":[],"notificationdata":[]}"#;
        (response.to_owned(), Some(USER_KEY.to_vec()))
      }
      "masterall" => {
        let response = include_str!("masterall.json").trim();
        (response.to_owned(), None)
      }
      _ => todo!(),
    }
  };

  let (encrypted, hash) = encrypt(response.as_bytes(), user_key.as_deref());

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
    let error = self.0;
    tracing::error!(%error, "api error");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)).into_response()
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
