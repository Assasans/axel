use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{io, str};

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
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
use tracing::{debug, info, info_span, trace, warn, Instrument, Span};

use crate::api::{
  account, assist, battle, capture, character, dungeon, exchange, friend, gacha, home, honor_list,
  idlink_confirm_google, interaction, items, login, login_bonus, maintenance_check, master_all, master_list, mission,
  notice, party_info, present, profile, quest_fame, quest_hunting, quest_main, story, transfer, tutorial, ApiRequest,
};
use crate::call::{ApiCallParams, CallCustom, CallMeta, CallResponse};
use crate::client_ip::add_client_ip;
use crate::normalize_path::normalize_path;
use crate::request_logging::log_requests_info;
use crate::user::session::Session;
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
    .layer(log_requests_info())
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
  trace!("hash: {:?}", hash);

  (encrypted, hash)
}

/// These are sent with every request and are completely useless except for `user_key`, `uuid`, and `deviceid`.
pub static HIDDEN_PARAMS: &[&str] = &[
  "countryname",
  "user_key",
  "ver",
  "client_masterversion",
  "deviceid",
  "advertising_id", // Same as "deviceid"
  "npsn",
  "nexonsn",
  "devicename",
  "appver",
  "adid",
  "ostype",
  "osname",
  "version2",
  "nptoken",
  "os",
  "version3",
  "rulever",
  "platform",
  "language",
  "version4",
  "loginPlatform",
  "is_skip_tutorial",
  "userCountry",
  "npaCode",
];

#[axum::debug_handler]
async fn api_call(
  State(state): State<Arc<AppState>>,
  Path(method): Path<String>,
  Query(params): Query<ApiCallParams>,
  headers: HeaderMap,
  body: Bytes,
) -> axum::response::Result<impl IntoResponse, AppError> {
  debug!("api call: {}", method);

  let jwt = headers.get(JWT_HEADER).ok_or_else(|| anyhow!("no jwt header"))?;
  trace!("jwt header: {:?}", jwt);
  let jwt = jwt.to_str().unwrap();
  let [_header, data, _signature] = &jwt.splitn(3, '.').collect::<Vec<_>>()[..] else {
    todo!()
  };
  let data = BASE64_STANDARD_NO_PAD.decode(data).unwrap();
  let meta: CallMeta = serde_json::from_slice(&data).unwrap();
  debug!("api call meta: {:?}", meta);

  let mut session_span: Option<Span> = None;
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

    session_span = Some(info_span!("session", user_id = ?session.user_id));
    Some(session)
  } else {
    None
  };

  let future = async {
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

    if matches!(&*method, "masterall" | "capturesend") {
      info!(?method, body = "(...)", "api call");
    } else {
      info!(?method, body = ?visible_params, "api call");
    }

    let request = ApiRequest {
      params: params.clone(),
      body: body.clone(),

      state: state.clone(),
    };

    let (response, use_user_key): (CallResponse<dyn CallCustom>, _) = match &*method {
      "idlink_confirm_google" => idlink_confirm_google::route(request).await?,
      "masterlist" => master_list::route(request).await?,
      "login" => login::login(state, request, &mut session).await?,
      "capturesend" => capture::capture_send(request).await?,
      "masterall" => master_all::route(request).await?,
      "tutorial" => tutorial::tutorial(state, request, &mut session).await?,
      "notice" => notice::notice(request).await?,
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
      "storyreward" => story::story_reward(request).await?,
      "story_read" => story::story_read(request).await?,
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
      "fame_quest_start" => quest_fame::fame_quest_start(request).await?,
      "fame_quest_result" => quest_fame::fame_quest_result(request).await?,
      "dungeon_status" => dungeon::dungeon_status(request).await?,
      "dungeon_area_top" => dungeon::dungeon_area_top(request).await?,
      "weaponlist" => items::weapon_list(request).await?,
      "accessorylist" => items::accessory_list(request).await?,
      "battlestart" => battle::battle_start(request).await?,
      "battlewaveresult" => battle::battle_wave_result(request).await?,
      "result" => battle::battle_result(request).await?,
      "friendlist" => friend::friend_list(state, request, &mut session).await?,
      "friendinfo" => friend::friend_info(state, request, &mut session).await?,
      "friendmute" => friend::friend_mute(state, request, &mut session).await?,
      "friendremove" => friend::friend_remove(state, request, &mut session).await?,
      "friendrequest" => friend::friend_request(state, request, &mut session).await?,
      "friendsearch" => friend::friend_search(state, request, &mut session).await?,
      "friend_recommendation_list" => friend::friend_recommendation_list(state, request, &mut session).await?,
      "greeting_list" => friend::greeting_list(state, request, &mut session).await?,
      "greeting_send" => friend::greeting_send(state, request, &mut session).await?,
      "assist_make_notice" => assist::assist_make_notice(request).await?,
      "assist_make_list" => assist::assist_make_list(request).await?,
      "exchangelist" => exchange::exchange_list(request).await?,
      "leavemenbers" => exchange::leave_members(request).await?,
      "character_piece_board_info" => character::character_piece_board_info(request).await?,
      "character_enhance_info" => character::character_enhance_info(request).await?,
      "idconfirm" => transfer::id_confirm(request).await?,
      "prepare_set_migration" => transfer::prepare_set_migration(request).await?,
      "newidcheck" => transfer::new_id_check(request).await?,
      "newid" => transfer::new_id(request).await?,
      "idlogin" => transfer::id_login(request).await?,
      "presentlist" => present::present_list(request).await?,
      "presentloglist" => present::present_log_list(request).await?,
      "presentget" => present::present_get(request).await?,
      "mission" => mission::mission_list(request).await?,
      "battlequestinfo" => mission::battle_quest_info(request).await?,
      "battlemarathoninfo" => mission::battle_marathon_info(request).await?,
      "marathon_info" => mission::marathon_info(request).await?,
      "marathon_stage_list" => mission::marathon_stage_list(request).await?,
      "marathon_quest_start" => mission::marathon_quest_start(request).await?,
      "marathon_quest_result" => mission::marathon_quest_result(request).await?,
      "marathon_boss_list" => mission::marathon_boss_list(request).await?,
      _ => todo!("api call '{}'", method),
    };

    let response = serde_json::to_string(&response).unwrap();
    if matches!(
      &*method,
      "masterlist" | "masterall" | "login" | "gachainfo" | "gacha_tutorial_reward"
    ) {
      debug!("response: (...)");
    } else {
      debug!("response: {}", response);
    }

    let user_key = if use_user_key {
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
    trace!("response jwt: {}", token);

    Ok(([(JWT_HEADER, token)], encrypted))
  };
  if let Some(session_span) = session_span {
    future.instrument(session_span).await
  } else {
    future.await
  }
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
