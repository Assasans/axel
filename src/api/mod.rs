use std::collections::HashMap;
use std::sync::Arc;

use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::call::ApiCallParams;
use crate::AppState;

pub mod account;
pub mod assist;
pub mod battle;
pub mod character;
pub mod dungeon;
pub mod exchange;
pub mod friend;
pub mod gacha;
pub mod home;
pub mod honor_list;
pub mod idlink_confirm_google;
pub mod interaction;
pub mod items;
pub mod login;
pub mod login_bonus;
pub mod maintenance_check;
pub mod master_all;
pub mod master_list;
pub mod notice;
pub mod party_info;
pub mod profile;
pub mod quest_fame;
pub mod quest_hunting;
pub mod quest_main;
pub mod story;
pub mod tutorial;

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RemoteDataCommand {
  UserParamNew = 1,
  UserParamDelete = 2,
  UserParamClear = 3,
  UserParamAdd = 4,
  UserParamUpdate = 5,
}

#[derive(Debug, Clone, Copy)]
pub enum RemoteDataItemType {
  /* IDA static analysis */
  Money,
  RealMoney,
  RealMoneyFree,
  /// "Food" icon
  Stamina,
  Exp,
  /// Called "rank" in game
  Level,
  Member(i32),
  MemberCostume,
  Another(i32),

  /* Dynamic analysis */
  Background,
}

// See [Wonder.Util.UserParam$$Add]
impl From<&RemoteDataItemType> for i32 {
  fn from(item_type: &RemoteDataItemType) -> Self {
    match item_type {
      RemoteDataItemType::Money => 1,
      RemoteDataItemType::RealMoney => 2,
      RemoteDataItemType::RealMoneyFree => 3,
      RemoteDataItemType::Stamina => 9,
      RemoteDataItemType::Exp => 10,
      RemoteDataItemType::Level => 23,
      RemoteDataItemType::Background => 24,
      RemoteDataItemType::Member(value) => {
        assert!(
          !matches!(value, 1 | 2 | 3 | 9 | 10 | 14 | 23 | 24),
          "member value should not match any other type"
        );
        *value
      }
      RemoteDataItemType::MemberCostume => 14,
      RemoteDataItemType::Another(value) => *value,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberParameter {
  pub id: i32,
  pub lv: i32,
  pub exp: i32,
  pub member_id: i32,
  pub ac_skill_id_a: i32,
  pub ac_skill_lv_a: i32,
  pub ac_skill_val_a: i32,
  pub ac_skill_id_b: i32,
  pub ac_skill_lv_b: i32,
  pub ac_skill_val_b: i32,
  pub ac_skill_id_c: i32,
  pub ac_skill_lv_c: i32,
  pub ac_skill_val_c: i32,
  pub hp: i32,
  pub magicattack: i32,
  pub defense: i32,
  pub magicdefence: i32,
  pub agility: i32,
  pub dexterity: i32,
  pub luck: i32,
  pub limit_break: i32,
  pub character_id: i32,
  pub passiveskill: i32,
  pub specialattack: i32,
  pub resist_state: i32,
  pub resist_attr: i32,
  pub attack: i32,
  pub waiting_room: i32,
  pub main_strength: i32,
  pub main_strength_for_fame_quest: i32,
  pub sub_strength: i32,
  pub sub_strength_for_fame_quest: i32,
  pub sub_strength_bonus: i32,
  pub sub_strength_bonus_for_fame_quest: i32,
  pub fame_hp_rank: i32,
  pub fame_attack_rank: i32,
  pub fame_defense_rank: i32,
  pub fame_magicattack_rank: i32,
  pub fame_magicdefence_rank: i32,
  // Unknown structure
  pub skill_pa_fame_list: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterParameter {
  pub id: i64,
  pub character_id: i32,
  pub rank: i32,
  pub rank_progress: i32,
  pub sp_skill: Vec<SpSkill>,
  pub character_enhance_stage_id_list: Vec<i32>,
  pub character_piece_board_stage_id_list: Vec<i32>,
  pub is_trial: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpSkill {
  pub group_id: String,
  pub id: String,
  pub lv: String,
  pub member_id: String,
  pub is_trial: bool,
}

// See [Wonder_Api_RemotedataItemsResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteData {
  pub cmd: i32,
  // Original server sends [uid] as a number, but the client uses a string type
  #[serde(skip_serializing_if = "Option::is_none")]
  pub uid: Option<String>,
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
  pub uniqid: i32,
  pub lv: i32,
  pub tag: String,
  #[serde(rename = "memberparameter")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub member_parameter: Option<MemberParameter>,
  #[serde(rename = "characterparameter")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub character_parameter: Option<CharacterParameter>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub is_trial: Option<bool>,
}

impl RemoteData {
  pub fn new(cmd: i32, item_type: i32, item_id: i64, item_num: i32, uniqid: i32, lv: i32, tag: String) -> Self {
    Self {
      cmd,
      item_type,
      item_id,
      item_num,
      uniqid,
      lv,
      tag,
      uid: None,
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct NotificationData {
  pub cmd: i32,
  #[serde(rename = "type")]
  pub kind: i32,
  pub key: i32,
  pub value: i32,
  pub msgkey: String,
  pub tag: String,
}

impl NotificationData {
  pub fn new(cmd: i32, kind: i32, key: i32, value: i32, msgkey: String, tag: String) -> Self {
    Self {
      cmd,
      kind,
      key,
      value,
      msgkey,
      tag,
    }
  }
}

pub struct ApiRequest {
  pub params: ApiCallParams,
  pub body: HashMap<String, String>,

  pub state: Arc<AppState>,
}
