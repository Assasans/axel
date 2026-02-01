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

use crate::api::{ApiRequest, *};
use crate::call::{ApiCallParams, CallMeta};
use crate::client_ip::add_client_ip;
use crate::handler::{HandlerContext, IntoHandlerResponse, Signed, Unsigned};
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

  #[rustfmt::skip]
  let router = crate::router::Router::new()
    .handle("idlink_confirm_google", idlink_confirm_google::idlink_confirm_google)
    .handle("masterlist", master_list::master_list)
    .handle("login", login::login)
    .handle("capturesend", capture::capture_send)
    .handle("masterall", master_all::master_all)
    .handle("tutorial", tutorial::tutorial)
    .handle("notice", notice::notice)
    .handle("gachainfo", gacha::gacha_info)
    .handle("gacha_tutorial", gacha::gacha_tutorial)
    .handle("gacha_tutorial_reward", gacha::gacha_tutorial_reward)
    .handle("gachachain", gacha::gacha_chain)
    .handle("gachanormal", gacha::gacha_normal)
    .handle("gacharate", gacha::gacha_rate)
    .handle("gacharate_assist", gacha::gacha_rate_assist)
    .handle("gacha_assist_log", gacha::gacha_assist_log)
    .handle("gachalog", gacha::gacha_log)
    .handle("root_box_check", root_check_box)
    .handle("maintenancecheck", maintenance_check::maintenance_check)
    .handle("firebasetoken", firebase_token)
    .handle("setname", profile::set_name)
    .handle("delete_account", profile::delete_account)
    .handle("storyreward", story::story_reward)
    .handle("story_read", story::story_read)
    .handle("loginbonus", login_bonus::login_bonus)
    .handle("home", home::home)
    .handle("home_members_set", home::home_members_set)
    .handle("home_current_member_set", home::home_current_member_set)
    .handle("missionhome", home::mission_home)
    .handle("profile", profile::profile)
    .handle("setprofile", profile::set_profile)
    .handle("honor_list", profile::honor_list)
    .handle("honor_set", profile::honor_set)
    .handle("seticon", profile::set_icon)
    .handle("interaction", interaction::interaction)
    .handle("partyinfo", party_info::party_info)
    .handle("storylist", story::story_list)
    .handle("quest_main_part_list", quest_main::quest_main_part_list)
    .handle("quest_main_stage_list", quest_main::quest_main_stage_list)
    .handle("quest_main_area_list", quest_main::quest_main_area_list)
    .handle("questhuntinglist", quest_hunting::quest_hunting_list)
    .handle("questhuntingstagelist", quest_hunting::quest_hunting_stage_list)
    .handle("quest_hunting_limit_stage_list", quest_hunting::quest_hunting_limit_stage_list)
    .handle("battlehuntingstart", quest_hunting::battle_hunting_start)
    .handle("battlehuntingresult", quest_hunting::battle_hunting_result)
    .handle("huntingquest_list_by_item", quest_hunting::hunting_quest_list_by_item)
    .handle("fame_quest_rank_list", quest_fame::fame_quest_rank_list)
    .handle("fame_quest_area_list", quest_fame::fame_quest_area_list)
    .handle("fame_quest_stage_list", quest_fame::fame_quest_stage_list)
    .handle("fame_quest_start", quest_fame::fame_quest_start)
    .handle("fame_quest_result", quest_fame::fame_quest_result)
    .handle("dungeon_status", dungeon::dungeon_status)
    .handle("dungeon_area_top", dungeon::dungeon_area_top)
    .handle("dungeon_area_retire", dungeon::dungeon_area_retire)
    .handle("dungeon_top", dungeon::dungeon_top)
    .handle("dungeon_list", dungeon::dungeon_list)
    .handle("dungeon_team_info", dungeon::dungeon_team_info)
    .handle("dungeon_area_challenge", dungeon::dungeon_area_challenge)
    .handle("dungeon_stage_party_info", dungeon::dungeon_stage_party_info)
    .handle("dungeon_battle_start", dungeon::dungeon_battle_start)
    .handle("dungeon_battle_defeat", dungeon::dungeon_battle_defeat)
    .handle("dungeon_team_offer", dungeon::dungeon_team_offer)
    .handle("dungeon_team_reset", dungeon::dungeon_team_reset)
    .handle("dungeon_area_skip", dungeon::dungeon_area_skip)
    .handle("weaponlist", items::weapon_list)
    .handle("accessorylist", items::accessory_list)
    .handle("battlestart", battle::battle_start)
    .handle("battleretire", battle::battle_retire)
    .handle("battlewaveresult", battle::battle_wave_result)
    .handle("result", battle::battle_result)
    .handle("friendlist", friend::friend_list)
    .handle("friendinfo", friend::friend_info)
    .handle("friendmute", friend::friend_mute)
    .handle("friendremove", friend::friend_remove)
    .handle("friendrequest", friend::friend_request)
    .handle("friendsearch", friend::friend_search)
    .handle("friend_recommendation_list", friend::friend_recommendation_list)
    .handle("greeting_list", friend::greeting_list)
    .handle("greeting_send", friend::greeting_send)
    .handle("assist_make_notice", assist::assist_make_notice)
    .handle("assist_make_list", assist::assist_make_list)
    .handle("exchangelist", exchange::exchange_list)
    .handle("leavemenbers", exchange::leave_members)
    .handle("exchange", exchange::exchange)
    .handle("character_piece_board_info", character::character_piece_board_info)
    .handle("character_enhance_info", character::character_enhance_info)
    .handle("idconfirm", transfer::id_confirm)
    .handle("prepare_set_migration", transfer::prepare_set_migration)
    .handle("newidcheck", transfer::new_id_check)
    .handle("newid", transfer::new_id)
    .handle("idlogin", transfer::id_login)
    .handle("presentlist", present::present_list)
    .handle("presentloglist", present::present_log_list)
    .handle("presentget", present::present_get)
    .handle("mission", mission::mission_list)
    .handle("battlequestinfo", mission::battle_quest_info)
    .handle("battlemarathoninfo", mission::battle_marathon_info)
    .handle("marathon_info", mission::marathon_info)
    .handle("marathon_stage_list", mission::marathon_stage_list)
    .handle("marathon_quest_start", mission::marathon_quest_start)
    .handle("marathon_quest_result", mission::marathon_quest_result)
    .handle("marathon_boss_list", mission::marathon_boss_list)
    .handle("panel_mission_list", panel_mission::panel_mission_list)
    .handle("panel_mission", panel_mission::panel_mission)
    .handle("sale_list", smith_sell::sale_list)
    .handle("sale", smith_sell::sale)
    .handle("blacksmithlist", smith_craft::blacksmith_list)
    .handle("blacksmith", smith_craft::blacksmith)
    .handle("itempoweruplist", smith_upgrade::item_power_up_list)
    .handle("blacksmithquestlist", smith_upgrade::blacksmith_quest_list)
    .handle("partymembers", party::party_members)
    .handle("gradeup", party::grade_up)
    .handle("limitbreak", party::limit_break)
    .handle("memberskillup", party::member_skill_up)
    .handle("update_party_form", party::update_party_form)
    .handle("partyoffer", party::party_offer)
    .handle("partyreset", party::party_reset)
    .handle("partychangelist", party::party_change_list)
    .handle("partychange_assist", party::party_change_assist)
    .handle("partynameset", party::party_name_set)
    .handle("partychange", party::party_change)
    .handle("party_strength", party::party_strength)
    .handle("expeditiontop", expedition::expedition_top)
    .handle("expeditioncharacter", expedition::expedition_character)
    .handle("expeditionset", expedition::expedition_set)
    .handle("advertisement_reward_status", ad_reward::advertisement_reward_status)
    .handle("shopitemlist", ad_reward::shop_item_list)
    .handle("purchase_google_limited_products_status", shop::purchase_google_limited_products_status)
    .handle("purchase_google_charge_status", shop::purchase_google_charge_status)
    .handle("buy", ad_reward::buy)
    .handle("surprise_mini_event_select", surprise::surprise_mini_event_select)
    .handle("surprise_mini_event_top", surprise::surprise_mini_event_top)
    .handle("surprise_quest_start", surprise::surprise_quest_start)
    .handle("surprise_quest_result", surprise::surprise_quest_result)
    .handle("surprise_short_event", surprise::surprise_short_event)
    .handle("surprise_story_start", surprise::surprise_story_start)
    .handle("surprise_story_select", surprise::surprise_story_select)
    .handle("surprise_story_result", surprise::surprise_story_result)
    .handle("scorechallengeinfo", battle_arena::score_challenge_info)
    .handle("scorechallengeranking", battle_arena::score_challenge_ranking)
    .handle("scorechallengebestscoreparty", battle_arena::score_challenge_best_score_party)
    .handle("scorechallengestart", battle_arena::score_challenge_start)
    .handle("scorechallengeresult", battle_arena::score_challenge_result)
    .handle("scorechallenge_mission", battle_arena::score_challenge_mission)
    .handle("scorechallenge_mission_list", battle_arena::score_challenge_mission_list)
    .handle("multi_battle_invitation_list", battle_multi::multi_battle_invitation_list)
    .handle("multi_battle_room_info", battle_multi::multi_battle_room_info)
    .handle("multi_battle_create_room", battle_multi::multi_battle_create_room)
    .handle("multi_battle_search_and_join_room", battle_multi::multi_battle_search_and_join_room)
    .handle("multi_battle_room_status", battle_multi::multi_battle_room_status)
    .handle("marathon_multi_start", battle_multi::marathon_multi_start)
    .handle("marathon_multi_battling", battle_multi::marathon_multi_battling)
    .handle("marathon_multi_result_confirm", battle_multi::marathon_multi_result_confirm)
    .handle("marathon_multi_log", battle_multi::marathon_multi_log)
    .handle("marathon_multi_result", battle_multi::marathon_multi_result)
    .handle("marathon_multi_stamp", battle_multi::marathon_multi_stamp)
    .handle("multi_battle_join_room", battle_multi::multi_battle_join_room)
    .handle("multi_battle_room_leave", battle_multi::multi_battle_room_leave);

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
  let session = if let Some(user_key) = &meta.uk {
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

    let span: Span = info_span!(
      "session",
      user_id = %session.user_id,
      username = tracing::field::Empty
    );
    // We don't want to do database query each time, so use cached username
    if let Some(username) = session.get_cached_username() {
      span.record("username", tracing::field::display(username));
    }
    session_span = Some(span);
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

    let response = router
      .dispatch(&method, HandlerContext {
        state: state.clone(),
        request: Some(request.clone()),
        session: session.as_ref().cloned(),
      })
      .await;
    let response = response.into_handler_response();

    let response_data = serde_json::to_string(&response.response).unwrap();
    if matches!(
      &*method,
      "masterlist" | "masterall" | "login" | "gachainfo" | "gacha_tutorial_reward"
    ) || response_data.len() > 10000
    {
      debug!("response: (...)");
    } else {
      debug!("response: {}", response_data);
    }

    let user_key = if let Some(session) = response.signing_session {
      Some(session.user_key.lock().unwrap().expect("no user key").to_vec())
    } else {
      None
    };

    let (encrypted, hash) = encrypt(response_data.as_bytes(), user_key.as_deref());

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

async fn root_check_box() -> impl IntoHandlerResponse {
  Unsigned(())
}

async fn firebase_token(session: Arc<Session>) -> impl IntoHandlerResponse {
  Signed((), session)
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
