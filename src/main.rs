pub mod api;
pub mod bool_as_int;
pub mod call;
pub mod master;
mod normalize_path;
pub mod session;
pub mod static_server;
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
use axum::{Router, ServiceExt};
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use clap::Parser;
use const_decoder::Decoder;
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use md5::Digest;
use rand::random;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use tokio::join;
use tower::Layer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::gacha::{
  BonusInfo, BonusItem, GachaChain, GachaChainGood, GachaGoodItem, GachaInfo, GachaItem, GachaTutorial,
};
use crate::api::home::{AdvertisementData, Home, MemberInfo};
use crate::api::honor_list::{HonorItem, HonorList};
use crate::api::idlink_confirm_google::IdLinkConfirmGoogle;
use crate::api::interaction::{Character, Interaction};
use crate::api::login::Login;
use crate::api::login_bonus::{LoginBonus, Omikuji, RandomLoginBonus, RouletteLoginBonus};
use crate::api::maintenance_check::MaintenanceCheck;
use crate::api::master_all::{get_masters, MasterAll};
use crate::api::master_list::{MasterList, MasterListItem};
use crate::api::notice::Notice;
use crate::api::profile::{DisplayPlayData, Profile};
use crate::api::story_reward::StoryReward;
use crate::api::RemoteData;
use crate::call::{ApiCallParams, CallCustom, CallMeta, CallResponse};
use crate::normalize_path::normalize_path;
use crate::session::{Session, UserId};

pub static AES_KEY: &[u8] = &Decoder::Base64.decode::<16>(b"0x9AHqGo1sHGl/nIvD+MhA==");
pub static AES_IV: [u8; 16] = Decoder::Base64.decode::<16>(b"Ng84GF0J4+ahev99Wk/qMg==");

pub static JWT_HEADER: &str = "X-Application-Header";

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[derive(Parser, Debug)]
struct Args {
  /// Publicly accessible URL of the API server, must support HTTPS. (e.g. "api.yourdomain.dev/")
  #[arg(long)]
  api: String,

  /// Enable proxy mode - save all requests and responses to `proxied/` directory.
  /// Disables API endpoints.
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

  info!("api server public url: {}", args.api);
  if args.proxy {
    info!("proxy mode is enabled");
  }

  // initialize lazies
  get_masters().await;

  info!("{}", serde_json::to_string(&GachaItem::new_simple(410211)).unwrap());

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
  let middleware = tower::util::MapRequestLayer::new(normalize_path);
  let app = middleware.layer(app);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:2020").await.unwrap();
  info!("api server started at {:?}", listener.local_addr().unwrap());

  let (static_result, _) = join!(static_server::start(), axum::serve(listener, app.into_make_service()));
  static_result.unwrap();

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

  let mut session = params
    .user_id
    .map(|user_id| state.sessions.lock().unwrap().get(&user_id).cloned())
    .flatten();

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
  } else
  /*if method == "gachainfo" {
    let response = json!({
      "gacha":[
        {"gachaid":100001,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":6,"ticket_num":10,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        {"gachaid":410321,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":24,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        {"gachaid":323083,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":8,"ticket_num":30,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":200021,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":17,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":323083,"daily":1,"type1":"","val1":0,"type10":"","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":1,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410211,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":19,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":500007,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":1,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410248,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":20,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410305,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":21,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410317,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":23,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410321,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":24,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410326,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":25,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410353,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":26,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410364,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":27,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410393,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":28,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410395,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":29,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":{"gacha_item_id":41039501,"items":[{"pack_id":241039501,"rate":100},{"pack_id":241039502,"rate":1000},{"pack_id":241039503,"rate":5000},{"pack_id":241039504,"rate":7900},{"pack_id":241039505,"rate":8000},{"pack_id":241039506,"rate":10000},{"pack_id":241039507,"rate":10000},{"pack_id":241039508,"rate":10000},{"pack_id":241039509,"rate":16000},{"pack_id":241039510,"rate":16000},{"pack_id":241039511,"rate":16000}]},"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410402,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":30,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410403,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":31,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410410,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":32,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410430,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":33,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410433,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":1,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":{"select_member_id_list":[]},"first_free_ids":[]},
        // {"gachaid":410436,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":1,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":{"select_character_id":0,"select_character_id_list":[100,101,102,103,104,105,106,107,109,110,111,112,113,114,115,116,117,118,119,108,151,128,169]},"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410437,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":34,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410441,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":35,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410458,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":36,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410486,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":37,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410490,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":38,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410509,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":39,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410522,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":40,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410531,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":41,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410535,"daily":0,"type1":"","val1":0,"type10":"limit","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":5,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410536,"daily":0,"type1":"","val1":0,"type10":"limit","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":1,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410544,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":42,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410546,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":43,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410548,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":44,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410550,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":45,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410552,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":46,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410553,"daily":0,"type1":"","val1":0,"type10":"limit","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":3,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410554,"daily":0,"type1":"step","val1":0,"type10":"","val10":0,"ticket":0,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":{"step":0,"loop":0,"is_drawable":true},"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410627,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":53,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410639,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":54,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410653,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":56,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410661,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":57,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]},
        // {"gachaid":410670,"daily":0,"type1":"","val1":0,"type10":"","val10":0,"ticket":58,"ticket_num":0,"draw_count":0,"remain_draw_count":0,"upperlimitcount":0,"user_story_id":0,"stepup_bonus":null,"random_bonus":null,"stepup_info":null,"select_info":null,"continuation_info":null,"member_select_info":null,"first_free_ids":[]}
      ],
      "status":0,
      "time":1723059410,
      "remotedata":[],
      "notificationdata":[
        // {"cmd":1,"type":12,"key":19,"value":200012,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410535,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410536,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410553,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410123,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410436,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410565,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410433,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410564,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},
        // {"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},
        // {"cmd":1,"type":26,"key":200012,"value":1,"msgkey":"","tag":""},
        // {"cmd":1,"type":26,"key":323083,"value":1,"msgkey":"","tag":""},
        // {"cmd":1,"type":26,"key":410436,"value":1,"msgkey":"","tag":""},
        // {"cmd":1,"type":26,"key":410536,"value":1,"msgkey":"","tag":""},
        // {"cmd":1,"type":26,"key":410554,"value":1,"msgkey":"","tag":""},
        // {"cmd":1,"type":27,"key":410535,"value":0,"msgkey":"","tag":""},
        // {"cmd":1,"type":27,"key":410554,"value":0,"msgkey":"","tag":""}
      ]
    });
    let response = serde_json::to_string(&response).unwrap();
    (response.to_owned(), None)
  } else*/
  {
    let (response, use_user_key): (CallResponse<dyn CallCustom>, _) = match &*method {
      "idlink_confirm_google" => (
        CallResponse::new_success(Box::new(IdLinkConfirmGoogle { islink: false })),
        false,
      ),
      "masterlist" => (
        CallResponse::new_success(Box::new(MasterList {
          masterversion: "202408050001".to_owned(),
          masterarray: get_masters()
            .await
            .iter()
            .map(|(_, master)| {
              MasterListItem::new(
                master.master_key.clone(),
                master.master.len() as u32,
                master.checkkey.clone(),
              )
            })
            .collect(),
        })),
        false,
      ),
      "login" => {
        info!(user_id = ?params.user_id, "create session");
        session = Some(if let Some(user_id) = params.user_id {
          // Existing user
          // TODO: Load from database...
          Arc::new(Session::new(user_id))
        } else {
          // New user
          let user_id = UserId::new(random::<u32>() as u64);
          Arc::new(Session::new(user_id))
        });
        let session = session.as_ref().unwrap();

        session.rotate_user_key();
        state.sessions.lock().unwrap().insert(session.user_id, session.clone());

        let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(Login {
          user_no: session.user_id.to_string(),
          user_key: hex::encode(session.user_key.lock().unwrap().expect("no user key")),
          user_name: "".to_string(),
          tutorial: 99,
          created_at: "".to_string(),
        }));
        response.add_remote_data(vec![
          RemoteData::new(3, 0, 0, 0, 0, 0, "-".to_owned()),
          RemoteData::new(4, 1, 0, 80000, 0, 0, "-".to_owned()),
          RemoteData::new(4, 2, 0, 6000, 0, 0, "-".to_owned()),
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
        let masters = get_masters().await;
        let masters = keys
          .iter()
          .map(|key| masters.get(*key).expect(&format!("master {:?} not found", key)))
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
      "gachainfo" => {
        let master = &get_masters().await["gacha"].master_decompressed;
        let master: Vec<master::gacha::Gacha> = serde_json::from_str(master).unwrap();
        let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaInfo {
          gacha: master
            .iter()
            .map(|gacha| GachaItem::new_simple(gacha.gacha_id.parse().unwrap()))
            .collect(), // gacha: vec![
                        //   GachaItem::new_simple(323083, 0),
                        // ]
        }));
        response.add_notifications(vec![
          // NotificationData::new(1, 12, 19, 200012, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410535, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410536, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410553, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410123, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410436, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410565, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410433, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410564, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 26, 200012, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 26, 323083, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 26, 410436, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 26, 410536, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 26, 410554, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 27, 410535, 1, "".to_owned(), "".to_owned()),
          // NotificationData::new(1, 27, 410554, 0, "".to_owned(), "".to_owned()),
        ]);
        (response, false)
      }
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
      "gachachain" => {
        let gacha_id: u32 = body["gacha_id"].parse().unwrap();
        let money_type: u8 = body["money_type"].parse().unwrap();

        let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaChain {
          gacha_id,
          goods: vec![
            GachaChainGood::new(4, 1032102, 1, true),
            GachaChainGood::new(4, 1012100, 1, true),
            GachaChainGood::new(4, 1013100, 1, true),
            GachaChainGood::new(4, 1012100, 1, true),
            GachaChainGood::new(4, 1013116, 1, true),
            GachaChainGood::new(4, 1012102, 1, true),
            GachaChainGood::new(4, 1012102, 1, true),
            GachaChainGood::new(4, 1012102, 1, true),
            GachaChainGood::new(4, 1012100, 1, true),
            GachaChainGood::new(4, 1012102, 1, true),
            GachaChainGood::new(4, 1013116, 1, true),
          ],
          bonus_info: BonusInfo {
            items: vec![BonusItem {
              item_type: 49,
              item_id: 4,
              item_num: 1,
            }],
            rare: 0,
            bonus_type: 1,
            bonus_animation: "".to_owned(),
          },
          bonus_step: None,
        }));
        response.add_remote_data(vec![
          RemoteData::new(1, 7, 2, 11, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 3, 3, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 13, 7, 0, 0, "".to_string()),
          RemoteData::new(1, 7, 34, 2, 0, 0, "show_button".to_string()),
          RemoteData::new(1, 6, 1, 30030001, 0, 0, "".to_string()),
          RemoteData::new(1, 10, 230731, 52307325, 0, 0, "".to_string()),
          RemoteData::new(1, 10, 230831, 52308305, 0, 0, "".to_string()),
        ]);

        (response, false)
      }
      "root_box_check" => (CallResponse::new_success(Box::new(())), false),
      "maintenancecheck" => (
        CallResponse::new_success(Box::new(MaintenanceCheck {
          typestatus: 0,
          system_id: None,
        })),
        false,
      ),
      "firebasetoken" => (CallResponse::new_success(Box::new(())), true),
      "setname" => (CallResponse::new_success(Box::new(())), true),
      "storyreward" => (
        CallResponse::new_success(Box::new(StoryReward { reward: vec![] })),
        true,
      ),
      "loginbonus" => (
        CallResponse::new_success(Box::new(LoginBonus {
          goods: vec![
            // LoginBonusGood::new(20001, 1, 3, 1, 1000),
            // LoginBonusGood::new(40266, 1, 21, 17, 1),
            // LoginBonusGood::new(40293, 1, 21, 17, 1),
            // LoginBonusGood::new(40294, 1, 21, 17, 1),
            // LoginBonusGood::new(80029, 1, 8, 1, 800),
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
      "interaction" => (
        CallResponse::new_success(Box::new(Interaction {
          characters: vec![
            Character::new(100, 1, 4, "".to_owned(), 1000101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(101, 1, 4, "".to_owned(), 1010101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(102, 1, 0, "".to_owned(), 1020101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(103, 1, 0, "".to_owned(), 1030101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(106, 1, 4, "".to_owned(), 1060101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(108, 1, 0, "".to_owned(), 1080101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(109, 1, 0, "".to_owned(), 1090101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(112, 1, 0, "".to_owned(), 1120101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(113, 1, 0, "".to_owned(), 1130101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(115, 1, 0, "".to_owned(), 1150101, 0, [0, 0, 0, 0], "".to_owned()),
            Character::new(128, 1, 0, "".to_owned(), 1280101, 0, [0, 0, 0, 0], "".to_owned()),
          ],
        })),
        true,
      ),
      "partyinfo" => (
        // CallResponse::new_success(Box::new(PartyInfo {
        //   party: vec![
        //   ],
        //   members: vec![],
        //   weapons: vec![],
        //   accessories: vec![],
        // })),
        CallResponse::new_success(Box::new(
          json!({"party":[{"party_forms":[{"id":666431194,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":1,"name":"Party1"},{"id":666431194,"form_no":2,"main":12,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":477,"specialskill":{"special_skill_id":101001,"trial":false},"skill_pa_fame":0,"party_no":1,"name":"Party1"},{"id":666431194,"form_no":3,"main":10,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":487,"specialskill":{"special_skill_id":106001,"trial":false},"skill_pa_fame":0,"party_no":1,"name":"Party1"},{"id":666431194,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":1,"name":"Party1"},{"id":666431194,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":1,"name":"Party1"}],"party_no":1,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":2,"name":"Party2"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":2,"name":"Party2"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":2,"name":"Party2"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":2,"name":"Party2"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":2,"name":"Party2"}],"party_no":2,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":3,"name":"Party3"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":3,"name":"Party3"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":3,"name":"Party3"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":3,"name":"Party3"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":3,"name":"Party3"}],"party_no":3,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":4,"name":"Party4"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":4,"name":"Party4"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":4,"name":"Party4"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":4,"name":"Party4"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":4,"name":"Party4"}],"party_no":4,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":5,"name":"Party5"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":5,"name":"Party5"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":5,"name":"Party5"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":5,"name":"Party5"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":5,"name":"Party5"}],"party_no":5,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":6,"name":"Party6"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":6,"name":"Party6"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":6,"name":"Party6"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":6,"name":"Party6"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":6,"name":"Party6"}],"party_no":6,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":7,"name":"Party7"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":7,"name":"Party7"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":7,"name":"Party7"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":7,"name":"Party7"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":7,"name":"Party7"}],"party_no":7,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":8,"name":"Party8"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":8,"name":"Party8"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":8,"name":"Party8"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":8,"name":"Party8"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":8,"name":"Party8"}],"party_no":8,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":9,"name":"Party9"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":9,"name":"Party9"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":9,"name":"Party9"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":9,"name":"Party9"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":9,"name":"Party9"}],"party_no":9,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":10,"name":"Party10"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":10,"name":"Party10"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":10,"name":"Party10"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":10,"name":"Party10"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":10,"name":"Party10"}],"party_no":10,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":11,"name":"Party11"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":11,"name":"Party11"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":11,"name":"Party11"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":11,"name":"Party11"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":11,"name":"Party11"}],"party_no":11,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":12,"name":"Party12"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":12,"name":"Party12"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":12,"name":"Party12"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":12,"name":"Party12"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":12,"name":"Party12"}],"party_no":12,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":13,"name":"Party13"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":13,"name":"Party13"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":13,"name":"Party13"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":13,"name":"Party13"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":13,"name":"Party13"}],"party_no":13,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":14,"name":"Party14"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":14,"name":"Party14"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":14,"name":"Party14"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":14,"name":"Party14"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":14,"name":"Party14"}],"party_no":14,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":15,"name":"Party15"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":15,"name":"Party15"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":15,"name":"Party15"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":15,"name":"Party15"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":15,"name":"Party15"}],"party_no":15,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":16,"name":"Party16"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":16,"name":"Party16"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":16,"name":"Party16"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":16,"name":"Party16"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":16,"name":"Party16"}],"party_no":16,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":17,"name":"Party17"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":17,"name":"Party17"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":17,"name":"Party17"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":17,"name":"Party17"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":17,"name":"Party17"}],"party_no":17,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":18,"name":"Party18"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":18,"name":"Party18"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":18,"name":"Party18"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":18,"name":"Party18"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":18,"name":"Party18"}],"party_no":18,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":19,"name":"Party19"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":19,"name":"Party19"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":19,"name":"Party19"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":19,"name":"Party19"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":19,"name":"Party19"}],"party_no":19,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}},{"party_forms":[{"id":0,"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":444,"specialskill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0,"party_no":20,"name":"Party20"},{"id":0,"form_no":2,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":20,"name":"Party20"},{"id":0,"form_no":3,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":20,"name":"Party20"},{"id":0,"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":20,"name":"Party20"},{"id":0,"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"strength":0,"specialskill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0,"party_no":20,"name":"Party20"}],"party_no":20,"assist":0,"sub_assists":[],"party_passive_skill":{"skill_id":0,"user_member_id":0}}],"members":[{"id":11,"lv":4,"exp":150,"member_id":1001100,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":0,"ac_skill_lv_c":1,"ac_skill_val_c":130,"hp":277,"attack":32,"magicattack":31,"defense":24,"magicdefence":22,"agility":72,"dexterity":78,"luck":88,"limit_break":0,"character_id":100,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":8,"lv":1,"exp":0,"member_id":1002102,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":20,"ac_skill_lv_c":1,"ac_skill_val_c":130,"hp":257,"attack":28,"magicattack":28,"defense":21,"magicdefence":20,"agility":73,"dexterity":79,"luck":87,"limit_break":0,"character_id":100,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":12,"lv":4,"exp":150,"member_id":1011100,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":170,"ac_skill_lv_c":1,"ac_skill_val_c":152,"hp":285,"attack":33,"magicattack":37,"defense":25,"magicdefence":27,"agility":66,"dexterity":76,"luck":10,"limit_break":0,"character_id":101,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":13,"lv":1,"exp":0,"member_id":1021100,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":20,"ac_skill_lv_c":1,"ac_skill_val_c":152,"hp":202,"attack":26,"magicattack":30,"defense":18,"magicdefence":21,"agility":68,"dexterity":71,"luck":72,"limit_break":0,"character_id":102,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":14,"lv":1,"exp":0,"member_id":1031100,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":127,"ac_skill_lv_c":1,"ac_skill_val_c":150,"hp":281,"attack":29,"magicattack":24,"defense":24,"magicdefence":24,"agility":68,"dexterity":10,"luck":64,"limit_break":0,"character_id":103,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":2,"lv":1,"exp":0,"member_id":1034100,"ac_skill_lv_a":1,"ac_skill_val_a":102,"ac_skill_lv_b":1,"ac_skill_val_b":130,"ac_skill_lv_c":1,"ac_skill_val_c":150,"hp":330,"attack":35,"magicattack":29,"defense":28,"magicdefence":28,"agility":68,"dexterity":10,"luck":64,"limit_break":0,"character_id":103,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":15,"lv":1,"exp":0,"member_id":1061100,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":154,"ac_skill_lv_c":1,"ac_skill_val_c":122,"hp":214,"attack":25,"magicattack":30,"defense":19,"magicdefence":22,"agility":69,"dexterity":68,"luck":67,"limit_break":0,"character_id":106,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":1,"lv":1,"exp":0,"member_id":1063113,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":122,"ac_skill_lv_c":1,"ac_skill_val_c":138,"hp":237,"attack":28,"magicattack":34,"defense":21,"magicdefence":25,"agility":70,"dexterity":69,"luck":66,"limit_break":0,"character_id":106,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":10,"lv":3,"exp":150,"member_id":1064217,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":128,"ac_skill_lv_c":1,"ac_skill_val_c":173,"hp":270,"attack":33,"magicattack":41,"defense":25,"magicdefence":29,"agility":69,"dexterity":67,"luck":68,"limit_break":0,"character_id":106,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":4,"lv":1,"exp":0,"member_id":1083110,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":165,"ac_skill_lv_c":1,"ac_skill_val_c":165,"hp":292,"attack":29,"magicattack":34,"defense":25,"magicdefence":25,"agility":61,"dexterity":66,"luck":63,"limit_break":0,"character_id":108,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":6,"lv":1,"exp":0,"member_id":1093100,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":170,"ac_skill_lv_c":1,"ac_skill_val_c":105,"hp":266,"attack":30,"magicattack":32,"defense":22,"magicdefence":24,"agility":68,"dexterity":67,"luck":65,"limit_break":0,"character_id":109,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":5,"lv":1,"exp":0,"member_id":1122100,"ac_skill_lv_a":1,"ac_skill_val_a":110,"ac_skill_lv_b":1,"ac_skill_val_b":139,"ac_skill_lv_c":1,"ac_skill_val_c":128,"hp":282,"attack":32,"magicattack":24,"defense":23,"magicdefence":19,"agility":71,"dexterity":70,"luck":62,"limit_break":0,"character_id":112,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":7,"lv":1,"exp":0,"member_id":1132100,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":154,"ac_skill_lv_c":1,"ac_skill_val_c":122,"hp":247,"attack":25,"magicattack":31,"defense":19,"magicdefence":22,"agility":69,"dexterity":73,"luck":73,"limit_break":0,"character_id":113,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":3,"lv":1,"exp":0,"member_id":1152102,"ac_skill_lv_a":1,"ac_skill_val_a":100,"ac_skill_lv_b":1,"ac_skill_val_b":170,"ac_skill_lv_c":1,"ac_skill_val_c":152,"hp":247,"attack":27,"magicattack":31,"defense":21,"magicdefence":23,"agility":71,"dexterity":74,"luck":70,"limit_break":0,"character_id":115,"waiting_room":0,"ex_flg":0,"is_undead":0},{"id":9,"lv":1,"exp":0,"member_id":1282100,"ac_skill_lv_a":1,"ac_skill_val_a":93,"ac_skill_lv_b":1,"ac_skill_val_b":128,"ac_skill_lv_c":1,"ac_skill_val_c":122,"hp":239,"attack":25,"magicattack":32,"defense":24,"magicdefence":24,"agility":71,"dexterity":74,"luck":72,"limit_break":0,"character_id":128,"waiting_room":0,"ex_flg":0,"is_undead":0}],"weapons":[],"accessories":[]}),
        )),
        true,
      ),
      _ => todo!(),
    };

    let response = serde_json::to_string(&response).unwrap();
    info!("response: {}", response);

    (
      response,
      if use_user_key {
        Some(
          session
            .expect("no session for user endpoint")
            .user_key
            .lock()
            .unwrap()
            .expect("no user key")
            .to_vec(),
        )
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
