use anyhow::Context;
use serde::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

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

pub async fn exchange_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let exchange_master_id: i32 = request.body["exchange_master_id"]
    .parse()
    .context("failed to parse exchange_master_id as i32")?;

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
        == exchange_master_id
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

  Ok((
    CallResponse::new_success(Box::new(ExchangeList {
      exchange_master_id,
      items,
    })),
    true,
  ))
}

// ids=1
pub async fn leave_members(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let ids = request.body["ids"]
    .split(',')
    .filter_map(|id| id.parse::<i32>().ok())
    .collect::<Vec<_>>();

  Ok((CallResponse::new_success(Box::new(())), true))
}
