use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

// See [Wonder_Api_ExchangelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExchangeList {
  pub exchange_master_id: i32,
  pub items: Vec<ExchangeItem>,
}

impl CallCustom for ExchangeList {}

// See [Wonder_Api_ExchangelistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExchangeItem {
  pub exchange_reward_master_id: i32,
  pub limit: i32,
  pub exchange_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeListRequest {
  pub exchange_master_id: i32,
}

pub async fn exchange_list(
  session: Arc<Session>,
  Params(params): Params<ExchangeListRequest>,
) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let items: Vec<Value> = serde_json::from_str(&masters["exchange_item"].master_decompressed).unwrap();
  let items = items
    .iter()
    .filter_map(|item| {
      if item
        .get("exchange_id")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<i32>()
        .unwrap()
        == params.exchange_master_id
      {
        Some(ExchangeItem {
          exchange_reward_master_id: item.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
          limit: 1000,
          exchange_num: 500,
        })
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  Ok(Signed(
    ExchangeList {
      exchange_master_id: params.exchange_master_id,
      items,
    },
    session,
  ))
}

#[derive(Debug, Deserialize)]
pub struct LeaveMembersRequest {
  #[serde(deserialize_with = "crate::serde_compat::comma_separated_i32")]
  pub ids: Vec<i32>,
}

// ids=1
pub async fn leave_members(
  session: Arc<Session>,
  Params(params): Params<LeaveMembersRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params.ids, "encountered stub: leave_members");
  Ok(Signed((), session))
}

// exchange_reward_master_id=10001
// num=1
pub async fn exchange(_request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  warn!("encountered stub: exchange");

  // See [Wonder_Api_ExchangeResponseDto_Fields]
  Ok(Signed((), session))
}
