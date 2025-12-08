use jwt_simple::prelude::Deserialize;
use serde::Serialize;
use tracing::warn;

use crate::api::party_info::{PartyPassiveSkillInfo, SpecialSkillInfo};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_DungeonStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStatusResponse {
  pub area_id: i32,
  pub status: i32,
}

impl CallCustom for DungeonStatusResponse {}

pub async fn dungeon_status() -> impl IntoHandlerResponse {
  Ok(Unsigned(CallResponse::new_success(Box::new(DungeonStatusResponse {
    area_id: 11,
    status: 0,
  }))))
}

// See [Wonder_Api_DungeonAreaTopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaTopResponse {
  pub is_practice: bool,
  pub stage_state: DungeonStageState,
  pub party_set: DungeonPartySet,
  pub clear_info: DungeonAreaClearInfo,
  pub unchoosed_benefit_id_list: Vec<i32>,
  pub benefit_re_lottery_count: i32,
  pub is_allow_trial: bool,
}

impl CallCustom for DungeonAreaTopResponse {}

// See [Wonder_Api_DungeonAreaClearInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaClearInfo {
  pub clear_rank: i32,
  pub reward_items: Vec<DungeonAreaClearRewardInfo>,
}

// See [Wonder_Api_DungeonAreaClearRewardInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaClearRewardInfo {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// See [Wonder_Api_DungeonStageStateResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStageState {
  pub stage_id: i32,
  pub is_challenge: bool,
  pub enemies: Vec<DungeonStageEnemyState>,
}

// See [Wonder_Api_DungeonStageEnemyStateResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStageEnemyState {
  pub enemy_id: i32,
  pub current_hp: i32,
}

// See [Wonder_Api_DungeonPartySetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonPartySet {
  pub stage_party_set: DungeonStagePartySet,
  pub team_members: Vec<PartyMember>,
  pub team_weapons: Vec<PartyWeapon>,
  pub team_accessories: Vec<PartyAccessory>,
}

// See [Wonder_Api_PartyinfoMembersResponseDto_Fields]
// Extended by [Wonder_Api_DungeonPartyMembersResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct PartyMember {
  pub id: i32,
  pub lv: i32,
  pub exp: i32,
  pub member_id: i64,
  pub ac_skill_lv_a: i32,
  pub ac_skill_val_a: i64,
  pub ac_skill_lv_b: i32,
  pub ac_skill_val_b: i64,
  pub ac_skill_lv_c: i32,
  pub ac_skill_val_c: i64,
  pub hp: i32,
  pub attack: i32,
  pub magicattack: i32,
  pub defense: i32,
  pub magicdefence: i32,
  pub agility: i32,
  pub dexterity: i32,
  pub luck: i32,
  pub limit_break: i32,
  pub character_id: i64,
  pub waiting_room: i32,
  pub ex_flg: i32,
  pub is_undead: i32,
}

// See [Wonder_Api_PartyinfoWeaponsResponseDto_Fields]
// Extended by [Wonder_Api_DungeonPartyWeaponsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyWeapon {
  pub id: i64,
  pub weapon_id: i64,
  pub trial: bool,
}

// See [Wonder_Api_PartyinfoAccessoriesResponseDto_Fields]
// See [Wonder_Api_DungeonPartyAccessoriesResponseDto_Fields]
// See [Wonder_Api_PartychangelistAccessoriesResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyAccessory {
  pub id: i64,
  pub accessory_id: i64,
}

// See [Wonder_Api_DungeonStagePartySetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStagePartySet {
  pub party: Vec<DungeonStagePartyForm>,
  pub reserved_party: Vec<DungeonStagePartyForm>,
  pub assist: i64,
  pub sub_assists: Vec<i64>,
  pub assist_remain_count: i32,
  pub party_passive_skill: PartyPassiveSkillInfo,
}

// See [Wonder_Api_DungeonStagePartyFormResponseDto_Fields]
// extends [Wonder_Api_DungeonPartyFormResponseDto_Fields]
// extends [Wonder_Api_BasicPartyFormResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStagePartyForm {
  /* Wonder_Api_BasicPartyFormResponseDto_Fields */
  pub id: i32,
  pub form_no: i32,
  pub main: i32,
  pub sub1: i32,
  pub sub2: i32,
  pub weapon: i64,
  pub acc: i64,
  pub strength: i32,
  pub specialskill: SpecialSkillInfo,
  pub skill_pa_fame: i64,

  /* Wonder_Api_DungeonStagePartyFormResponseDto_Fields */
  pub current_hp: i32,
  pub max_hp: i32,
  pub current_sp: i32,
}

#[derive(Debug, Deserialize)]
pub struct DungeonAreaTopRequest {
  pub area_id: i32,
}

pub async fn dungeon_area_top(Params(params): Params<DungeonAreaTopRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_area_top");

  Ok(Unsigned(DungeonAreaTopResponse {
    is_practice: false,
    stage_state: DungeonStageState {
      stage_id: 1101,
      is_challenge: false,
      enemies: vec![],
    },
    party_set: DungeonPartySet {
      stage_party_set: DungeonStagePartySet {
        party: vec![],
        reserved_party: vec![],
        assist: 0,
        sub_assists: vec![],
        assist_remain_count: 0,
        party_passive_skill: Default::default(),
      },
      team_members: vec![],
      team_weapons: vec![],
      team_accessories: vec![],
    },
    clear_info: DungeonAreaClearInfo {
      clear_rank: 0,
      reward_items: vec![],
    },
    unchoosed_benefit_id_list: vec![],
    benefit_re_lottery_count: 0,
    is_allow_trial: false,
  }))
}

// TODO: dungeon_area_retire no body no reply
