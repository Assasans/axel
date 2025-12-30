use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Context;
use crate::api::master_all::{get_master_manager, get_masters};
use crate::api::quest_fame::FameQuestReleaseConditionInfo;
use crate::api::quest_hunting::{HuntingQuest, extract_items};
use crate::api::smith_craft::BlacksmithEquippedItemResponseDto;
use crate::AppState;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;

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
  pub islock: i32,
  pub trial: bool,
}

pub async fn item_power_up_list(
  state: Arc<AppState>,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
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
  let items = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;

  Ok(Unsigned(PowerUpList {
    items: items
      .iter()
      .map(|item| ItempoweruplistItemsResponseDto {
        item_type: item.get::<_, i64>(0) as i32,
        item_id: item.get::<_, i64>(1),
        target_item_id: 0,
        lv: 0,
        islock: 0,
        trial: false,
      })
      .collect(),
    equipped_weapon_list: vec![],
    equipped_accessory_list: vec![],
  }))
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
        let rewards = extract_items(&rewards[&id]);
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
        let rewards = extract_items(&rewards[&id]);

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
