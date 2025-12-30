use crate::AppState;
use crate::api::master_all::get_master_manager;
use crate::api::{ApiRequest, RemoteDataItemType};
use crate::blob::IntoRemoteData;
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::user::session::Session;
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
      .map(|item| BlacksmithlistItemsResponseDto {
        item_id: item["id"].as_str().unwrap().parse().unwrap(),
        item_type: RemoteDataItemType::Weapon.into(),
        unlock: 1,
        newflag: 0,
      })
      .collect(),
    equipped_weapon_list: vec![],
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

  // TODO: Does not consume items, and does not validate anything.
  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;
  let update = UpdateItemCountBy::new(&transaction).await?;
  let item = update
    .run(
      session.user_id,
      (RemoteDataItemType::from(params.item_type), params.item_id),
      params.num,
    )
    .await
    .context("failed to execute query")?;
  debug!(?item, "crafted item");

  let mut response = CallResponse::new_success(Box::new(BlacksmithResponse {
    items: vec![BlacksmithItem {
      unique_id: 123,
      item_type: item.item.item_type.into(),
      item_id: item.item.item_id,
      item_num: item.quantity,
    }],
    returned_items: vec![BlacksmithReturnedItem {
      item_type: item.item.item_type.into(),
      item_id: item.item.item_id,
      item_num: item.quantity,
    }],
  }));
  response.remote.extend(item.into_remote_data());

  transaction.commit().await.context("failed to commit transaction")?;

  Ok(Signed(response, session))
}
