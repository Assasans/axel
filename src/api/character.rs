use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::api::master_all::get_masters;
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_CharacterPieceBoardInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct CharacterPieceBoardInfo {
  pub board_info: Vec<PieceBoardInfo>,
  pub reward_ids: Vec<i32>,
}

impl CallCustom for CharacterPieceBoardInfo {}

// See [Wonder_Api_PieceBoardInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PieceBoardInfo {
  pub board_id: i32,
  pub stage_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CharacterPieceBoardInfoRequest {
  pub character_id: i32,
}

pub async fn character_piece_board_info(
  Params(params): Params<CharacterPieceBoardInfoRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: character_piece_board_info");

  Ok(Unsigned(CallResponse::new_success(Box::new(CharacterPieceBoardInfo {
    board_info: vec![PieceBoardInfo {
      board_id: 100001,
      stage_id: 1,
    }],
    reward_ids: vec![],
  }))))
}

// See [Wonder_Api_CharacterEnhanceInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct CharacterEnhanceInfo {
  pub progress: Vec<CharacterEnhanceInfoProgress>,
  pub trial_timestamp: i64,
}

impl CallCustom for CharacterEnhanceInfo {}

// See [System_Collections_Generic_List_CharacterEnhanceInfoProgressResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct CharacterEnhanceInfoProgress {
  pub root_id: i32,
  pub root_stage_id: i32,
  pub stage_id: i32,
  pub parameter: EnhanceParameter,
  pub unique_weapon_id: i64,
  pub specialskill: Vec<EnhanceSpecialSkill>,
  pub unique_stone: EnhanceUniqueStone,
  pub material_items: Vec<CharacterEnhanceMaterial>,
  pub money: i32,
}

// See [Wonder_Api_EnhanceSpecialskillResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct EnhanceSpecialSkill {
  pub sp_id: i64,
  pub sp_group_id: i64,
  pub sp_lv: i32,
}

// See [Wonder_Api_EnhanceUniqueStoneResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct EnhanceUniqueStone {
  pub unique_stone_id: i64,
  pub unique_stone_lv: i32,
}

// See [Wonder_Api_EnhanceParameterResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct EnhanceParameter {
  pub hp: i32,
  pub attack: i32,
  pub magicattack: i32,
  pub defense: i32,
  pub magicdefence: i32,
  pub agility: i32,
  pub dexterity: i32,
  pub luck: i32,
}

// See [Wonder_Api_CharacterEnhanceMaterialResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct CharacterEnhanceMaterial {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct CharacterEnhanceInfoRequest {
  pub character_id: i32,
}

// Thanks to https://youtu.be/o5UUz2kHhto for unbricking this endpoint
pub async fn character_enhance_info(Params(params): Params<CharacterEnhanceInfoRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: character_enhance_info");

  let masters = get_masters().await;
  let trials: Vec<Value> = serde_json::from_str(&masters["character_enhance"].master_decompressed).unwrap();
  let trials = trials
    .iter()
    .filter(|trial| trial["character_id"].as_str().unwrap().parse::<i32>().unwrap() == params.character_id)
    .collect::<Vec<_>>();

  Ok(Unsigned(CharacterEnhanceInfo {
    progress: trials
      .iter()
      .map(|trial| CharacterEnhanceInfoProgress {
        root_id: trial["root_id"].as_str().unwrap().parse().unwrap(),
        root_stage_id: trial["root_stage_id"].as_str().unwrap().parse().unwrap(),
        stage_id: trial["stage_id"].as_str().unwrap().parse().unwrap(),
        parameter: EnhanceParameter {
          hp: 100,
          attack: 50,
          magicattack: 50,
          defense: 30,
          magicdefence: 30,
          agility: 20,
          dexterity: 20,
          luck: 10,
        },
        unique_weapon_id: 0,
        specialskill: vec![],
        unique_stone: EnhanceUniqueStone {
          unique_stone_id: 0,
          unique_stone_lv: 0,
        },
        material_items: vec![],
        money: 1000,
      })
      .collect(),
    trial_timestamp: chrono::Utc::now().timestamp(),
  }))
}
