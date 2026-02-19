//! Hierarchy is Part (1) -> Area (Chapter 1) -> Stage (Chapter 1-1)

use crate::api::dungeon::BattleSkipReward;
use crate::api::master_all::get_master_manager;
use crate::api::quest::quest_hunting::BattleHuntingSkipRequest;
use crate::api::{NotificationData, RemoteDataItemType};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

// See [Wonder_Api_QuestMainPartListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestMainPartListResponse {
  pub quests: Vec<QuestMainPartListItem>,
}

impl CallCustom for QuestMainPartListResponse {}

// See [Wonder_Api_QuestMainPartListQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestMainPartListItem {
  pub quest_part_id: i32,
  /// 0 - unlocked (new), 1 - unlocked, 2 - completed, 3 - 100% completed
  pub status: i32,
}

pub async fn quest_main_part_list() -> impl IntoHandlerResponse {
  let parts = get_master_manager().get_master("main_quest_part");
  let parts = parts
    .iter()
    .map(|part| {
      let part_id = part.get("part").unwrap().as_str().unwrap().parse::<i32>().unwrap();
      QuestMainPartListItem {
        quest_part_id: part_id,
        status: 3,
      }
    })
    .collect::<Vec<_>>();

  Ok(Unsigned(QuestMainPartListResponse { quests: parts }))
}

// See [Wonder_Api_QuestMainAreaListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestMainAreaListResponse {
  pub normal_area_list: Vec<QuestMainAreaListItem>,
  pub hard_area_list: Vec<QuestMainAreaListItem>,
  pub expert_area_list: Vec<QuestMainAreaListItem>,
}

impl CallCustom for QuestMainAreaListResponse {}

// See [Wonder_Api_QuestMainAreaResponseDto_Fields]
#[derive(Clone, Debug, Serialize)]
pub struct QuestMainAreaListItem {
  pub quest_area_master_id: i32,
  /// 0 - unlocked (new), 1 - unlocked, 2 - completed, 3 - 100% completed
  pub status: i32,
}

pub async fn quest_main_area_list() -> impl IntoHandlerResponse {
  let areas = get_master_manager().get_master("mainquest_area");
  let areas = areas
    .iter()
    .map(|stage| {
      let stage_id = stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap();
      QuestMainAreaListItem {
        quest_area_master_id: stage_id,
        status: 3,
      }
    })
    .collect::<Vec<_>>();

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(QuestMainAreaListResponse {
    normal_area_list: areas.clone(),
    hard_area_list: areas.clone(),
    expert_area_list: areas,
  }));
  response.add_notifications(vec![NotificationData::new(1, 7, 20, 1, "".to_owned(), "".to_owned())]);

  Ok(Unsigned(response))
}

// See [Wonder_Api_QuestMainStageListRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct QuestMainStageListRequest {
  pub area_id: i32,
}

// See [Wonder_Api_QuestMainStageListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestMainStageListResponse {
  pub quests: Vec<QuestMainStageListItem>,
}

impl CallCustom for QuestMainStageListResponse {}

// See [Wonder_Api_QuestMainStageResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestMainStageListItem {
  pub quest_stage_id: i32,
  pub status: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  /// Number of attempts done. Unlimited for Normal stages, 3 for Hard stages, 1 for Expert stages.
  pub challenge_count: i32,
  pub difficulty: i32,
}

pub async fn quest_main_stage_list(Params(params): Params<QuestMainStageListRequest>) -> impl IntoHandlerResponse {
  let stages = get_master_manager().get_master("mainquest_stage");
  let stages = stages
    .iter()
    .filter(|stage| stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .map(|stage| {
      let stage_id = stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();
      let difficulty = stage.get("mode").unwrap().as_str().unwrap().parse::<i32>().unwrap();
      QuestMainStageListItem {
        quest_stage_id: stage_id,
        status: 3,
        task1: 1,
        task2: 1,
        task3: 1,
        challenge_count: 0,
        difficulty,
      }
    })
    .collect::<Vec<_>>();

  Ok(Unsigned(QuestMainStageListResponse {
    quests: stages
  }))
}

// See [Wonder_Api_BattleskipRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct BattleSkipRequest {
  pub skip: Vec<SkipInfoRequestDto>,
  pub splitcount: i32,
  pub max_splitcount: i32,
}

// See [Wonder_Api_SkipInfoRequestDto_Fields]
#[derive(Debug, Deserialize)]
pub struct SkipInfoRequestDto {
  pub quest_id: i32,
  pub skip_count: i32,
}

// See [Wonder_Api_BattleskipResponseDto_Fields]
// See [Wonder_Api_BattlehuntingskipResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleSkipResponse {
  pub lvup: i32,
  pub reward: Vec<BattleSkipReward>,
}

impl CallCustom for BattleSkipResponse {}

pub async fn battle_skip(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleHuntingSkipRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_skip");

  Ok(Unsigned(BattleSkipResponse {
    lvup: 0,
    reward: vec![BattleSkipReward {
      dropnum: 1,
      exp: 500,
      money: 1000,
      itemtype: RemoteDataItemType::RealMoney.into(),
      itemid: 1,
      itemnum: 5000,
    }],
  }))
}
