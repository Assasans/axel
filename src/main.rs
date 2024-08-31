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
use reqwest::header::CONTENT_TYPE;
use tokio::join;
use tower::Layer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::gacha::GachaItem;
use crate::api::master_all::get_masters;
use crate::api::{
  gacha, home, honor_list, idlink_confirm_google, interaction, login, login_bonus, maintenance_check, master_all,
  master_list, notice, party_info, profile, story_reward, ApiRequest,
};
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
    let request = ApiRequest {
      params: params.clone(),
      body: body.clone(),

      state: state.clone(),
    };

    let (response, use_user_key): (CallResponse<dyn CallCustom>, _) = match &*method {
      "idlink_confirm_google" => idlink_confirm_google::route(request).await?,
      "masterlist" => master_list::route(request).await?,
      "login" => login::route(request, &mut session).await?,
      "capturesend" => (CallResponse::new_success(Box::new(())), true),
      "masterall" => master_all::route(request).await?,
      "tutorial" => (CallResponse::new_success(Box::new(())), false),
      "notice" => notice::route(request).await?,
      "gachainfo" => gacha::gacha_info(request).await?,
      "gacha_tutorial" => gacha::gacha_tutorial(request).await?,
      "gacha_tutorial_reward" => gacha::gacha_tutorial_reward(request).await?,
      "gachachain" => gacha::gacha_chain(request).await?,
      "root_box_check" => (CallResponse::new_success(Box::new(())), false),
      "maintenancecheck" => maintenance_check::route(request).await?,
      "firebasetoken" => (CallResponse::new_success(Box::new(())), true),
      "setname" => (CallResponse::new_success(Box::new(())), true),
      "storyreward" => story_reward::route(request).await?,
      "loginbonus" => login_bonus::route(request).await?,
      "home" => home::route(request).await?,
      "profile" => profile::route(request).await?,
      "honor_list" => honor_list::route(request).await?,
      "interaction" => interaction::route(request).await?,
      "partyinfo" => party_info::route(request).await?,
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
