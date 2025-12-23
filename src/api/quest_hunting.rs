//! Hierarchy is Area (Relic Quest) -> Stage (Eris - Beginner)
//! Reference: https://youtu.be/S9fX6sbXRHw (also shows character upgrade and promotion)

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_QuesthuntinglistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingListResponse {
  pub limitquests: Vec<HuntingLimitQuest>,
  pub freequests: Vec<HuntingFreeQuest>,
  /// Whether "purchase more attempts" button is enabled
  pub enablepackage: bool,
}

impl CallCustom for QuestHuntingListResponse {}

// See [Wonder_Api_QuesthuntinglistLimitquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingLimitQuest {
  pub area_id: i32,
  pub status: i32,
  /// Attempts left. "Challenges {master.limit_count - limit} / {master.limit_count}"
  pub limit: i32,
}

// See [Wonder_Api_QuesthuntinglistFreequestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingFreeQuest {
  pub area_id: i32,
  pub status: i32,
}

pub async fn quest_hunting_list() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["huntingquest_area"].master_decompressed).unwrap();

  Ok(Unsigned(QuestHuntingListResponse {
    limitquests: areas
      .iter()
      .filter(|area| area.get("type").unwrap().as_str().unwrap() == "LIMITED")
      .map(|area| {
        let area_id = area.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();

        HuntingLimitQuest {
          area_id,
          status: 0,
          limit: 1,
        }
      })
      .collect::<Vec<_>>(),
    freequests: areas
      .iter()
      .filter(|area| area.get("type").unwrap().as_str().unwrap() == "FREE")
      .map(|area| {
        let area_id = area.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();

        HuntingFreeQuest { area_id, status: 0 }
      })
      .collect::<Vec<_>>(),
    enablepackage: true,
  }))
}

// See [Wonder_Api_QuesthuntingstagelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingStageListResponse {
  pub quests: Vec<HuntingStageQuest>,
}

impl CallCustom for QuestHuntingStageListResponse {}

// Wonder.Util.ParameterUtil$$GetAttributeNum
// See [Wonder.Util.ParameterUtil$$GetAttributeNum]
// "all" = 8
// "wind" = 3
// "earth" = 2
// "water" = 1
// "0" = 7
// "thunder" = 4
// "light" = 5
// "cursed" = 6
// "fire" = 0
// "unattributed" = 7

// See [Wonder_Api_QuesthuntingstagelistQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingStageQuest {
  pub stage_id: i32,
  pub status: i32,
  pub newstage: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
}

#[derive(Debug, Deserialize)]
pub struct QuestHuntingStageListRequest {
  pub area_id: i32,
}

pub async fn quest_hunting_stage_list(
  Params(params): Params<QuestHuntingStageListRequest>,
) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["huntingquest_stage"].master_decompressed).unwrap();

  // TODO: This should probably return remote data or notification data, but I have no dumps for it.
  //  All stages are locked...
  Ok(Unsigned(QuestHuntingStageListResponse {
    quests: stages
      .iter()
      .filter(|stage| stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
      .map(|stage| HuntingStageQuest {
        stage_id: stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        status: 1,
        newstage: 1,
        task1: 12,
        task2: 13,
        task3: 15,
      })
      .collect::<Vec<_>>(),
  }))
}

// See [Wonder_Api_QuestHuntingLimitStageListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingLimitStageListResponse {
  pub quests: Vec<HuntingLimitStageQuest>,
  pub bonuspack: i32,
}

impl CallCustom for QuestHuntingLimitStageListResponse {}

// See [Wonder_Api_QuestHuntingLimitStageListQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingLimitStageQuest {
  pub stage_id: i32,
  pub challenge_count: i32,
}

pub async fn quest_hunting_limit_stage_list(
) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["huntingquest_stage"].master_decompressed).unwrap();

  Ok(Unsigned(QuestHuntingLimitStageListResponse {
    quests: stages
      .iter()
      .map(|stage| HuntingLimitStageQuest {
        stage_id: stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        challenge_count: 42,
      })
      .collect::<Vec<_>>(),
    bonuspack: 0,
  }))
}
