use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::api::{NotificationData, RemoteDataItemType};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

// See [Wonder_Api_PresentlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PresentList {
  pub presents: Vec<Present>,
}

impl CallCustom for PresentList {}

// See [Wonder_Api_PresentlistPresentsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Present {
  pub id: i32,
  pub present_id: i32,
  pub senddate: i64,
  pub expireddate: i64,
  pub item_type: i32,
  /// Must be `1` even for singleton items.
  pub item_id: i64,
  pub item_num: i32,
  pub msg: String,
}

#[derive(Debug, Deserialize)]
pub struct PresentListRequest {
  pub start: i32,
  pub end: i32,
}

pub async fn present_list(
  session: Arc<Session>,
  Params(params): Params<PresentListRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.start, ?params.end, "encountered stub: present_list");

  let mut response = CallResponse::new_success(Box::new(PresentList {
    presents: vec![Present {
      id: 28231780,
      present_id: 48,
      senddate: (Utc::now() - chrono::Duration::days(1)).timestamp(),
      expireddate: (Utc::now() + chrono::Duration::days(3)).timestamp(),
      item_type: RemoteDataItemType::RealMoney.into(),
      item_id: 1,
      item_num: 10000,
      msg: "skebob".to_owned(),
    }],
  }));
  response.add_notifications(vec![NotificationData::new(1, 7, 2, 6, "".to_owned(), "".to_owned())]);

  Ok(Signed(response, session))
}

// See [Wonder_Api_PresentloglistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PresentLogList {
  pub presents: Vec<PresentLog>,
}

impl CallCustom for PresentLogList {}

// See [Wonder_Api_PresentloglistPresentsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PresentLog {
  pub id: i32,
  pub present_id: i32,
  pub senddate: i64,
  pub recveddate: i64,
  pub item_type: i32,
  /// Must be `1` even for singleton items.
  pub item_id: i64,
  pub item_num: i32,
  pub msg: String,
}

#[derive(Debug, Deserialize)]
pub struct PresentLogListRequest {
  pub start: i32,
  pub end: i32,
}

pub async fn present_log_list(
  session: Arc<Session>,
  Params(params): Params<PresentLogListRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.start, ?params.end, "encountered stub: present_log_list");

  let response = CallResponse::new_success(Box::new(PresentLogList {
    presents: vec![PresentLog {
      id: 28231780,
      present_id: 48,
      senddate: (Utc::now() - chrono::Duration::days(1)).timestamp(),
      recveddate: (Utc::now() - chrono::Duration::minutes(5)).timestamp(),
      item_type: RemoteDataItemType::RealMoney.into(),
      item_id: 1,
      item_num: 10000,
      msg: "skebob".to_owned(),
    }],
  }));

  Ok(Signed(response, session))
}

// See [Wonder_Api_PresentgetResponseDto_Fields]
#[derive(Debug, Serialize)]
struct PresentGet {
  pub presents: Vec<PresentGetReceived>,
  pub unrecvpresents: Vec<PresentGetUnreceived>,
}

impl CallCustom for PresentGet {}

// See [Wonder_Api_PresentgetPresentsResponseDto_Fields]
#[derive(Debug, Serialize)]
struct PresentGetReceived {
  pub id: i32,
  pub present_id: i32,
  pub senddate: i64,
  pub expireddate: i64,
  pub item_type: i32,
  /// Must be `1` even for singleton items.
  pub item_id: i64,
  pub item_num: i32,
}

// See [Wonder_Api_PresentgetUnrecvpresentsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PresentGetUnreceived {
  pub id: i32,
  pub present_id: i32,
  pub senddate: i64,
  pub expireddate: i64,
  pub item_type: i32,
  /// Must be `1` even for singleton items.
  pub item_id: i64,
  pub item_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct PresentGetRequest {
  #[serde(deserialize_with = "crate::serde_compat::comma_separated_i32")]
  pub ids: Vec<i32>,
}

pub async fn present_get(session: Arc<Session>, Params(params): Params<PresentGetRequest>) -> impl IntoHandlerResponse {
  warn!(?params.ids, "encountered stub: present_get");

  let mut response = CallResponse::new_success(Box::new(PresentGet {
    presents: vec![PresentGetReceived {
      id: 28231780,
      present_id: 48,
      senddate: (Utc::now() - chrono::Duration::days(1)).timestamp(),
      expireddate: (Utc::now() + chrono::Duration::days(7)).timestamp(),
      item_type: RemoteDataItemType::RealMoney.into(),
      item_id: 1,
      item_num: 10000,
    }],
    unrecvpresents: vec![PresentGetUnreceived {
      id: 28231781,
      present_id: 3,
      senddate: (Utc::now() - chrono::Duration::days(1)).timestamp(),
      expireddate: (Utc::now() + chrono::Duration::days(7)).timestamp(),
      item_type: RemoteDataItemType::Stamina.into(),
      item_id: 1,
      item_num: 500,
    }],
  }));
  response.add_notifications(vec![
    NotificationData::new(1, 7, 3, 4, "".to_owned(), "".to_owned()),
    NotificationData::new(1, 7, 13, 7, "".to_owned(), "".to_owned()),
    NotificationData::new(1, 7, 34, 1, "show_button_new".to_owned(), "".to_owned()),
    NotificationData::new(1, 6, 1, 30030001, "".to_owned(), "".to_owned()),
    NotificationData::new(1, 10, 230731, 52307325, "".to_owned(), "".to_owned()),
    NotificationData::new(1, 10, 230831, 52308305, "".to_owned(), "".to_owned()),
  ]);

  Ok(Signed(response, session))
}
