use crate::api::master_all::get_master_manager;
use crate::api::RemoteDataItemType;
use crate::blob::{AddEquipment, IntoRemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

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

pub async fn blacksmith_list() -> impl IntoHandlerResponse {
  let equip_weapons = get_master_manager().get_master("equip_weapon");
  let equip_accessories = get_master_manager().get_master("equip_accessory");
  let items = get_master_manager()
    .get_master("item")
    .into_iter()
    .map(|item| {
      let id = item["id"].as_str().unwrap().to_string();
      (id, item)
    })
    .collect::<HashMap<_, _>>();

  let get_item_type = |item_id: i64| -> i32 {
    let item = items
      .get(&item_id.to_string())
      .expect(&format!("item {} not found", item_id));
    item["type"].as_str().unwrap().parse().unwrap()
  };

  Ok(Unsigned(BlacksmithList {
    items: equip_weapons
      .into_iter()
      // .filter(|item| item["item_id"].as_str().unwrap().parse::<i32>().unwrap() == 11110)
      .map(|item| BlacksmithlistItemsResponseDto {
        item_id: item["item_id"].as_str().unwrap().parse().unwrap(),
        item_type: RemoteDataItemType::Weapon.into(),
        unlock: 1,
        newflag: 0,
      })
      .collect(),
    equipped_weapon_list: equip_weapons
      .into_iter()
      .filter(|item| item["item_id"].as_str().unwrap().parse::<i32>().unwrap() == 11110)
      .enumerate()
      .map(|(i, item)| BlacksmithEquippedItemResponseDto {
        unique_id: 1000011110,
        item_id: item["item_id"].as_str().unwrap().parse().unwrap(),
        use_party_num: 0,
        is_challenging_dungeon: true,
      })
      .collect(),
    equipped_accessory_list: vec![],
  }))
}

// See [Wonder_Api_BlacksmithResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithResponse {
  pub items: Vec<BlacksmithItem>,
  pub returned_items: Vec<BlacksmithReturnedItem>,
}

impl CallCustom for BlacksmithResponse {}

// See [Wonder_Api_BlacksmithItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithItem {
  pub unique_id: i64,
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// See [Wonder_Api_BlacksmithReturnedItemsResponseDto_Fields]
// See [Wonder_Api_ItempowerupReturnedItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithReturnedItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// body={"num": "1", "item_id": "12211", "material_equipments": "[]", "item_type": "5"}
#[derive(Debug, Deserialize)]
pub struct BlacksmithRequest {
  pub num: i32,
  pub item_type: i32,
  pub item_id: i64,
  pub material_equipments: Vec<i64>,
}

pub async fn blacksmith(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BlacksmithRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: blacksmith");

  let equip_weapon_details = get_master_manager().get_master("equip_weapon_details");

  // (item_id, level) -> item_id_details
  let item_to_item_details: HashMap<(i64, i32), i64> = equip_weapon_details
    .iter()
    .map(|data| {
      let item_id: i64 = data["item_id"].as_str().unwrap().parse::<i64>().unwrap();
      let level: i32 = data["lv"].as_str().unwrap().parse::<i32>().unwrap();
      let item_id_details: i64 = data["item_id_details"].as_str().unwrap().parse::<i64>().unwrap();
      ((item_id, level), item_id_details)
    })
    .collect::<HashMap<_, _>>();

  // TODO: Does not consume items, and does not validate anything.
  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;
  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      insert into user_items_equipment (user_id, item_type, item_id, level)
      values ($1, $2, $3, 0)
      returning id, item_type, item_id, level, is_locked
    "#)
    .await
    .context("failed to prepare statement")?;
  let row = transaction
    .query_one(
      &statement,
      &[&session.user_id, &(params.item_type as i64), &params.item_id],
    )
    .await
    .context("failed to execute query")?;
  debug!(?row, "created equipment item");

  let id = row.get::<_, i64>("id");
  let item_type = row.get::<_, i64>("item_type") as i32;
  let item_id = row.get::<_, i64>("item_id");
  let level = row.get::<_, i32>("level");
  let is_locked = row.get::<_, bool>("is_locked");
  let item_details_id = *item_to_item_details
    .get(&(item_id, level))
    .expect(&format!("missing item details for item_id={} level={}", item_id, level));

  let mut response = CallResponse::new_success(Box::new(BlacksmithResponse {
    items: vec![BlacksmithItem {
      unique_id: id,
      item_type,
      item_id: item_details_id,
      item_num: 1,
    }],
    returned_items: vec![
      // BlacksmithReturnedItem {
      //   item_type: 5,
      //   item_id: 111101,
      //   item_num: 2,
      // }
    ],
  }));
  response.remote.extend(
    AddEquipment::new(
      RemoteDataItemType::from(item_type),
      item_details_id,
      id as i32,
      is_locked,
    )
      .into_remote_data(),
  );

  transaction.commit().await.context("failed to commit transaction")?;

  Ok(Signed(response, session))
}
