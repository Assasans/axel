//! Hierarchy is Rank (I) -> Area (Near the Axel Village) -> Stage (Dire Bunny Raid)

use serde::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

// See [Wonder_Api_FameQuestRankListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestFameRankListResponse {
  quest_rank_id_list: Vec<i32>,
  has_emergency_quest: bool,
}

impl CallCustom for QuestFameRankListResponse {}

pub async fn fame_quest_rank_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let masters = get_masters().await;
  let ranks: Vec<Value> = serde_json::from_str(&masters["fame_quest_rank"].master_decompressed).unwrap();

  Ok((
    CallResponse::new_success(Box::new(QuestFameRankListResponse {
      quest_rank_id_list: ranks
        .iter()
        .map(|rank| rank.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap())
        .collect::<Vec<_>>(),
      has_emergency_quest: false,
    })),
    false,
  ))
}

// See [Wonder_Api_FameQuestAreaListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestFameAreaListResponse {
  area_info_list: Vec<FameQuestAreaInfo>,
  has_emergency_quest: bool,
}

impl CallCustom for QuestFameAreaListResponse {}

// See [Wonder_Api_FameQuestAreaInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct FameQuestAreaInfo {
  pub area_id: i32,
}

pub async fn fame_quest_area_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let quest_rank_id: i32 = request.body["quest_rank_id"].parse().unwrap();

  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["fame_quest_area"].master_decompressed).unwrap();

  Ok((
    CallResponse::new_success(Box::new(QuestFameAreaListResponse {
      area_info_list: areas
        .iter()
        .filter(|area| {
          area
            .get("quest_rank_id")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<i32>()
            .unwrap()
            == quest_rank_id
        })
        .map(|rank| FameQuestAreaInfo {
          area_id: rank.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        })
        .collect::<Vec<_>>(),
      has_emergency_quest: false,
    })),
    false,
  ))
}

// See [Wonder_Api_FameQuestStageListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestFameStageListResponse {
  quest_list: Vec<FameQuestStageInfo>,
  unlock_area_id_list: Vec<i32>,
  has_emergency_quest: bool,
  remaining_count: i32,
  transition_fame_quest_id: i32,
  can_skip: bool,
}

impl CallCustom for QuestFameStageListResponse {}

// See [Wonder_Api_FameQuestStageInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct FameQuestStageInfo {
  pub stage_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub expired_at: i64,
  pub release_condition: FameQuestReleaseConditionInfo,
  pub bonus_skill_pa_fame_rate: i32,
}

// See [Wonder_Api_FameQuestReleaseConditionInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct FameQuestReleaseConditionInfo {
  pub key_quest: i32,
  pub story: i32,
  pub event_story: i32,
}

pub async fn fame_quest_stage_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let area_id: i32 = request.body["area_id"].parse().unwrap();
  let mode: i32 = request.body["mode"].parse().unwrap();

  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["fame_quest_stage"].master_decompressed).unwrap();

  Ok((
    CallResponse::new_success(Box::new(QuestFameStageListResponse {
      quest_list: stages
        .iter()
        .filter(|stage| {
          stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == area_id
            && stage.get("mode").unwrap().as_str().unwrap().parse::<i32>().unwrap() == mode
        })
        .map(|stage| FameQuestStageInfo {
          stage_id: stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
          task1: 0,
          task2: 0,
          task3: 0,
          expired_at: 0,
          release_condition: FameQuestReleaseConditionInfo {
            key_quest: 0,
            story: 0,
            event_story: 0,
          },
          bonus_skill_pa_fame_rate: 0,
        })
        .collect::<Vec<_>>(),
      unlock_area_id_list: vec![area_id],
      has_emergency_quest: false,
      remaining_count: 1,
      transition_fame_quest_id: 0,
      can_skip: false,
    })),
    false,
  ))
}
