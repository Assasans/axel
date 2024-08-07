pub mod api;
pub mod bool_as_int;
pub mod call;
pub mod session;
pub mod string_as_base64;

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, str};

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::{MatchedPath, Path, Query, Request, State};
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
use serde_json::Value;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::gacha::{GachaGoodItem, GachaInfo, GachaItem, GachaTutorial};
use crate::api::home::{AdvertisementData, Home, MemberInfo};
use crate::api::honor_list::{HonorItem, HonorList};
use crate::api::idlink_confirm_google::IdLinkConfirmGoogle;
use crate::api::login::Login;
use crate::api::login_bonus::{LoginBonus, LoginBonusGood, Omikuji, RandomLoginBonus, RouletteLoginBonus};
use crate::api::maintenance_check::MaintenanceCheck;
use crate::api::master_all::{MasterAll, MASTERS};
use crate::api::master_list::{MasterList, MASTER_LIST};
use crate::api::notice::Notice;
use crate::api::profile::{DisplayPlayData, Profile};
use crate::api::RemoteData;
use crate::call::{ApiCallParams, CallCustom, CallMeta, CallResponse};
use crate::session::{Session, UserId};

pub static AES_KEY: &[u8] = &Decoder::Base64.decode::<16>(b"0x9AHqGo1sHGl/nIvD+MhA==");
pub static AES_IV: [u8; 16] = Decoder::Base64.decode::<16>(b"Ng84GF0J4+ahev99Wk/qMg==");

pub static JWT_HEADER: &str = "X-Application-Header";

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[derive(Parser, Debug)]
struct Args {
  #[arg(long, default_value_t = false)]
  proxy: bool,
}

struct AppState {
  proxy: bool,
  sessions: Mutex<HashMap<UserId, Arc<Session>>>,
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

  let state = AppState {
    proxy: args.proxy,
    sessions: Mutex::new(HashMap::new()),
  };

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
    .with_state(Arc::new(state));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:2020").await.unwrap();
  info!("api server started at {:?}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();

  Ok(())
}

fn encrypt(data: &[u8], user_key: Option<&[u8]>) -> (Vec<u8>, Digest) {
  let encrypted =
    Aes128CbcEnc::new(AES_KEY.into(), user_key.unwrap_or(&AES_IV).into()).encrypt_padded_vec_mut::<Pkcs7>(data);
  let hash = md5::compute(&encrypted);
  debug!("hash: {:?}", hash);

  (encrypted, hash)
}

/// These are sent with every request and are completely useless except for `user_key`.
pub static HIDDEN_PARAMS: &[&str] = &[
  "countryname",
  "user_key",
  "ver",
  "client_masterversion",
  "deviceid",
  "npsn",
  "nexonsn",
  "devicename",
  "appver",
  "adid",
  "ostype",
  "osname",
];

async fn api_call(
  State(state): State<Arc<AppState>>,
  Path(method): Path<String>,
  Query(params): Query<ApiCallParams>,
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

  let session = state
    .sessions
    .lock()
    .unwrap()
    .entry(params.user_id)
    .or_insert_with(|| {
      info!(user_id = ?params.user_id, "create session");
      Arc::new(Session::new(params.user_id))
    })
    .clone();

  let iv = meta
    .uk
    .as_ref()
    .map(|uk| hex::decode(uk).expect(&format!("failed to parse user key: {:?}", uk)))
    .as_deref()
    .unwrap_or(&AES_IV)
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

  let visible_params = body
    .iter()
    .filter(|(key, _)| !HIDDEN_PARAMS.contains(&key.as_str()))
    .collect::<HashMap<_, _>>();
  info!(?method, ?meta, ?params, body = ?visible_params, "api call");

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
    debug!("upstream response: {:?}", response_data);

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
    let upstream_iv = upstream_uk.as_deref().unwrap_or(&AES_IV).to_owned();

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
    let (response, use_user_key): (CallResponse<dyn CallCustom>, _) = match &*method {
      "idlink_confirm_google" => (
        CallResponse::new_success(Box::new(IdLinkConfirmGoogle { islink: false })),
        false,
      ),
      "masterlist" => (
        CallResponse::new_success(Box::new(MasterList {
          masterversion: "202408050001".to_owned(),
          masterarray: MASTER_LIST.clone(),
        })),
        false,
      ),
      "login" => {
        session.rotate_user_key();

        let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(Login {
          user_no: session.user_id.to_string(),
          user_key: hex::encode(session.user_key.lock().unwrap().expect("no user key")),
          user_name: "".to_string(),
          tutorial: 0,
          created_at: "".to_string(),
        }));
        response.add_remote_data(vec![
          RemoteData::new(3, 0, 0, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 1, 0, 80000, 0, 0, "-".to_owned()),
          RemoteData::new(4, 2, 0, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 3, 0, 3000, 0, 0, "-".to_owned()),
          RemoteData::new(4, 9, 0, 10, 0, 0, "-".to_owned()),
          RemoteData::new(4, 10, 0, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 23, 0, 1, 0, 0, "-".to_owned()),
          RemoteData::new(4, 28, 230731, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 34, 2, 3, 0, 0, "-".to_owned()),
          RemoteData::new(4, 34, 1, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 40, 0, 1, 0, 0, "-".to_owned()),
        ]);

        (response, true)
      }
      "capturesend" => (CallResponse::new_success(Box::new(())), true),
      "masterall" => {
        let keys = body["master_keys"].split(",").collect::<Vec<_>>();
        info!("loading masters: {:?}", keys);
        let masters = MASTERS
          .iter()
          .filter(|master| keys.contains(&master.master_key.as_str()))
          .cloned()
          .collect::<Vec<_>>();
        (
          CallResponse::new_success(Box::new(MasterAll {
            masterversion: "202408050001".to_owned(),
            masterarray: masters,
            compressed: true,
          })),
          false,
        )
      }
      "tutorial" => (CallResponse::new_success(Box::new(())), false),
      "notice" => (
        CallResponse::new_custom(
          1,
          Box::new(Notice {
            answer_alarm: "fail".to_owned(),
          }),
        ),
        false,
      ),
      "gachainfo" => (
        CallResponse::new_success(Box::new(GachaInfo {
          gacha: vec![
            GachaItem::new_simple(100001),
            GachaItem::new_simple(100002),
            GachaItem::new_simple(100010),
            GachaItem::new_simple(200001),
            GachaItem::new_simple(200011),
            GachaItem::new_simple(200021),
            GachaItem::new_simple(323083),
            GachaItem::new_simple(410211),
            GachaItem::new_simple(500007),
            GachaItem::new_simple(410248),
            GachaItem::new_simple(410305),
            GachaItem::new_simple(410317),
            GachaItem::new_simple(410321),
            GachaItem::new_simple(410326),
            GachaItem::new_simple(410353),
            GachaItem::new_simple(410364),
            GachaItem::new_simple(410393),
            GachaItem::new_simple(410395),
            GachaItem::new_simple(410402),
            GachaItem::new_simple(410403),
            GachaItem::new_simple(410410),
            GachaItem::new_simple(410430),
            GachaItem::new_simple(410433),
            GachaItem::new_simple(410436),
            GachaItem::new_simple(410437),
            GachaItem::new_simple(410441),
            GachaItem::new_simple(410458),
            GachaItem::new_simple(410486),
            GachaItem::new_simple(410490),
            GachaItem::new_simple(410509),
            GachaItem::new_simple(410522),
            GachaItem::new_simple(410531),
            GachaItem::new_simple(410535),
            GachaItem::new_simple(410536),
            GachaItem::new_simple(410544),
            GachaItem::new_simple(410546),
            GachaItem::new_simple(410548),
            GachaItem::new_simple(410550),
            GachaItem::new_simple(410552),
            GachaItem::new_simple(410553),
            GachaItem::new_simple(410554),
            GachaItem::new_simple(410627),
            GachaItem::new_simple(410639),
            GachaItem::new_simple(410653),
            GachaItem::new_simple(410661),
            GachaItem::new_simple(410670),
          ],
        })),
        false,
      ),
      "gacha_tutorial" => {
        if body["type"] == "1" {
          (
            CallResponse::new_success(Box::new(GachaTutorial {
              gacha_id: 100002,
              goods: vec![
                GachaGoodItem::new(4, 1032102, 1, true),
                GachaGoodItem::new(4, 1692100, 1, true),
                GachaGoodItem::new(4, 1182100, 1, true),
                GachaGoodItem::new(4, 1092100, 1, true),
                GachaGoodItem::new(4, 1024126, 1, true),
                GachaGoodItem::new(4, 1092100, 1, true),
                GachaGoodItem::new(4, 1002100, 1, true),
                GachaGoodItem::new(4, 1052102, 1, true),
                GachaGoodItem::new(4, 1083100, 1, true),
                GachaGoodItem::new(4, 1174130, 1, true),
              ],
            })),
            false,
          )
        } else {
          (
            CallResponse::new_success(Box::new(GachaTutorial {
              gacha_id: 100002,
              goods: vec![],
            })),
            false,
          )
        }
      }
      "gacha_tutorial_reward" => {
        let response = include_str!("gacha-tutorial-reward.json");
        let response: Value = serde_json::from_str(response).unwrap();
        (CallResponse::new_success(Box::new(response)), false)
      }
      "root_box_check" => (CallResponse::new_success(Box::new(())), true),
      "maintenancecheck" => (
        CallResponse::new_success(Box::new(MaintenanceCheck {
          typestatus: 0,
          system_id: None,
        })),
        true,
      ),
      "setname" => (CallResponse::new_success(Box::new(())), true),
      "loginbonus" => (
        CallResponse::new_success(Box::new(LoginBonus {
          goods: vec![
            LoginBonusGood::new(20001, 1, 3, 1, 1000),
            LoginBonusGood::new(40266, 1, 21, 17, 1),
            LoginBonusGood::new(40293, 1, 21, 17, 1),
            LoginBonusGood::new(40294, 1, 21, 17, 1),
            LoginBonusGood::new(80029, 1, 8, 1, 800),
          ],
          omikuji: Omikuji {
            omikuji_id: 0,
            fortune_id: 0,
          },
          random_login_bonus: RandomLoginBonus {
            random_loginbonus_id: 0,
            lot_id: 0,
            story_id: 0,
            user_story_id: 0,
            days: vec![],
          },
          roulette_login_bonus: RouletteLoginBonus {
            roulette_loginbonus_id: 0,
            result_pattern_id: 0,
            roulette_view_id: 0,
            days: vec![],
            sns_share_results: vec![],
          },
        })),
        true,
      ),
      "home" => (
        CallResponse::new_success(Box::new(Home {
          multi_battle_invitation: None,
          member_info: MemberInfo {
            current_member_id: 1011100,
            member_ids: vec![1011100, 0, 0, 0, 0],
          },
          advertisement_data: AdvertisementData {
            id: 10006,
            reward_type: 1,
            status: 0,
          },
          display_plan_map: false,
        })),
        true,
      ),
      "profile" => (
        CallResponse::new_success(Box::new(Profile {
          name: "Aqua".to_string(),
          profile: "Wahhh! Kazuma, he! Kazuma, he wahhh!".to_string(),
          icon: 0,
          honor_id: 62010250,
          display_play_data: vec![
            DisplayPlayData::new(1, 2, 1),
            DisplayPlayData::new(4, 14, 1),
            DisplayPlayData::new(2, -1, 1),
            DisplayPlayData::new(3, 3, 1),
            DisplayPlayData::new(5, 1722883930, 1),
            DisplayPlayData::new(6, -2, 1),
            DisplayPlayData::new(7, 1, 1),
          ],
        })),
        true,
      ),
      "honor_list" => (
        CallResponse::new_success(Box::new(HonorList {
          honor_list: vec![
            HonorItem::new(60000000, false, false),
            HonorItem::new(62010250, true, false),
          ],
        })),
        true,
      ),
      _ => todo!(),
    };

    let response = serde_json::to_string(&response).unwrap();
    info!("response: {}", response);

    (
      response,
      if use_user_key {
        Some(session.user_key.lock().unwrap().expect("no user key").to_vec())
      } else {
        None
      },
    )
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
