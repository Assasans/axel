//! Hierarchy is Area (Relic Quest) -> Stage (Eris - Beginner)

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
  pub enablepackage: bool,
  pub status: i32,
}

impl CallCustom for QuestHuntingListResponse {}

// See [Wonder_Api_QuesthuntinglistLimitquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingLimitQuest {
  pub area_id: i32,
  pub status: i32,
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
    limitquests: vec![],
    freequests: areas
      .iter()
      .filter(|area| area.get("type").unwrap().as_str().unwrap() == "FREE")
      .map(|area| {
        let area_id = area.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();

        HuntingFreeQuest { area_id, status: 0 }
      })
      .collect::<Vec<_>>(),
    enablepackage: false,
    status: 0,
  }))
}

// See [Wonder_Api_QuesthuntingstagelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingStageListResponse {
  pub quests: Vec<HuntingStageQuest>,
  pub status: i32,
}

impl CallCustom for QuestHuntingStageListResponse {}

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

  Ok(Unsigned(QuestHuntingStageListResponse {
    quests: stages
      .iter()
      .filter(|stage| stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
      .map(|stage| HuntingStageQuest {
        stage_id: stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        status: 0,
        newstage: 0,
        task1: 0,
        task2: 0,
        task3: 0,
      })
      .collect::<Vec<_>>(),
    status: 0,
  }))
}
