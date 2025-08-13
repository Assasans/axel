use anyhow::Context;
use serde::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

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

pub async fn character_piece_board_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let character_id: i32 = request.body["character_id"]
    .parse()
    .context("failed to parse character_id as i32")?;

  Ok((
    CallResponse::new_success(Box::new(CharacterPieceBoardInfo {
      board_info: vec![PieceBoardInfo {
        board_id: 100001,
        stage_id: 1,
      }],
      reward_ids: vec![],
    })),
    false,
  ))
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

// TODO: BROKEN - hard lockup
pub async fn character_enhance_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let character_id: i32 = request.body["character_id"]
    .parse()
    .context("failed to parse character_id as i32")?;

  Ok((
    CallResponse::new_success(Box::new(CharacterEnhanceInfo {
      progress: vec![/*CharacterEnhanceInfoProgress {
        root_id: 1,
        root_stage_id: 5,
        stage_id: 1810105,
        parameter: EnhanceParameter {
          hp: 1,
          attack: 2,
          magicattack: 3,
          defense: 4,
          magicdefence: 5,
          agility: 6,
          dexterity: 7,
          luck: 8,
        },
        unique_weapon_id: 0,
        specialskill: vec![],
        unique_stone: EnhanceUniqueStone {
          unique_stone_id: 0,
          unique_stone_lv: 0,
        },
        material_items: vec![],
        money: 900,
      }*/],
      trial_timestamp: chrono::Utc::now().timestamp(),
    })),
    false,
  ))
}
