use serde::Serialize;

use crate::api::master_all::get_master_manager;
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_WeaponlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct WeaponList {
  pub items: Vec<WeaponListItem>,
}

impl CallCustom for WeaponList {}

// See [Wonder_Api_WeaponlistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct WeaponListItem {
  /// Opaque
  pub id: i64,
  /// `equip_weapon_details.item_id_details`
  pub weapon_id: i64,
  #[serde(with = "crate::bool_as_int")]
  pub islock: bool,
  pub trial: bool,
}

pub async fn weapon_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let equip_weapons = get_master_manager().get_master("equip_weapon_details");

  Ok(Unsigned(WeaponList {
    items: equip_weapons.iter().map(|item| WeaponListItem {
      id: item["item_id_details"].as_str().unwrap().parse().unwrap(),
      weapon_id: item["item_id_details"].as_str().unwrap().parse().unwrap(),
      islock: false,
      trial: false,
    }).collect(),
  }))
}

// See [Wonder_Api_AccessorylistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AccessoryList {
  pub items: Vec<AccessoryListItem>,
}

impl CallCustom for AccessoryList {}

// See [Wonder_Api_AccessorylistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AccessoryListItem {
  /// Opaque
  pub id: i64,
  /// `equip_accessory_details.item_id_details`
  pub accessory_id: i64,
  #[serde(with = "crate::bool_as_int")]
  pub islock: bool,
}

pub async fn accessory_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let equip_accessories = get_master_manager().get_master("equip_accessory_details");

  Ok(Unsigned(AccessoryList {
    items: equip_accessories.iter().map(|item| AccessoryListItem {
      id: item["item_id_details"].as_str().unwrap().parse().unwrap(),
      accessory_id: item["item_id_details"].as_str().unwrap().parse().unwrap(),
      islock: false,
    }).collect(),
  }))
}
