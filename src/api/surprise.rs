// Called "Branch" in-game

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::warn;

use crate::api::battle::BattleMember;
use crate::api::master_all::get_masters;
use crate::api::party_info::PartyPassiveSkillInfo;
use crate::api::{NotificationData, RemoteDataItemType};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_SurpriseMiniEventSelectResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseMiniEventSelectResponse {
  pub select_list: Vec<SurpriseEvent>,
}

impl CallCustom for SurpriseMiniEventSelectResponse {}

// See [Wonder_Api_SurpriseEventStockListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseEvent {
  pub user_stock_id: i32,
  pub surprise_event_id: i32,
  pub expired_date: i64,
}

pub async fn surprise_mini_event_select() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let surprise_events: Vec<Value> = serde_json::from_str(&masters["surprise_event"].master_decompressed).unwrap();

  warn!("encountered stub: surprise_mini_event_select");

  Ok(Unsigned(SurpriseMiniEventSelectResponse {
    select_list: surprise_events
      .iter()
      .map(|event| SurpriseEvent {
        user_stock_id: 42,
        surprise_event_id: event["id"].as_str().unwrap().parse().unwrap(),
        expired_date: 0,
      })
      .collect(),
  }))
}

// See [Wonder_Api_SurpriseMiniEventTopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseMiniEventTop {
  pub best_score: i32,
  pub expired_date: i64,
}

impl CallCustom for SurpriseMiniEventTop {}

#[derive(Debug, Deserialize)]
pub struct SurpriseMiniEventTopRequest {
  pub surprise_event_id: i32,
  pub user_stock_id: i32,
}

// user_stock_id=42
// surprise_event_id=10001
pub async fn surprise_mini_event_top(Params(params): Params<SurpriseMiniEventTopRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: surprise_mini_event_top");

  Ok(Unsigned(SurpriseMiniEventTop {
    best_score: 2112,
    expired_date: 0,
  }))
}

// See [Wonder_Api_SurpriseQuestStartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseQuestStart {
  pub party: SurpriseQuestStartParty,
  pub members: Vec<BattleMember>,
}

impl CallCustom for SurpriseQuestStart {}

// See [Wonder_Api_SurpriseQuestStartPartyResponseDto_Fields]
// TODO: Same as BattleParty?
#[derive(Debug, Serialize)]
pub struct SurpriseQuestStartParty {
  pub party_forms: Vec<BasicBattlePartyForm>,
  pub assist: i32,
  pub sub_assists: Vec<i32>,
  pub party_passive_skill: PartyPassiveSkillInfo,
}

// See [Wonder_Api_BasicBattlePartyFormResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BasicBattlePartyForm {
  pub id: i32,
  pub party_no: i32,
  pub form_no: i32,
  pub main: i32,
  pub sub1: i32,
  pub sub2: i32,
  pub weapon: i32,
  pub acc: i32,
  pub skill_pa_fame: i64,
}

#[derive(Debug, Deserialize)]
pub struct SurpriseQuestStartRequest {
  pub surprise_quest_id: i32,
  pub party_no: i32,
  pub user_stock_id: i32,
  pub surprise_event_id: i32,
}

// surprise_quest_id=10001
// party_no=1
// user_stock_id=42
// surprise_event_id=10001
pub async fn surprise_quest_start(Params(params): Params<SurpriseQuestStartRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: surprise_quest_start");

  let mut response = CallResponse::new_success(Box::new(json!({
    "party": {
      "party_forms": [
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 1,
          "main": 11,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 2,
          "main": 12,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 3,
          "main": 10,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 4,
          "main": 0,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 5,
          "main": 0,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        }
      ],
      "assist": 0,
      "sub_assists": [],
      "party_passive_skill": {
        "skill_id": 0,
        "user_member_id": 0
      }
    },
    "members": [
      {
        "id": 11,
        "lv": 1,
        "exp": 0,
        "member_id": 1001100,
        "ac_skill_id_a": 10000100,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 110,
        "ac_skill_id_b": 54000130,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 0,
        "ac_skill_id_c": 22000140,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 130,
        "hp": 245,
        "magicattack": 26,
        "defense": 20,
        "magicdefence": 19,
        "agility": 72,
        "dexterity": 78,
        "luck": 88,
        "limit_break": 0,
        "character_id": 100,
        "passiveskill": 210201,
        "specialattack": 100001,
        "resist_state": 210201,
        "resist_attr": 150000000,
        "attack": 27,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      },
      {
        "id": 12,
        "lv": 1,
        "exp": 0,
        "member_id": 1011100,
        "ac_skill_id_a": 10000100,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 110,
        "ac_skill_id_b": 31002024,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 170,
        "ac_skill_id_c": 12000146,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 152,
        "hp": 252,
        "magicattack": 31,
        "defense": 21,
        "magicdefence": 23,
        "agility": 66,
        "dexterity": 76,
        "luck": 10,
        "limit_break": 0,
        "character_id": 101,
        "passiveskill": 220001,
        "specialattack": 101001,
        "resist_state": 220001,
        "resist_attr": 155000000,
        "attack": 28,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      },
      {
        "id": 10,
        "lv": 1,
        "exp": 0,
        "member_id": 1064217,
        "ac_skill_id_a": 210000000000010000i64,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 100,
        "ac_skill_id_b": 210042000000312082i64,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 128,
        "ac_skill_id_c": 212200000000030074i64,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 173,
        "hp": 247,
        "magicattack": 36,
        "defense": 22,
        "magicdefence": 26,
        "agility": 69,
        "dexterity": 67,
        "luck": 68,
        "limit_break": 0,
        "character_id": 106,
        "passiveskill": 212004,
        "specialattack": 106001,
        "resist_state": 212004,
        "resist_attr": 100005000,
        "attack": 29,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      }
    ]
  })));
  response.add_notifications(vec![NotificationData::new(1, 7, 6, 0, "".to_string(), "".to_string())]);
  Ok(Unsigned(response))
}

// See [Wonder_Api_SurpriseQuestResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseQuestResult {
  pub reward: Vec<SurpriseQuestResultReward>,
  pub previous_best_score: i32,
  pub best_score: i32,
}

impl CallCustom for SurpriseQuestResult {}

// See [Wonder_Api_SurpriseQuestResultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseQuestResultReward {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
  pub is_rare: i32,
}

/// Cabbage fights
pub async fn surprise_quest_result() -> impl IntoHandlerResponse {
  warn!("encountered stub: surprise_quest_result");

  Ok(Unsigned(SurpriseQuestResult {
    reward: vec![SurpriseQuestResultReward {
      item_type: RemoteDataItemType::RealMoney.into(),
      item_id: 1,
      // - “Ka~zuma-san! How much did you get from this quest?”
      // - “A little over a million.”
      // - “A million!?”
      item_num: 1_042_000,
      is_rare: 1,
    }],
    previous_best_score: 2112,
    best_score: 4242,
  }))
}

// See [Wonder_Api_SurpriseShortEventResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseShortEvent {
  pub surprise_short_id: i32,
  pub result: SurpriseShortEventResult,
}

impl CallCustom for SurpriseShortEvent {}

#[derive(Debug, Deserialize)]
pub struct SurpriseShortEventRequest {
  pub surprise_event_id: i32,
  pub user_stock_id: i32,
}

// surprise_event_id=20002
// user_stock_id=42
/// Explosions
pub async fn surprise_short_event(Params(params): Params<SurpriseShortEventRequest>) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let surprise_events: Vec<Value> = serde_json::from_str(&masters["surprise_short"].master_decompressed).unwrap();
  // TODO: Seems like there is no [event ID -> pool of result IDs] mapping, thus we would need to make our own
  let surprise_event_results: Vec<Value> =
    serde_json::from_str(&masters["surprise_short_result"].master_decompressed).unwrap();

  let surprise_event = surprise_events
    .iter()
    .find(|event| event["surprise_event_id"].as_str().unwrap().parse::<i32>().unwrap() == params.surprise_event_id)
    .unwrap();

  warn!(?params, "encountered stub: surprise_short_event");

  Ok(Unsigned(SurpriseShortEvent {
    surprise_short_id: surprise_event["surprise_short_id"].as_str().unwrap().parse().unwrap(),
    result: SurpriseShortEventResult {
      result_pattern_id: 8,
      // XXX: Some values (e.g. 15 for "Explosion Walk") cause softlock (fixable with Escape)
      // result_pattern_id: surprise_event_results
      //   .iter()
      //   .choose(&mut rand::rng())
      //   .unwrap()["result_pattern_id"]
      //   .as_str()
      //   .unwrap()
      //   .parse()
      //   .unwrap(),
    },
  }))
}

// See [Wonder_Api_SurpriseShortEventResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseShortEventResult {
  pub result_pattern_id: i32,
}

// See [Wonder_Api_SurpriseStoryStartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseStoryStart {
  pub result_id_list: Vec<i32>,
}

impl CallCustom for SurpriseStoryStart {}

#[derive(Debug, Deserialize)]
pub struct SurpriseStoryStartRequest {
  pub surprise_event_id: i32,
  pub user_stock_id: i32,
  pub surprise_story_id: i32,
}

// surprise_event_id=40001
// user_stock_id=42
// surprise_story_id=1001
/// Vanir box gambling
/// TODO: Is it broken? No selection appears in-game besides Vanir Box case.
pub async fn surprise_story_start(Params(params): Params<SurpriseStoryStartRequest>) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let surprise_stories: Vec<Value> =
    serde_json::from_str(&masters["surprise_story_result"].master_decompressed).unwrap();

  warn!(?params, "encountered stub: surprise_story_start");

  Ok(Unsigned(SurpriseStoryStart {
    result_id_list: surprise_stories
      .iter()
      .map(|story| story["id"].as_str().unwrap().parse().unwrap())
      .collect(),
  }))
}

#[derive(Debug, Deserialize)]
pub struct SurpriseStorySelectRequest {
  pub surprise_story_id: i32,
  pub surprise_event_id: i32,
  pub user_stock_id: i32,
  pub result_id: i32,
}

// surprise_story_id=1001
// surprise_event_id=40001
// user_stock_id=42
// result_id=2
pub async fn surprise_story_select(Params(params): Params<SurpriseStorySelectRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: surprise_story_select");

  // See [Wonder_Api_SurpriseStorySelectResponseDto_Fields]
  Ok(Unsigned(()))
}

// See [Wonder_Api_SurpriseStoryResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseStoryResult {
  pub reward: Vec<SurpriseStoryReward>,
}

impl CallCustom for SurpriseStoryResult {}

// See [Wonder_Api_StoryrewardRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct SurpriseStoryReward {
  pub item_type: i32,
  pub item_id: i64,
  pub num: i32,
}

#[derive(Debug, Deserialize)]
pub struct SurpriseStoryResultRequest {
  pub surprise_story_id: i32,
  pub surprise_event_id: i32,
  pub user_stock_id: i32,
}

// surprise_story_id=1001
// surprise_event_id=40001
// user_stock_id=42
pub async fn surprise_story_result(Params(params): Params<SurpriseStoryResultRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: surprise_story_result");

  Ok(Unsigned(SurpriseStoryResult {
    reward: vec![SurpriseStoryReward {
      item_type: RemoteDataItemType::RealMoney.into(),
      item_id: 1,
      num: 520_000,
    }],
  }))
}
