use std::sync::Arc;

use chrono::Utc;
use indoc::indoc;
use jwt_simple::prelude::{Deserialize, Serialize};

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::session::Session;
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

pub async fn friend_list(
  state: Arc<AppState>,
  request: ApiRequest,
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let session = session.as_ref().ok_or_else(|| anyhow::anyhow!("session is not set"))?;

  Ok((
    CallResponse::new_success(Box::new(FriendList {
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
    })),
    true,
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
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let session = session.as_ref().ok_or_else(|| anyhow::anyhow!("session is not set"))?;

  Ok((
    CallResponse::new_success(Box::new(GreetingList {
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
    })),
    true,
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
  #[serde(with = "crate::string_as_base64")]
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
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let session = session.as_ref().ok_or_else(|| anyhow::anyhow!("session is not set"))?;

  Ok((
    CallResponse::new_success(Box::new(FriendRecommendationList {
      friend_first_count: 0,
      friend_first_rank_total: 0,
      friend_count: 1,
      request_count: 0,
      friend_data: vec![FriendRecommendation {
        user_no: "-1".to_owned(),
        user_icon: 62010650,
        user_name: "Yunyun".to_owned(),
        user_rank: 42,
        first: true,
      }],
      mission_completed: false,
    })),
    true,
  ))
}
