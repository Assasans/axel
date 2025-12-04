use serde::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::quest_fame::FameQuestReleaseConditionInfo;
use crate::api::smith_craft::BlacksmithEquippedItemResponseDto;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

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

pub async fn item_power_up_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let items: Vec<Value> = serde_json::from_str(&masters["item"].master_decompressed).unwrap();

  Ok(Unsigned(CallResponse::new_success(Box::new(PowerUpList {
    items: items
      .iter()
      .map(|item| ItempoweruplistItemsResponseDto {
        item_id: item["id"].as_str().unwrap().parse().unwrap(),
        item_type: item["type"].as_str().unwrap().parse().unwrap(),
        target_item_id: 0,
        lv: 0,
        islock: 0,
        trial: false,
      })
      .collect(),
    equipped_weapon_list: vec![],
    equipped_accessory_list: vec![],
  }))))
}

// See [Wonder_Api_BlacksmithquestlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithQuestList {
  pub quests: Vec<BlacksmithQuest>,
  pub huntingquests: Vec<BlacksmithQuest>,
  pub eventquests: Vec<BlacksmithQuest>,
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

// Shared fields:
// See [Wonder_Api_BlacksmithquestlistQuestsResponseDto_Fields]
// See [Wonder_Api_BlacksmithquestlistHuntingquestsResponseDto_Fields]
// See [Wonder_Api_BlacksmithquestlistEventquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BlacksmithQuest {
  pub quest_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub limit: i32,
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

pub async fn blacksmith_quest_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(CallResponse::new_success(Box::new(BlacksmithQuestList {
    quests: vec![],
    huntingquests: vec![],
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
  }))))
}
