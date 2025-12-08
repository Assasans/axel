use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use indoc::indoc;
use jwt_simple::prelude::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::{debug, info, warn};

use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;
use crate::AppState;

// See [Wonder_Api_FriendlistResponseDto_Fields]
#[derive(Serialize, Deserialize)]
struct FriendList {
  pub friend_data: Vec<FriendData>,
  /// >= 100 disables Add Friend button
  pub friend_count: i32,
  pub greeting_sent_count: i32,
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
  /// First friend 🔰 (Shoshinsha mark)
  pub first: bool,
  pub mute: bool,
  /// 0 - none, 1 - has greeted, 2 - has messaged
  pub greeting_status: i32,
  #[serde(with = "crate::string_as_base64")]
  pub profile_comment: String,
  pub honor_id: i64,
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(i32)]
pub enum FriendListSortType {
  LatestLoginDescending = 0,
  LatestLoginAscending = 1,
}

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(i32)]
pub enum FriendListKind {
  SentRequests = 1,
  PendingApproval = 2,
  Friends = 3,
}

#[derive(Debug, Deserialize)]
pub struct FriendListRequest {
  pub page: i32,
  pub sort_type: FriendListSortType,
  #[serde(rename = "list_number")]
  pub kind: FriendListKind,
}

// page=0
// sort_type=1
// list_number=2
pub async fn friend_list(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendListRequest>,
) -> impl IntoHandlerResponse {
  let friends = match params.kind {
    FriendListKind::SentRequests => {
      vec![]
    }
    FriendListKind::PendingApproval => {
      vec![]
    }
    FriendListKind::Friends => {
      let client = state.pool.get().await.context("failed to get database connection")?;
      #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select
            users.id,
            users.username,
            users.about_me,
            users.favorite_member,
            users.honor,
            (select max(last_used) from user_devices where user_devices.user_id = users.id) as most_recent_last_used
          from users
          -- [username is not null] should always be true due to [tutorial_progress = 99], but just in case
          where id != $1 and tutorial_progress = 99 and username is not null
          -- Limit to 99 because >=100 values disable friend adding on the client
          limit 99
        "#)
        .await
        .context("failed to prepare statement")?;
      let rows = client
        .query(&statement, &[&session.user_id])
        .await
        .context("failed to execute query")?;
      info!(?rows, "get friend list query executed");
      rows
        .iter()
        .map(|row| {
          let id: i64 = row.get(0);
          let username: String = row.get(1);
          let about_me: Option<String> = row.get(2);
          let favorite_member: i64 = row.get(3);
          let honor: i64 = row.get(4);
          let last_used: Option<DateTime<Utc>> = row.get(5);
          let last_used = last_used.unwrap_or(DateTime::<Utc>::MIN_UTC);

          debug!(
            ?username,
            ?about_me,
            ?favorite_member,
            ?honor,
            ?last_used,
            "fetched user profile data"
          );
          FriendData {
            user_no: id.to_string(),
            user_icon: favorite_member,
            user_name: username,
            user_rank: 1,
            last_access_time: last_used.timestamp(),
            first: true,
            mute: false,
            greeting_status: 0,
            profile_comment: about_me.unwrap_or_default(),
            honor_id: honor,
          }
        })
        .collect::<Vec<_>>()
    }
  };

  Ok(Signed(
    FriendList {
      friend_count: friends.len() as i32,
      friend_data: friends,
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

pub async fn greeting_list(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
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

#[derive(Debug, Clone, Copy, Deserialize_repr)]
#[repr(i32)]
pub enum FriendRecommendationKind {
  RecommendedFirstFriends = 1,
  SimilarlyRankedPlayers = 2,
}

#[derive(Debug, Deserialize)]
pub struct FriendRecommendationRequest {
  #[serde(rename = "type")]
  pub kind: FriendRecommendationKind,
}

pub async fn friend_recommendation_list(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendRecommendationRequest>,
) -> impl IntoHandlerResponse {
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

#[derive(Debug, Deserialize)]
pub struct GreetingSendRequest {
  #[serde(rename = "user_no", deserialize_with = "crate::serde_compat::vec_as_i64")]
  pub user_ids: Vec<i64>,
  #[serde(with = "crate::string_as_base64")]
  pub message: String,
}

// user_no=["-1"]
// message=a2lsbMKgeW91cnNlbGY
pub async fn greeting_send(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<GreetingSendRequest>,
) -> impl IntoHandlerResponse {
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

#[derive(Debug, Deserialize)]
pub struct FriendInfoRequest {
  pub friend_user_no: i64,
}

pub async fn friend_info(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendInfoRequest>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select
        users.id,
        users.username,
        users.about_me,
        users.favorite_member,
        users.honor,
        (select max(last_used) from user_devices where user_devices.user_id = users.id) as most_recent_last_used
      from users
      where id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let rows = client
    .query(&statement, &[&params.friend_user_no])
    .await
    .context("failed to execute query")?;
  info!(?rows, "get friend info query executed");
  let row = rows
    .first()
    .ok_or_else(|| anyhow::anyhow!("no profile found for user {:?}", params.friend_user_no))?;
  let id: i64 = row.get(0);
  let username: String = row.get(1);
  let about_me: Option<String> = row.get(2);
  let favorite_member: i64 = row.get(3);
  let honor: i64 = row.get(4);
  let last_used: Option<DateTime<Utc>> = row.get(5);
  let last_used = last_used.unwrap_or(DateTime::<Utc>::MIN_UTC);

  Ok(Signed(
    FriendInfo {
      user_no: id.to_string(),
      user_icon: favorite_member,
      user_name: username,
      profile_comment: about_me.unwrap_or_default(),
      honor_id: honor,
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
        FriendDisplayPlayData::new(5, last_used.timestamp()),
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

#[derive(Debug, Deserialize)]
pub struct FriendMuteRequest {
  pub friend_user_no: i64,
}

// Braindead API design, it just toggles mute state instead of having "is_muted" parameter.
pub async fn friend_mute(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendMuteRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.friend_user_no, "encountered stub: friend_mute");

  // See [Wonder_Api_FriendmuteResponseDto_Fields]
  Ok(Signed((), session))
}

#[derive(Debug, Deserialize)]
pub struct FriendRemoveRequest {
  pub friend_user_no: i64,
}

pub async fn friend_remove(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendRemoveRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.friend_user_no, "encountered stub: friend_remove");

  // See [Wonder_Api_FriendremoveResponseDto_Fields]
  Ok(Signed((), session))
}

#[derive(Debug, Deserialize)]
pub struct FriendRequestRequest {
  pub friend_user_no: i64,
}

pub async fn friend_request(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<FriendRequestRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.friend_user_no, "encountered stub: friend_request");

  // See [Wonder_Api_FriendrequestResponseDto_Fields]
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

#[derive(Debug, Deserialize)]
pub struct FriendSearchRequest {
  pub friend_user_no: i64,
}

pub async fn friend_search(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(request): Params<FriendSearchRequest>,
) -> impl IntoHandlerResponse {
  warn!(?request.friend_user_no, "encountered stub: friend_search");

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
