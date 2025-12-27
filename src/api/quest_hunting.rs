//! Hierarchy is Area (Relic Quest) -> Stage (Eris - Beginner)
//! Reference: https://youtu.be/S9fX6sbXRHw (also shows character upgrade and promotion)

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;
use crate::api::{battle, RemoteDataItemType};
use crate::api::battle_multi::{BattleClearReward, BattleMemberExp, BattleMemberLove};
use crate::api::master_all::get_masters;
use crate::AppState;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;

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
  /// 0 - unlocked (new), 1 - unlocked, 2 - completed, 3 - 100% completed
  pub status: i32,
  /// Unknown
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
        stage_id: stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        status: 2,
        newstage: 0,
        task1: 1,
        task2: 1,
        task3: 0,
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

pub async fn quest_hunting_limit_stage_list() -> impl IntoHandlerResponse {
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

// body={"quest_id": "416055", "party_no": "1"}
#[derive(Debug, Deserialize)]
pub struct BattleHuntingStartRequest {
  pub quest_id: i32,
  #[serde(rename = "party_no")]
  pub party_id: i32,
}

pub async fn battle_hunting_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleHuntingStartRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_hunting_start");

  battle::make_battle_start(&state, &session, params.party_id).await
}

// See [Wonder_Api_BattlehuntingresultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleHuntingResultResponse {
  pub limit: i32,
  pub exp: i32,
  pub lvup: i32,
  pub money: i32,
  pub storyunlock: Vec<i32>,
  pub love: Vec<BattleMemberLove>,
  pub member_exp: Vec<BattleMemberExp>,
  pub mission: Vec<i32>,
  pub reward: Vec<BattleReward>,
  pub clearreward: Vec<BattleClearReward>,
}

// See [Wonder_Api_BattlehuntingresultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  pub is_rare: i32,
}

impl CallCustom for BattleHuntingResultResponse {}

// body={"party_no": "1", "win": "1", "quest_id": "416011", "memcheckcount": "0", "clearquestmission": "[12,13,15]", "wave": "3"}
#[derive(Debug, Deserialize)]
pub struct BattleHuntingResultRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub win: i32,
  pub quest_id: i32,
  pub memcheckcount: i32,
  pub clearquestmission: Vec<i32>,
  pub wave: i32,
}

pub async fn battle_hunting_result(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleHuntingResultRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_hunting_result");

  Ok(Unsigned(BattleHuntingResultResponse {
    limit: 1,
    exp: 230,
    lvup: 3,
    money: 42000,
    storyunlock: vec![],
    love: vec![],
    member_exp: vec![],
    mission: params.clearquestmission,
    reward: vec![
      BattleReward {
        itemtype: RemoteDataItemType::RealMoney.into(),
        itemid: 1,
        itemnum: 54000,
        is_rare: 1,
      },
    ],
    clearreward: vec![],
  }))
}
