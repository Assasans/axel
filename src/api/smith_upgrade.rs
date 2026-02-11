use crate::api::master_all::get_master_manager;
use crate::api::quest::parse_reward_items;
use crate::api::quest::quest_fame::FameQuestReleaseConditionInfo;
use crate::api::quest::quest_hunting::HuntingQuest;
use crate::api::smith_craft::{BlacksmithEquippedItemResponseDto, BlacksmithItem, BlacksmithReturnedItem};
use crate::api::RemoteDataItemType;
use crate::blob::{AddEquipment, IntoRemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

// See [Wonder_Api_ItempoweruplistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PowerUpList {
  pub items: Vec<ItempoweruplistItemsResponseDto>,
  pub equipped_weapon_list: Vec<BlacksmithEquippedItemResponseDto>,
  pub equipped_accessory_list: Vec<BlacksmithEquippedItemResponseDto>,
}

impl CallCustom for PowerUpList {}

// See [Wonder_Api_ItempoweruplistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ItempoweruplistItemsResponseDto {
  pub item_type: i32,
  pub item_id: i64,
  pub target_item_id: i32,
  pub lv: i32,
  #[serde(with = "crate::bool_as_int")]
  pub islock: bool,
  pub trial: bool,
}

pub async fn item_power_up_list(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
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
  let items = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;

  Ok(Unsigned(PowerUpList {
    items: items
      .iter()
      .map(|item| ItempoweruplistItemsResponseDto {
        item_type: item.get::<_, i64>("item_type") as i32,
        item_id: item.get::<_, i64>("item_id"),
        target_item_id: item.get::<_, i64>("id") as i32,
        lv: item.get::<_, i32>("level"),
        islock: item.get::<_, bool>("is_locked"),
        trial: false,
      })
      .collect(),
    equipped_weapon_list: vec![],
    equipped_accessory_list: vec![],
  }))
}

// See [Wonder_Api_ItempowerupMaterialEquipmentsRequestDto_Fields]
#[derive(Debug, Deserialize)]
pub struct ItemPowerUpMaterial {
  pub unique_id: i32,
  pub item_type: i32,
}

// See [Wonder_Api_ItempowerupRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct ItemPowerUpRequest {
  pub target_item_id: i64,
  pub item_type: i32,
  pub material_equipments: Vec<ItemPowerUpMaterial>,
  pub powerup_count: i32,
}

// See [Wonder_Api_ItempowerupItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ItemPowerUpItem {
  pub itemtype: i32,
  pub itemid: i32,
  pub itemnum: i32,
}

// See [Wonder_Api_ItempowerupResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ItemPowerUpResponse {
  pub items: Vec<ItemPowerUpItem>,
  pub returned_items: Vec<BlacksmithReturnedItem>,
}

impl CallCustom for ItemPowerUpResponse {}

pub async fn item_power_up(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<ItemPowerUpRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: item_power_up");

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
      update user_items_equipment
      set level = level + $4
      where id = $1 and user_id = $2 and item_type = $3
      returning id, item_type, item_id, level, is_locked
    "#)
    .await
    .context("failed to prepare statement")?;
  let row = transaction
    .query_one(
      &statement,
      &[&params.target_item_id, &session.user_id, &(params.item_type as i64), &params.powerup_count],
    )
    .await
    .context("failed to execute query")?;
  debug!(?row, "upgraded equipment item");

  let id = row.get::<_, i64>("id");
  let item_type = row.get::<_, i64>("item_type") as i32;
  let item_id = row.get::<_, i64>("item_id");
  let level = row.get::<_, i32>("level");
  let is_locked = row.get::<_, bool>("is_locked");
  let item_details_id = *item_to_item_details
    .get(&(item_id, level))
    .expect(&format!("missing item details for item_id={} level={}", item_id, level));
  info!(?id, ?item_type, ?item_id, ?level, ?is_locked, "upgraded item");

  let mut response = CallResponse::new_success(Box::new(ItemPowerUpResponse {
    items: vec![ItemPowerUpItem {
      itemtype: item_type,
      itemid: item_details_id as i32,
      itemnum: 1,
    }],
    returned_items: vec![],
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

  Ok(Unsigned(response))
}

// See [Wonder_Api_BlacksmithquestlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithQuestList {
  pub quests: Vec<BlacksmithQuest>,
  pub huntingquests: Vec<HuntingQuest>,
  pub eventquests: Vec<EventQuest>,
  pub eventboxgachas: Vec<i32>,
  pub exchanges: Vec<i32>,
  pub expedition: i32,
  pub scorechallenge: i32,
  pub scorechallenge_ex: i32,
  pub character_enhance: i32,
  pub dungeon: DungeonAreaMaterialInfoResponseDto,
  pub fame_quest: Vec<FameQuestMaterialInfoResponseDto>,
}

impl CallCustom for BlacksmithQuestList {}

// See [Wonder_Api_BlacksmithquestlistQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithQuest {
  pub quest_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub limit: i32,
}

// See [Wonder_Api_BlacksmithquestlistEventquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct EventQuest {
  pub quest_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub limit: i32,
  pub status: i32,
}

// See [Wonder_Api_DungeonAreaMaterialInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaMaterialInfoResponseDto {
  pub area_ids: Vec<i32>,
  pub unlocked_area_ids: Vec<i32>,
  pub challenging_area_id: i32,
}

// See [Wonder_Api_FameQuestMaterialInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct FameQuestMaterialInfoResponseDto {
  pub stage_id: i32,
  pub is_challengeable: bool,
  pub release_condition: FameQuestReleaseConditionInfo,
  pub remaining_count: i32,
}

// body={"item_id": "12211"}
#[derive(Debug, Deserialize)]
pub struct BlacksmithQuestListRequest {
  pub item_id: i64,
}

pub async fn blacksmith_quest_list(Params(params): Params<BlacksmithQuestListRequest>) -> impl IntoHandlerResponse {
  let main_quests = {
    let stages = get_master_manager().get_master("mainquest_stage");
    let rewards = get_master_manager()
      .get_master("mainquest_stage_itemreward")
      .into_iter()
      .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
      .collect::<HashMap<_, _>>();
    stages
      .into_iter()
      .filter(|stage| {
        let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
        let rewards = parse_reward_items(&rewards[&id]);
        rewards.iter().any(|item| item.item_id == params.item_id)
      })
      .collect::<Vec<_>>()
  };

  let hunting_quests = {
    let stages = get_master_manager().get_master("huntingquest_stage");
    let rewards = get_master_manager()
      .get_master("huntingquest_stage_itemreward")
      .into_iter()
      .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
      .collect::<HashMap<_, _>>();

    stages
      .into_iter()
      .filter(|stage| {
        let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
        let rewards = parse_reward_items(&rewards[&id]);

        rewards.iter().any(|item| item.item_id == params.item_id)
      })
      .collect::<Vec<_>>()
  };

  Ok(Unsigned(BlacksmithQuestList {
    quests: main_quests
      .into_iter()
      .map(|stage| {
        let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
        BlacksmithQuest {
          quest_id: id,
          task1: 1,
          task2: 1,
          task3: 0,
          limit: 42,
        }
      })
      .collect(),
    huntingquests: hunting_quests
      .into_iter()
      .map(|stage| {
        let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
        HuntingQuest {
          quest_id: id,
          task1: 1,
          task2: 1,
          task3: 0,
          limit: 42,
          status: 2,
        }
      })
      .collect(),
    eventquests: vec![],
    eventboxgachas: vec![],
    exchanges: vec![],
    expedition: 0,
    scorechallenge: 0,
    scorechallenge_ex: 0,
    character_enhance: 0,
    dungeon: DungeonAreaMaterialInfoResponseDto {
      area_ids: vec![],
      unlocked_area_ids: vec![],
      challenging_area_id: 0,
    },
    fame_quest: vec![],
  }))
}
