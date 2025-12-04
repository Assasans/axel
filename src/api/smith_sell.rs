use jwt_simple::prelude::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_SaleListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SaleList {
  pub items: Vec<SaleItem>,
}

impl CallCustom for SaleList {}

// See [Wonder_Api_SaleListItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SaleItem {
  pub item_type: i32,
  pub item_id: i64,
  pub target_item_id: i64,
  pub item_num: i32,
  pub islock: i32,
  pub isuse: i32,
  pub trial: bool,
}

pub async fn sale_list(request: ApiRequest) -> impl IntoHandlerResponse {
  // "equip" for Equipment, "material" for Crafting Materials
  let kind = &request.body["type"];

  let masters = get_masters().await;
  let items: Vec<Value> = serde_json::from_str(&masters["item"].master_decompressed).unwrap();
  let equip_weapons: Vec<Value> = serde_json::from_str(&masters["equip_weapon"].master_decompressed).unwrap();

  // TODO: Client does not display anything when Equipment is selected
  let items = items
    .iter()
    .map(|item| SaleItem {
      item_type: item["type"].as_str().unwrap().parse().unwrap(),
      item_id: item["id"].as_str().unwrap().parse().unwrap(),
      target_item_id: 0,
      item_num: 1,
      islock: 0,
      isuse: 0,
      trial: false,
    })
    .collect();

  Ok(Unsigned(CallResponse::new_success(Box::new(SaleList { items }))))
}

// type=material
// items=[{"item_type":15,"target_item_id":0,"use_num":1},{"item_type":15,"target_item_id":0,"use_num":1}]
pub async fn sale(_request: ApiRequest) -> impl IntoHandlerResponse {
  // See [Wonder_Api_SaleResponseDto_Fields]
  Ok(Unsigned(CallResponse::new_success_empty()))
}
