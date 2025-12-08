//! Hierarchy is Rank (I) -> Area (Near the Axel Village) -> Stage (Dire Bunny Raid)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;

use crate::api::master_all::get_masters;
use crate::api::{battle, ApiRequest};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_FameQuestRankListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestFameRankListResponse {
  quest_rank_id_list: Vec<i32>,
  has_emergency_quest: bool,
}

impl CallCustom for QuestFameRankListResponse {}

pub async fn fame_quest_rank_list() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let ranks: Vec<Value> = serde_json::from_str(&masters["fame_quest_rank"].master_decompressed).unwrap();

  Ok(Unsigned(QuestFameRankListResponse {
    quest_rank_id_list: ranks
      .iter()
      .map(|rank| rank.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap())
      .collect::<Vec<_>>(),
    has_emergency_quest: false,
  }))
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

#[derive(Debug, Deserialize)]
pub struct FameQuestAreaListRequest {
  pub quest_rank_id: i32,
}

pub async fn fame_quest_area_list(Params(params): Params<FameQuestAreaListRequest>) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["fame_quest_area"].master_decompressed).unwrap();

  Ok(Unsigned(QuestFameAreaListResponse {
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
          == params.quest_rank_id
      })
      .map(|rank| FameQuestAreaInfo {
        area_id: rank.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
      })
      .collect::<Vec<_>>(),
    has_emergency_quest: false,
  }))
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
  /// See [Wonder.Api.FameQuestStageInfoResponseDto$$get_IsAllClearReleaseCondition]
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

#[derive(Debug, Deserialize)]
pub struct FameQuestStageListRequest {
  pub area_id: i32,
  pub mode: i32,
}

pub async fn fame_quest_stage_list(Params(params): Params<FameQuestStageListRequest>) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["fame_quest_area"].master_decompressed).unwrap();
  let stages: Vec<Value> = serde_json::from_str(&masters["fame_quest_stage"].master_decompressed).unwrap();

  let area = areas
    .iter()
    .find(|area| area.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .unwrap();
  let current_rank = area
    .get("quest_rank_id")
    .unwrap()
    .as_str()
    .unwrap()
    .parse::<i32>()
    .unwrap();
  debug!("current rank: {}", current_rank);

  Ok(Unsigned(QuestFameStageListResponse {
    quest_list: stages
      .iter()
      .filter(|stage| {
        stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id
          && stage.get("mode").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.mode
      })
      .map(|stage| FameQuestStageInfo {
        stage_id: stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        task1: 0,
        task2: 0,
        task3: 0,
        expired_at: 0,
        release_condition: FameQuestReleaseConditionInfo {
          key_quest: 2,
          story: 2,
          event_story: 2,
        },
        bonus_skill_pa_fame_rate: 0,
      })
      .collect::<Vec<_>>(),
    // All areas for the given rank, thanks https://www.youtube.com/watch?v=Muk190J7LFo
    unlock_area_id_list: areas
      .iter()
      .filter(|area| {
        let area_rank = area
          .get("quest_rank_id")
          .unwrap()
          .as_str()
          .unwrap()
          .parse::<i32>()
          .unwrap();
        area_rank == current_rank
      })
      .map(|area| area.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap())
      .collect::<Vec<_>>(),
    has_emergency_quest: false,
    remaining_count: 1,
    transition_fame_quest_id: 0,
    can_skip: false,
  }))
}

// See [Wonder_Api_FameQuestStartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct FameQuestStart {
  // pub party: FameQuestParty,
  // pub members: Vec<FameQuestMember>,
}

// use_supplement_num=0
// party_no=1
// stage_id=710011
// cost_ratio=1
pub async fn fame_quest_start(request: ApiRequest) -> impl IntoHandlerResponse {
  let use_supplement_num: i32 = request.body["use_supplement_num"].parse().unwrap();
  let party_no: i32 = request.body["party_no"].parse().unwrap();
  let stage_id: i32 = request.body["stage_id"].parse().unwrap();
  let cost_ratio: i32 = request.body["cost_ratio"].parse().unwrap();

  battle::battle_start(request).await
}

// Interestingly it does not have [memcheckcount].
// party_no=1
// win=0
// clear_mission_list=[0,0,0]
// stage_id=710011
pub async fn fame_quest_result(request: ApiRequest) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();
  let win: i32 = request.body["win"].parse().unwrap();
  let clear_mission_list: Vec<i32> = serde_json::from_str(&request.body["clear_mission_list"]).unwrap();
  let stage_id: i32 = request.body["stage_id"].parse().unwrap();

  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "fame_rank_up": 0,
    "money": 0,
    "exp": 0,
    "lvlup": 0,
    "love": [
      {
        "character_id": 100,
        "love": 4
      },
      {
        "character_id": 101,
        "love": 4
      },
      {
        "character_id": 106,
        "love": 4
      }
    ],
    "member_exp": [
      {
        "member_id": 1001100,
        "exp": 150
      },
      {
        "member_id": 1011100,
        "exp": 150
      },
      {
        "member_id": 1064217,
        "exp": 150
      }
    ],
    "mission_reward": [
      {
        "itemtype": 15,
        "itemid": 5001,
        "itemnum": 4,
        "mission": 0,
        "is_rare": 0
      },
      {
        "itemtype": 18,
        "itemid": 1,
        "itemnum": 1,
        "mission": 0,
        "is_rare": 0
      },
      {
        "itemtype": 16,
        "itemid": 151,
        "itemnum": 1,
        "mission": 0,
        "is_rare": 0
      },
      {
        "itemtype": 18,
        "itemid": 2,
        "itemnum": 2,
        "mission": 0,
        "is_rare": 0
      },
      {
        "itemtype": 27,
        "itemid": 230831,
        "itemnum": 3,
        "mission": 0,
        "is_rare": 0
      }
    ],
    "clear_reward": [
      {
        "itemtype": 15,
        "itemid": 1100,
        "itemnum": 3,
        "mission": 1
      },
      {
        "itemtype": 4,
        "itemid": 1061100,
        "itemnum": 1,
        "mission": 1
      },
      {
        "itemtype": 3,
        "itemid": 1,
        "itemnum": 50,
        "mission": 3
      }
    ],
    "lottery_potion_list": [],
  })));

  Ok(Unsigned(response))
}
