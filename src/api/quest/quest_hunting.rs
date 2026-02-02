//! Hierarchy is Area (Relic Quest) -> Stage (Eris - Beginner)
//! Reference: https://youtu.be/S9fX6sbXRHw (also shows character upgrade and promotion)

use crate::api::battle_multi::{BattleCharacterLove, BattleClearReward, BattleMemberExp};
use crate::api::master_all::{get_master_manager, get_masters};
use crate::api::party_info::{Party, PartyForm, SpecialSkillInfo};
use crate::api::smith_upgrade::{DungeonAreaMaterialInfoResponseDto, FameQuestMaterialInfoResponseDto};
use crate::api::{battle, MemberFameStats, RemoteDataItemType};
use crate::blob::IntoRemoteData;
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::member::{FetchUserMembers, FetchUserParty, Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};
use crate::api::battle::{apply_reward_multiplier, grant_rewards, make_battle_member_exp_and_character_love};
use crate::api::quest::parse_reward_items;

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
  /// Must contain all characters used in the battle
  pub love: Vec<BattleCharacterLove>,
  /// Must contain all members used in the battle
  pub member_exp: Vec<BattleMemberExp>,
  pub mission: Vec<i32>,
  pub reward: Vec<BattleReward>,
  pub clearreward: Vec<BattleClearReward>,
}

// See [Wonder_Api_FameQuestResultRewardResponseDto_Fields]
// See [Wonder_Api_BattlehuntingresultRewardResponseDto_Fields]
// See [Wonder_Api_ResultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  #[serde(with = "crate::bool_as_int")]
  pub is_rare: bool,
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
  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;

  let rewards = get_master_manager()
    .get_master("huntingquest_stage_itemreward")
    .iter()
    .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
    .collect::<HashMap<_, _>>();
  let mut rewards = parse_reward_items(rewards[&params.quest_id]);
  apply_reward_multiplier(&transaction, &session, &mut rewards).await?;
  let update_items = grant_rewards(&transaction, &session, &rewards).await?;
  transaction.commit().await.context("failed to commit transaction")?;

  let party = FetchUserParty::new(&client)
    .await?
    .run(session.user_id, params.party_id as i64)
    .await?;

  let (member_exp, love) = make_battle_member_exp_and_character_love(&party, &client, &session).await?;
  let mut response = CallResponse::new_success(Box::new(BattleHuntingResultResponse {
    limit: 1,
    exp: 230,
    lvup: 0,
    money: 42000,
    storyunlock: vec![],
    love,
    // TODO: We must send only members that are used in the party, otherwise hardlock occurs??
    member_exp,
    mission: params.clearquestmission,
    reward: rewards
      .iter()
      .map(|item| BattleReward {
        itemtype: item.item_type,
        itemid: item.item_id,
        itemnum: item.item_num,
        is_rare: item.item_rare,
      })
      .collect(),
    clearreward: vec![],
  }));
  response.remote.extend(update_items);
  Ok(Unsigned(response))
}

// See [Wonder_Api_HuntingquestListByItemResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingQuestListByItemResponse {
  pub huntingquests: Vec<HuntingQuest>,
  pub eventboxgachas: Vec<i32>,
  pub exchanges: Vec<i32>,
  pub expedition: i32,
  pub scorechallenge: i32,
  pub scorechallenge_ex: i32,
  pub dungeon: DungeonAreaMaterialInfoResponseDto,
  pub fame_quest: Vec<FameQuestMaterialInfoResponseDto>,
}

// See [Wonder_Api_HuntingquestListByItemHuntingquestsResponseDto_Fields]
// See [Wonder_Api_BlacksmithquestlistHuntingquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingQuest {
  pub quest_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub limit: i32,
  pub status: i32,
}

impl CallCustom for HuntingQuestListByItemResponse {}

// body={"item_type": "16", "item_id": "161"}
#[derive(Debug, Deserialize)]
pub struct HuntingQuestListByItemRequest {
  pub item_type: i32,
  pub item_id: i64,
}

pub async fn hunting_quest_list_by_item(
  Params(params): Params<HuntingQuestListByItemRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: hunting_quest_list_by_item");

  let stages = get_master_manager().get_master("huntingquest_stage");
  let rewards = get_master_manager()
    .get_master("huntingquest_stage_itemreward")
    .into_iter()
    .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
    .collect::<HashMap<_, _>>();

  let stages = stages.iter().filter(|stage| {
    let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
    let rewards = parse_reward_items(&rewards[&id]);

    rewards
      .iter()
      .any(|item| item.item_type == params.item_type && item.item_id == params.item_id)
  });

  Ok(Unsigned(HuntingQuestListByItemResponse {
    huntingquests: stages
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
    eventboxgachas: vec![20043],
    exchanges: vec![100],
    expedition: 0,
    scorechallenge: 0,
    scorechallenge_ex: 0,
    dungeon: DungeonAreaMaterialInfoResponseDto {
      area_ids: vec![],
      unlocked_area_ids: vec![],
      challenging_area_id: 0,
    },
    fame_quest: vec![],
  }))
}
