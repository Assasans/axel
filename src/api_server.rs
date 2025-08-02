use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, str};

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::{MatchedPath, Path, Query, Request, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{middleware, Router, ServiceExt};
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use const_decoder::Decoder;
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::claims::JWTClaims;
use md5::Digest;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span, warn};

use crate::api::{
  account, battle, dungeon, friend, gacha, home, honor_list, idlink_confirm_google, interaction, items, login,
  login_bonus, maintenance_check, master_all, master_list, notice, party_info, profile, quest_fame, quest_hunting,
  quest_main, story, story_reward, tutorial, ApiRequest,
};
use crate::call::{ApiCallParams, CallCustom, CallMeta, CallResponse};
use crate::client_ip::{add_client_ip, ClientIp};
use crate::normalize_path::normalize_path;
use crate::session::Session;
use crate::{AppError, AppState};

pub static AES_KEY: &[u8] = &Decoder::Base64.decode::<16>(b"0x9AHqGo1sHGl/nIvD+MhA==");
pub static AES_IV: [u8; 16] = Decoder::Base64.decode::<16>(b"Ng84GF0J4+ahev99Wk/qMg==");

pub static JWT_HEADER: &str = "X-Application-Header";

pub type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
pub type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub async fn start(state: Arc<AppState>) -> io::Result<()> {
  let app = Router::new()
    .route("/", get(get_root_friendly))
    .route("/api/{*method}", post(api_call))
    .layer(
      TraceLayer::new_for_http()
        // Create our own span for the request and include the matched path. The matched
        // path is useful for figuring out which handler the request was routed to.
        .make_span_with(|request: &Request| {
          let method = request.method();
          let uri = request.uri();

          // axum automatically adds this extension.
          let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());
          let client_ip = request
            .extensions()
            .get::<ClientIp>()
            .map(|client_ip| client_ip.0)
            .unwrap();

          info_span!("request", %client_ip, %method, %uri, matched_path)
        })
        // By default, `TraceLayer` will log 5xx responses but we're doing our specific
        // logging of errors so disable that
        .on_failure(()),
    )
    .layer(middleware::from_fn(add_client_ip))
    .with_state(state.clone());
  let middleware = tower::util::MapRequestLayer::new(normalize_path);
  let app = middleware.layer(app);

  let listener = TcpListener::bind(state.settings.api_server.bind_address).await.unwrap();
  info!(
    "api server started at {:?} -> {}",
    listener.local_addr()?,
    state.settings.api_server.public_url
  );
  axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await
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

  let mut session = if let Some(user_key) = &meta.uk {
    let user_key = const_hex::decode(user_key).expect(&format!("failed to parse user key: {:?}", user_key));
    let user_key: [u8; 16] = user_key
      .clone()
      .try_into()
      .expect(&format!("user key is not 16 bytes: {:?}", user_key));

    let mut sessions = state.sessions.lock().unwrap();
    let session = sessions
      .values()
      .find(|session| session.user_key.lock().unwrap().as_ref() == Some(&user_key))
      .cloned();
    let session = if let Some(session) = session {
      debug!("found session for {:?} by user key", session.user_id);
      session
    } else {
      let user_id = params
        .user_id
        .expect("user_id is not set in params, but (invalid) user key is present");
      let session = Arc::new(Session::new(user_id, None));
      session.rotate_user_key();
      warn!("created fake session for {}", user_id);

      sessions.insert(user_id, session.clone());
      session
    };

    if let Some(user_id) = params.user_id {
      if session.user_id != user_id {
        warn!(
          "user_id in params ({}) does not match user_id in session ({})",
          user_id, session.user_id
        );
      }
    } else {
      warn!("user_id is not set in params, but user key is present in meta");
    }

    Some(session)
  } else {
    None
  };

  let iv = meta
    .uk
    .as_ref()
    .map(|uk| const_hex::decode(uk).expect(&format!("failed to parse user key: {:?}", uk)))
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

    if upstream_jwt.is_some() {
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
        .map(|uk| const_hex::decode(uk).expect(&format!("failed to parse user key: {:?}", uk)));
      let upstream_iv = upstream_uk.as_deref().unwrap_or(&AES_IV).to_owned();

      let response = Aes128CbcDec::new(AES_KEY.into(), upstream_iv.as_slice().into())
        .decrypt_padded_vec_mut::<Pkcs7>(&response_data)
        .expect("failed to decrypt body");
      info!("upstream decrypted response: {}", String::from_utf8_lossy(&response));
      let response =
        str::from_utf8(&response).expect(&format!("failed to convert upstream body to string: {:?}", response));

      let response = if method == "login" {
        r#"{"user_no":"359537457194","user_key":"34ce6215e064469a89b9b76dc27af0d6","user_name":"LS0t","tutorial":1,"status":0,"created_at":"2024-08-02 14:39:30","time":1722864558,"remotedata":[{"cmd":3,"item_type":0,"item_id":0,"item_num":0,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":1,"item_id":0,"item_num":80000,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":2,"item_id":0,"item_num":0,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":3,"item_id":0,"item_num":3000,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":9,"item_id":0,"item_num":10,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":10,"item_id":0,"item_num":0,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":23,"item_id":0,"item_num":1,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":28,"item_id":230731,"item_num":0,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":34,"item_id":2,"item_num":3,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":34,"item_id":1,"item_num":0,"uniqid":0,"lv":0,"tag":"-"},{"cmd":4,"item_type":40,"item_id":0,"item_num":1,"uniqid":0,"lv":0,"tag":"-"}],"notificationdata":[{"cmd":1,"type":7,"key":6,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":8,"key":0,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":14,"value":1,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":14,"value":1,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":14,"value":1,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":14,"value":1,"msgkey":"","tag":""},{"cmd":1,"type":6,"key":1,"value":30030001,"msgkey":"","tag":""},{"cmd":1,"type":10,"key":230731,"value":52307325,"msgkey":"","tag":""},{"cmd":1,"type":10,"key":230831,"value":52308305,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":200012,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410535,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410536,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410553,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410123,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410436,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410565,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410433,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410564,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},{"cmd":1,"type":12,"key":19,"value":410554,"msgkey":"","tag":""},{"cmd":1,"type":14,"key":21,"value":11003,"msgkey":"","tag":""},{"cmd":1,"type":14,"key":21,"value":31015,"msgkey":"","tag":""},{"cmd":1,"type":14,"key":21,"value":31016,"msgkey":"","tag":""},{"cmd":1,"type":14,"key":21,"value":31017,"msgkey":"","tag":""},{"cmd":1,"type":16,"key":1,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":15,"key":1,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":23,"key":3,"value":1,"msgkey":"","tag":""},{"cmd":1,"type":25,"key":3,"value":0,"msgkey":"0","tag":""},{"cmd":1,"type":14,"key":4,"value":1722609570,"msgkey":"","tag":""},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"100"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"101"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"102"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"103"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"104"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"105"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"106"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"107"},{"cmd":1,"type":19,"key":5,"value":0,"msgkey":"","tag":"23083"},{"cmd":1,"type":19,"key":5,"value":1722620388,"msgkey":"","tag":"50081"},{"cmd":1,"type":7,"key":23,"value":1,"msgkey":"[\"12209\",\"12206\",\"12204\",\"12203\",\"12205\",\"12207\",\"12208\",\"12210\",\"12211\",\"12212\",\"12320\",\"12900\",\"12100\",\"12200\",\"12300\",\"12310\"]","tag":""},{"cmd":1,"type":7,"key":401,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":4011,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":4012,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":15,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":16,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":18,"value":1722864558,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":3,"value":2,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":13,"value":7,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":11,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":12,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":24,"value":0,"msgkey":"","tag":""},{"cmd":1,"type":7,"key":14,"value":1,"msgkey":"","tag":""}]}"#.to_owned()
      } else {
        response.to_owned()
      };

      fs::write(format!("proxied/{}-{method}.http", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()), format!("method: {method}\nrequest jwt: {jwt}\nresponse jwt: {upstream_jwt}\n\n---REQUEST---\n{body_raw}\n\n---RESPONSE---\n{response}")).unwrap();

      // info!("upstream response: {}", response);
      (response.to_owned(), upstream_uk)
    } else {
      let response = str::from_utf8(&response_data).expect(&format!(
        "failed to convert upstream body to string: {:?}",
        response_data
      ));

      let response = if method == "capturesend" {
        r#"{"status":0,"time":1722865498,"remotedata":[],"notificationdata":[]}"#.to_owned()
      } else {
        response.to_owned()
      };

      let response = if method == "idlink_confirm_google" {
        r#"{"islink":0,"status":0,"time":1722865489,"remotedata":[],"notificationdata":[]}"#.to_owned()
      } else {
        response.to_owned()
      };

      fs::write(format!("proxied/{}-{method}.http", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()), format!("method: {method}\nrequest jwt: {jwt}\nresponse jwt: NONE\n\n---REQUEST---\n{body_raw}\n\n---RESPONSE---\n{response}")).unwrap();

      // info!("upstream response: {}", response);
      (response.to_owned(), None)
    }
  } else {
    let request = ApiRequest {
      params: params.clone(),
      body: body.clone(),

      state: state.clone(),
    };

    let (response, use_user_key): (CallResponse<dyn CallCustom>, _) = match &*method {
      "idlink_confirm_google" => idlink_confirm_google::route(request).await?,
      "masterlist" => master_list::route(request).await?,
      "login" => login::login(state, request, &mut session).await?,
      "capturesend" => (CallResponse::new_success(Box::new(())), true),
      "masterall" => master_all::route(request).await?,
      "tutorial" => tutorial::tutorial(state, request, &mut session).await?,
      "notice" => notice::route(request).await?,
      "gachainfo" => gacha::gacha_info(request).await?,
      "gacha_tutorial" => gacha::gacha_tutorial(request).await?,
      "gacha_tutorial_reward" => gacha::gacha_tutorial_reward(request).await?,
      "gachachain" => gacha::gacha_chain(request).await?,
      "gachanormal" => gacha::gacha_normal(request).await?,
      "gacharate" => gacha::gacha_rate(request).await?,
      "gachalog" => gacha::gacha_log(request).await?,
      "root_box_check" => (CallResponse::new_success(Box::new(())), false),
      "maintenancecheck" => maintenance_check::route(request).await?,
      "firebasetoken" => (CallResponse::new_success(Box::new(())), true),
      "setname" => account::set_name(state, request, &mut session).await?,
      "storyreward" => story_reward::route(request).await?,
      "loginbonus" => login_bonus::route(request).await?,
      "home" => home::route(request).await?,
      "profile" => profile::profile(state, request, &mut session).await?,
      "honor_list" => honor_list::route(request).await?,
      "interaction" => interaction::route(request).await?,
      "partyinfo" => party_info::route(request).await?,
      "storylist" => story::story_list(request).await?,
      "quest_main_part_list" => quest_main::quest_main_part_list(request).await?,
      "quest_main_stage_list" => quest_main::quest_main_stage_list(request).await?,
      "quest_main_area_list" => quest_main::quest_main_area_list(request).await?,
      "questhuntinglist" => quest_hunting::quest_hunting_list(request).await?,
      "questhuntingstagelist" => quest_hunting::quest_hunting_stage_list(request).await?,
      "fame_quest_rank_list" => quest_fame::fame_quest_rank_list(request).await?,
      "fame_quest_area_list" => quest_fame::fame_quest_area_list(request).await?,
      "fame_quest_stage_list" => quest_fame::fame_quest_stage_list(request).await?,
      "dungeon_status" => dungeon::dungeon_status(request).await?,
      "dungeon_area_top" => dungeon::dungeon_area_top(request).await?,
      "weaponlist" => items::weapon_list(request).await?,
      "accessorylist" => items::accessory_list(request).await?,
      "battlestart" => battle::battle_start(request).await?,
      "battlewaveresult" => battle::battle_wave_result(request).await?,
      "result" => battle::result(request).await?,
      "friendlist" => friend::friend_list(state, request, &mut session).await?,
      "greeting_list" => friend::greeting_list(state, request, &mut session).await?,
      "friend_recommendation_list" => friend::friend_recommendation_list(state, request, &mut session).await?,
      _ => todo!("api call '{}'", method),
    };

    let response = serde_json::to_string(&response).unwrap();
    if matches!(&*method, "masterlist" | "masterall" | "login" | "gachainfo") {
      info!("response: (...)");
    } else {
      info!("response: {}", response);
    }

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
  custom.insert("cs".to_owned(), const_hex::encode(&*hash));
  if let Some(user_key) = user_key {
    custom.insert("uk".to_owned(), const_hex::encode(&*user_key));
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

async fn get_root_friendly() -> axum::response::Result<impl IntoResponse, AppError> {
  let name = env!("CARGO_PKG_NAME");
  let version = env!("CARGO_PKG_VERSION");

  Ok(Html(format!(
    "<DOCTYPE html>
    <html>
      <head>
        <title>Axel API server</title>
      </head>
      <body>
        <h1>Welcome to the Axel API server!</h1>
        <p>This server processes all API requests from the game client.</p>
        <hr />
        <i>{name}/{version}</i>
      </body>
    </html>",
  )))
}
