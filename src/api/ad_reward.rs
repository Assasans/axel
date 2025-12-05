use serde::Serialize;
use serde_json::Value;
use tracing::warn;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_AdvertisementRewardStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AdvertisementRewardStatus {
  pub advertisement_data_list: Vec<AdvertisementData>,
}

impl CallCustom for AdvertisementRewardStatus {}

// See [Wonder_Api_AdvertisementDataResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AdvertisementData {
  pub id: i32,
  pub reward_type: i32,
  pub status: i32,
}

// reward_type_list=[3]
pub async fn advertisement_reward_status(_request: ApiRequest) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let ad_rewards: Vec<Value> = serde_json::from_str(&masters["ad_reward"].master_decompressed).unwrap();

  Ok(Unsigned(AdvertisementRewardStatus {
    advertisement_data_list: ad_rewards
      .iter()
      .map(|ad_reward| AdvertisementData {
        id: ad_reward["id"].as_str().unwrap().parse().unwrap(),
        reward_type: ad_reward["reward_type"].as_str().unwrap().parse().unwrap(),
        status: 0,
      })
      .collect(),
  }))
}

// See [Wonder_Api_ShopitemlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ShopItemList {
  pub items: Vec<ShopItem>,
}

impl CallCustom for ShopItemList {}

// See [Wonder_Api_ShopitemlistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ShopItem {
  pub shop_item_master_id: i32,
  pub interval_time: i32,
  pub buycount: i32,
}

// shop_master_id=4
pub async fn shop_item_list(request: ApiRequest) -> impl IntoHandlerResponse {
  let shop_master_id: i32 = request.body["shop_master_id"].parse().unwrap();

  let masters = get_masters().await;
  let shop_items: Vec<Value> = serde_json::from_str(&masters["shop_item"].master_decompressed).unwrap();

  Ok(Unsigned(ShopItemList {
    items: shop_items
      .iter()
      .filter(|item| item["shop_id"].as_str().unwrap().parse::<i32>().unwrap() == shop_master_id)
      .map(|item| ShopItem {
        shop_item_master_id: item["id"].as_str().unwrap().parse().unwrap(),
        interval_time: 0,
        buycount: 0,
      })
      .collect(),
  }))
}

// See [Wonder_Api_BuyResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BuyResponse {
  pub buymoney: i32,
  pub buyrealmoney: i32,
  pub buyrealmoneyfree: i32,
  pub money: i32,
  pub realmoney: i32,
  pub realmoneyfree: i32,
}

impl CallCustom for BuyResponse {}

// money_type=2
// count=2
// shop_item_master_id=11000
pub async fn buy(_request: ApiRequest) -> impl IntoHandlerResponse {
  warn!("encountered stub: ad_reward::buy");
  Ok(Unsigned(BuyResponse {
    buymoney: 0,
    buyrealmoney: 0,
    buyrealmoneyfree: 0,
    money: 0,
    realmoney: 0,
    realmoneyfree: 0,
  }))
}
