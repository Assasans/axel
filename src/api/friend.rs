use std::sync::Arc;

use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use chrono::Utc;
use indoc::indoc;
use jwt_simple::prelude::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;

#[derive(Serialize, Deserialize)]
struct FriendList {
  pub friend_data: Vec<FriendData>,
  pub friend_count: i64,
  pub greeting_sent_count: i64,
}

impl CallCustom for FriendList {}

// See [Wonder_Api_FriendlistFriendDataResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendData {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  pub user_rank: i32,
  pub last_access_time: i64,
  pub first: bool,
  pub mute: bool,
  /// 0 - none, 1 - has greeted, 2 - has messaged
  pub greeting_status: i32,
  #[serde(with = "crate::string_as_base64")]
  pub profile_comment: String,
  pub honor_id: i64,
}

// page=0
// sort_type=1
// list_number=2
pub async fn friend_list(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let page: i32 = request.body["page"].parse().context("failed to parse page as i32")?;
  // 0 - latest login descending, 1 - latest login ascending
  let sort_type: i32 = request.body["sort_type"]
    .parse()
    .context("failed to parse sort_type as i32")?;
  // 1 - sent requests, 2 - pending approval, 3 - friends
  let list_number: i32 = request.body["list_number"]
    .parse()
    .context("failed to parse list_number as i32")?;

  Ok(Signed(
    FriendList {
      friend_data: vec![FriendData {
        user_no: "-1".to_owned(),
        user_icon: 1083110,
        user_name: "Megumin".to_owned(),
        user_rank: 9,
        last_access_time: (Utc::now() - chrono::Duration::days(1)).timestamp(),
        first: true,
        mute: false,
        greeting_status: 0,
        profile_comment: indoc! {"
          I'm gonna type a whole bunch of random stuff right now so it's gonna make it seem like
          I have lots to talk about regarding this topic. I'll uncover the spoiler tags for certain
          words such as Aqua and people will be confused as to why she was mentioned.
        "}
        .to_owned(),
        honor_id: 62010250,
      }],
      friend_count: 1,
      greeting_sent_count: 0,
    },
    session,
  ))
}

#[derive(Serialize, Deserialize)]
struct GreetingList {
  pub sent_count: i32,
  pub received_count: i32,
  pub greeting_data: Vec<GreetingData>,
  pub greeting_count: i32,
}

impl CallCustom for GreetingList {}

// See [Wonder_Api_GreetingListDataResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct GreetingData {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  #[serde(with = "crate::string_as_base64")]
  pub message: String,
  pub first: bool,
}

pub async fn greeting_list(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  Ok(Signed(
    GreetingList {
      sent_count: 0,
      received_count: 1,
      greeting_data: vec![GreetingData {
        user_no: "-1".to_owned(),
        user_icon: 1083110,
        user_name: "Megumin".to_owned(),
        message: indoc! {"
          Darkness blacker than black and darker than dark, I beseech thee, combine with my deep crimson.
          The time of awakening cometh. Justice, fallen upon the infallible boundary, appear now as an
          intangible distortion! Dance, Dance, Dance! I desire for my torrent of power a destructive force:
          a destructive force without equal! Return all creation to cinders, and come from the abyss!
          This is the mightiest means of attack known to man, the ultimate attack magic! EXPLOSION!!
        "}
        .to_owned(),
        first: true,
      }],
      greeting_count: 1,
    },
    session,
  ))
}

// See [Wonder_Api_FriendRecommendationListResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendRecommendationList {
  pub friend_first_count: i32,
  pub friend_first_rank_total: i32,
  pub friend_count: i32,
  pub request_count: i32,
  pub friend_data: Vec<FriendRecommendation>,
  pub mission_completed: bool,
}

impl CallCustom for FriendRecommendationList {}

#[derive(Serialize, Deserialize)]
struct FriendRecommendation {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  pub user_rank: i32,
  pub first: bool,
}

pub async fn friend_recommendation_list(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  // 1 - recommended first friends, 2 - similarly ranked players
  let kind: i32 = request.body["type"].parse().context("failed to parse type as i32")?;

  Ok(Signed(
    FriendRecommendationList {
      friend_first_count: 0,
      friend_first_rank_total: 0,
      friend_count: 1,
      request_count: 0,
      friend_data: vec![FriendRecommendation {
        user_no: "-1".to_owned(),
        user_icon: 1083110,
        user_name: "Yunyun".to_owned(),
        user_rank: 42,
        first: true,
      }],
      mission_completed: false,
    },
    session,
  ))
}

// See [Wonder_Api_GreetingSendResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct GreetingSend {
  /// Friend Beef amount
  pub item_count: i32,
  pub send_data: Vec<GreetingSendData>,
}

impl CallCustom for GreetingSend {}

// See [Wonder_Api_GreetingSendDataResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct GreetingSendData {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile_comment: String,
  pub first: bool,
}

// user_no=["-1"]
// message=a2lsbMKgeW91cnNlbGY
pub async fn greeting_send(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let user_ids = serde_json::from_str::<Vec<String>>(&request.body["user_no"])
    .context("failed to parse user_no as Vec<String>")?
    .into_iter()
    .map(|id| id.parse::<i64>())
    .collect::<Result<Vec<_>, _>>()
    .context("failed to parse user_no as Vec<i64>")?;

  let message = &request.body["message"];
  let message = BASE64_STANDARD_NO_PAD
    .decode(message)
    .context("failed to decode message from base64")?;

  Ok(Signed(
    GreetingSend {
      item_count: 0,
      send_data: vec![GreetingSendData {
        user_no: "-1".to_owned(),
        user_icon: 1083110,
        user_name: "Megumin".to_owned(),
        profile_comment: "Nah.".to_owned(),
        first: true,
      }],
    },
    session,
  ))
}

// See [Wonder_Api_FriendInfoResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendInfo {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile_comment: String,
  pub honor_id: i64,
  pub display_play_data: Vec<FriendDisplayPlayData>,
  /// >= 100 disables Add Friend button
  pub user_friend_count: i32,
  /// >= 100 disables Add Friend button
  pub user_request_count: i32,
  /// Actual value is invisible to the client, >= 100 disables Add Friend button
  pub target_friend_count: i32,
  /// Actual value is invisible to the client, >= 100 disables Add Friend button
  pub target_request_received_count: i32,
  pub friend_status: FriendStatus,
  /// Shoshinsha mark 🔰
  pub first: bool,
  pub mute: bool,
  /// 0 - none, 1 - has greeted, 2 - has messaged
  pub greeting_status: i32,
  pub greeting_sent_count: i32,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum FriendStatus {
  /// Not sure if this is a defined behavior, but any out of range value will hide friend related buttons.
  Disabled = -1,
  None = 0,
  OutgoingRequest = 1,
  IncomingRequest = 2,
  Friends = 3,
}

impl CallCustom for FriendInfo {}

// See [Wonder_Api_FriendinfoDisplayPlayDataResponseDto_Fields]
// and [Wonder_Api_FriendsearchDisplayPlayDataResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendDisplayPlayData {
  #[serde(rename = "type")]
  pub kind: i32,
  pub value: i64,
}

impl FriendDisplayPlayData {
  pub fn new(kind: i32, value: i64) -> Self {
    Self { kind, value }
  }
}

pub async fn friend_info(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let friend_user_no: i64 = request.body["friend_user_no"]
    .parse()
    .context("failed to parse friend_user_no")?;

  Ok(Signed(
    FriendInfo {
      user_no: "-1".to_string(),
      user_icon: 1083110,
      user_name: "Megumin".to_owned(),
      profile_comment: "█████ a whole bunch of ████ so it will seem like I ████ ██████.".to_owned(),
      honor_id: 62010250,
      display_play_data: vec![
        // "Player rank"
        FriendDisplayPlayData::new(1, 2),
        // "Character gallery characters"
        FriendDisplayPlayData::new(4, 14),
        // "Party power": -1 - N/A, 0 - hide, 1+ - power
        FriendDisplayPlayData::new(2, 200),
        // "Total crowns earned"
        FriendDisplayPlayData::new(3, 3),
        // "Latest login", clamped at 1 month at the client
        FriendDisplayPlayData::new(5, Utc::now().timestamp()),
        // "Arena ranking": -2 - calculating ranking, -1 - unranked, 0 - hide, 1+ - rank
        FriendDisplayPlayData::new(6, 42),
        // "Affinity"
        FriendDisplayPlayData::new(7, 1),
      ],
      user_friend_count: 99,
      user_request_count: 50,
      target_friend_count: 0,
      target_request_received_count: 0,
      friend_status: FriendStatus::Friends,
      first: true,
      mute: false,
      greeting_status: 0,
      greeting_sent_count: 50,
    },
    session,
  ))
}

// Braindead API design, it just toggles mute state instead of having "is_muted" parameter.
pub async fn friend_mute(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let friend_user_no: i64 = request.body["friend_user_no"]
    .parse()
    .context("failed to parse friend_user_no")?;

  Ok(Signed((), session))
}

pub async fn friend_remove(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let friend_user_no: i64 = request.body["friend_user_no"]
    .parse()
    .context("failed to parse friend_user_no")?;

  Ok(Signed((), session))
}

pub async fn friend_request(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let friend_user_no: i64 = request.body["friend_user_no"]
    .parse()
    .context("failed to parse friend_user_no")?;

  Ok(Signed((), session))
}

// See [Wonder_Api_FriendsearchResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendSearch {
  pub user_no: String,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile_comment: String,
  pub honor_id: i64,
  pub display_play_data: Vec<FriendDisplayPlayData>,
  pub user_friend_count: i32,
  pub user_request_count: i32,
  pub target_friend_count: i32,
  pub target_request_received_count: i32,
  pub friend_status: FriendStatus,
  pub first: bool,
  pub mute: bool,
  /// 0 - none, 1 - has greeted, 2 - has messaged
  pub greeting_status: i32,
  pub greeting_sent_count: i32,
}

impl CallCustom for FriendSearch {}

pub async fn friend_search(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let friend_user_no: i64 = request.body["friend_user_no"]
    .parse()
    .context("failed to parse friend_user_no")?;

  Ok(Signed(
    FriendSearch {
      user_no: "-1".to_string(),
      user_icon: 1083110,
      user_name: "Megumin".to_owned(),
      profile_comment: "█████ a whole bunch of ████ so it will seem like I ████ ██████.".to_owned(),
      honor_id: 62010250,
      display_play_data: vec![
        // "Player rank"
        FriendDisplayPlayData::new(1, 2),
        // "Character gallery characters"
        FriendDisplayPlayData::new(4, 14),
        // "Party power": -1 - N/A, 0 - hide, 1+ - power
        FriendDisplayPlayData::new(2, 200),
        // "Total crowns earned"
        FriendDisplayPlayData::new(3, 3),
        // "Latest login", clamped at 1 month at the client
        FriendDisplayPlayData::new(5, Utc::now().timestamp()),
        // "Arena ranking": -2 - calculating ranking, -1 - unranked, 0 - hide, 1+ - rank
        FriendDisplayPlayData::new(6, 42),
        // "Affinity"
        FriendDisplayPlayData::new(7, 1),
      ],
      user_friend_count: 99,
      user_request_count: 50,
      target_friend_count: 0,
      target_request_received_count: 0,
      friend_status: FriendStatus::Friends,
      first: true,
      mute: false,
      greeting_status: 0,
      greeting_sent_count: 50,
    },
    session,
  ))
}
