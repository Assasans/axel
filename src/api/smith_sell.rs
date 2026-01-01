use anyhow::Context;
use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::master_all::get_master_manager;
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;
use crate::AppState;

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
  #[serde(with = "crate::bool_as_int")]
  pub islock: bool,
  #[serde(with = "crate::bool_as_int")]
  pub isuse: bool,
  pub trial: bool,
}

#[derive(Debug, Deserialize)]
pub enum SaleListKind {
  /// "Equipment"
  #[serde(rename = "equip")]
  Equipment,

  /// "Crafting Materials"
  #[serde(rename = "material")]
  Material,
}

#[derive(Debug, Deserialize)]
pub struct SaleListRequest {
  #[serde(rename = "type")]
  pub kind: SaleListKind,
}

pub async fn sale_list(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<SaleListRequest>,
) -> impl IntoHandlerResponse {
  let items = get_master_manager().get_master("item");
  let equip_weapons = get_master_manager().get_master("equip_weapon");
  let equip_weapon_details = get_master_manager().get_master("equip_weapon_details");

  let client = state.get_database_client().await?;

  let items = match params.kind {
    SaleListKind::Equipment => {
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

      #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select
            id,
            item_type,
            item_id,
            level,
            is_locked
          from user_items_equipment
          where user_id = $1
        "#)
        .await
        .context("failed to prepare statement")?;
      let rows = client
        .query(&statement, &[&session.user_id])
        .await
        .context("failed to execute query")?;

      rows
        .iter()
        .map(|item| {
          let item_id = item.get::<_, i64>("item_id");
          let level = item.get::<_, i32>("level");
          let item_details_id = *item_to_item_details
            .get(&(item_id, level))
            .expect(&format!("missing item details for item_id={} level={}", item_id, level));

          SaleItem {
            item_type: item.get::<_, i64>("item_type") as i32,
            item_id: item_details_id,
            target_item_id: item.get::<_, i64>("id"),
            item_num: 1,
            islock: item.get::<_, bool>("is_locked"),
            isuse: false,
            trial: false,
          }
        })
        .collect()
    }
    SaleListKind::Material => {
      #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select
            item_type,
            item_id,
            quantity
          from user_items
          where user_id = $1
        "#)
        .await
        .context("failed to prepare statement")?;
      let rows = client
        .query(&statement, &[&session.user_id])
        .await
        .context("failed to execute query")?;

      rows
        .iter()
        .map(|item| SaleItem {
          item_type: item.get::<_, i64>("item_type") as i32,
          item_id: item.get::<_, i64>("item_id"),
          target_item_id: item.get::<_, i64>("item_id"),
          item_num: item.get::<_, i32>("quantity"),
          islock: false,
          isuse: false,
          trial: false,
        })
        .collect()
    }
  };

  Ok(Unsigned(SaleList { items }))
}

// type=material
// items=[{"item_type":15,"target_item_id":0,"use_num":1},{"item_type":15,"target_item_id":0,"use_num":1}]
pub async fn sale(_request: ApiRequest) -> impl IntoHandlerResponse {
  // See [Wonder_Api_SaleResponseDto_Fields]
  Ok(Unsigned(()))
}
