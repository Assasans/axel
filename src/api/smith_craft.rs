use jwt_simple::prelude::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_BlacksmithlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithList {
  pub items: Vec<BlacksmithlistItemsResponseDto>,
  pub equipped_weapon_list: Vec<BlacksmithEquippedItemResponseDto>,
  pub equipped_accessory_list: Vec<BlacksmithEquippedItemResponseDto>,
}

impl CallCustom for BlacksmithList {}

// See [Wonder_Api_BlacksmithlistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithlistItemsResponseDto {
  pub item_id: i64,
  pub item_type: i32,
  pub unlock: i32,
  pub newflag: i32,
}

// See [Wonder_Api_BlacksmithEquippedItemResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithEquippedItemResponseDto {
  pub unique_id: i64,
  pub item_id: i64,
  pub use_party_num: i32,
  pub is_challenging_dungeon: bool,
}

pub async fn blacksmith_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let equip_weapons: Vec<Value> = serde_json::from_str(&masters["equip_weapon"].master_decompressed).unwrap();
  let equip_accessories: Vec<Value> = serde_json::from_str(&masters["equip_accessory"].master_decompressed).unwrap();
  let items: Vec<Value> = serde_json::from_str(&masters["item"].master_decompressed).unwrap();

  let get_item_type = |item_id: i64| -> i32 {
    let item = items
      .iter()
      .find(|item| item["id"].as_str().unwrap().parse::<i64>().unwrap() == item_id)
      .expect(&format!("item {} not found", item_id));
    item["type"].as_str().unwrap().parse().unwrap()
  };

  // let blacksmith_items

  Ok(Unsigned(CallResponse::new_success(Box::new(BlacksmithList {
    items: items
      .iter()
      .map(|item| BlacksmithlistItemsResponseDto {
        item_id: item["id"].as_str().unwrap().parse().unwrap(),
        item_type: item["type"].as_str().unwrap().parse().unwrap(),
        unlock: 1,
        newflag: 0,
      })
      .collect(),
    equipped_weapon_list: vec![],
    equipped_accessory_list: vec![],
  }))))
}

// See [Wonder_Api_BlacksmithResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithResponseDto {
  pub items: Vec<BlacksmithItemsResponseDto>,
  pub returned_items: Vec<BlacksmithReturnedItemsResponseDto>,
}

impl CallCustom for BlacksmithResponseDto {}

// See [Wonder_Api_BlacksmithItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithItemsResponseDto {
  pub unique_id: i64,
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// See [Wonder_Api_BlacksmithReturnedItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithReturnedItemsResponseDto {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// num=1
// item_id=34412
// material_equipments=[]
// item_type=6
pub async fn blacksmith(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(CallResponse::new_success(Box::new(BlacksmithResponseDto {
    items: vec![BlacksmithItemsResponseDto {
      unique_id: 1,
      item_type: 6,
      item_id: 34412,
      item_num: 1,
    }],
    returned_items: vec![],
  }))))
}
