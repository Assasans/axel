use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;

use crate::api::ApiRequest;
use crate::api::master_all::{get_master_manager, get_masters};
use crate::call::CallCustom;
use crate::extractor::Params;
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

#[derive(Debug, Deserialize)]
pub struct SaleListRequest {
  /// "equip" for Equipment, "material" for Crafting Materials
  #[serde(rename = "type")]
  pub kind: String,
}

pub async fn sale_list(Params(params): Params<SaleListRequest>) -> impl IntoHandlerResponse {
  let items = get_master_manager().get_master("item");
  let equip_weapons = get_master_manager().get_master("equip_weapon");

  warn!(?params, "encountered stub: sale_list");

  // TODO: Client does not display anything when Equipment is selected
  let items = items
    .iter()
    .map(|item| SaleItem {
      item_type: item["type"].as_str().unwrap().parse().unwrap(),
      item_id: item["id"].as_str().unwrap().parse().unwrap(),
      target_item_id: item["id"].as_str().unwrap().parse().unwrap(),
      item_num: 1,
      islock: 0,
      isuse: 0,
      trial: false,
    })
    .chain(equip_weapons.iter().map(|item| SaleItem {
      item_type: 5,
      item_id: item["item_id"].as_str().unwrap().parse().unwrap(),
      target_item_id: item["item_id"].as_str().unwrap().parse().unwrap(),
      item_num: 1,
      islock: 0,
      isuse: 0,
      trial: false,
    }))
    .collect();

  Ok(Unsigned(SaleList { items }))
}

// type=material
// items=[{"item_type":15,"target_item_id":0,"use_num":1},{"item_type":15,"target_item_id":0,"use_num":1}]
pub async fn sale(_request: ApiRequest) -> impl IntoHandlerResponse {
  // See [Wonder_Api_SaleResponseDto_Fields]
  Ok(Unsigned(()))
}
