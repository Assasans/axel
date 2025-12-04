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
pub mod capture;
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
pub mod mission;
pub mod notice;
pub mod party;
pub mod party_info;
pub mod present;
pub mod profile;
pub mod quest_fame;
pub mod quest_hunting;
pub mod quest_main;
pub mod smith_craft;
pub mod smith_sell;
pub mod smith_upgrade;
pub mod story;
pub mod transfer;
pub mod tutorial;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RemoteDataCommand {
  UserParamNew = 1,
  UserParamDelete = 2,
  /// Delete all user parameters
  UserParamClear = 3,
  UserParamAdd = 4,
  UserParamUpdate = 5,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
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
  // See [Wonder.UI.Mypage.HomeMemberSettingCell._SetData_d__8$$MoveNext], requires characterparameter
  Character,
  // See [Wonder.UI.Mypage.HomeMemberSettingCell._SetData_d__8$$MoveNext], requires memberparameter
  Member,
  MemberCostume,
  // See [Wonder.UI.Chara.MemberPlanningData$$.ctor_71291148], requires uniqid and is_trial
  SpecialSkill,
  // See [Wonder.UI.Chara.MemberPlanningData$$.ctor_71291148], requires uniqid and is_trial
  Weapon,
  // TODO: 6 - ?? - requires uniqid and is_trial
  // See [Wonder.UI.Common.MemberPlanningEquipBoxCell$$SetTrial]
  Unknown6,
  GachaTicket,
  FameRank,

  /* Dynamic analysis */
  MemberBackground,
  MaterialWA,
  MaterialLimit,
  SkillPotion,
  PowerPotion,
  // See [Wonder.UI.Chara.CharaPresentConfirmDialog._OnClickSendButton_d__29$$MoveNext], requires uniqid
  MaterialLove,
  ExchangeMedal,
  SkipTicket,
  Ticket,
  BossTicket,
  SlayerMedal,
  StaminaItem,
  DreamTicket,
  #[deprecated(note = "seems to be unused")]
  CampaignTicket,
  AssistTicket,
  AssistMaterial,
  DungeonRedraw,
  /// "Potential"
  CharacterPiece,
  EventTicket,
  FamePotion,
  FameScroll,
  CollaborationMedal,

  Another(i32),
}

// See [Wonder.Util.UserParam$$Add] code and [Wonder.Util.UserParam$$Get] cross-references
impl From<RemoteDataItemType> for i32 {
  fn from(item_type: RemoteDataItemType) -> Self {
    match item_type {
      RemoteDataItemType::Money => 1,
      RemoteDataItemType::RealMoney => 2,
      RemoteDataItemType::RealMoneyFree => 3,
      RemoteDataItemType::Member => 4,
      RemoteDataItemType::Weapon => 5,
      RemoteDataItemType::Unknown6 => 6,
      RemoteDataItemType::SkipTicket => 8,
      RemoteDataItemType::Stamina => 9,
      RemoteDataItemType::Exp => 10,
      RemoteDataItemType::Character => 11,
      RemoteDataItemType::SpecialSkill => 12,
      RemoteDataItemType::MemberCostume => 14,
      RemoteDataItemType::MaterialWA => 15,
      RemoteDataItemType::MaterialLimit => 16,
      RemoteDataItemType::SkillPotion => 17,
      RemoteDataItemType::PowerPotion => 18,
      RemoteDataItemType::MaterialLove => 19,
      RemoteDataItemType::ExchangeMedal => 20,
      RemoteDataItemType::Ticket => 21,
      RemoteDataItemType::Level => 23,
      RemoteDataItemType::MemberBackground => 24,
      // TODO: 26 - ??
      // See [Wonder.Util.UserParam$$UpdateVoiceItem]
      RemoteDataItemType::BossTicket => 27,
      RemoteDataItemType::SlayerMedal => 28,
      RemoteDataItemType::StaminaItem => 29,
      RemoteDataItemType::DreamTicket => 30,
      RemoteDataItemType::CampaignTicket => 32,
      RemoteDataItemType::GachaTicket => 33,
      // TODO: 35 - assist detail id - requires uniqid
      // See [Wonder.UI.Assist.AssistGachaResultWindow._ShowPoweUpEffect_d__29$$MoveNext] and [Wonder.Util.UserParam$$GetAssistByAssistId]
      RemoteDataItemType::AssistTicket => 36,
      RemoteDataItemType::AssistMaterial => 37,
      RemoteDataItemType::DungeonRedraw => 38,
      RemoteDataItemType::CharacterPiece => 39,
      RemoteDataItemType::FameRank => 40,
      RemoteDataItemType::EventTicket => 42,
      // TODO: 43 - skill pa fame unique id
      // See [Wonder.UI.Chara.MemberPlanningUnitCell$$GetSkillPaFameDatas]
      RemoteDataItemType::FamePotion => 44,
      RemoteDataItemType::FameScroll => 45,
      RemoteDataItemType::CollaborationMedal => 46,
      RemoteDataItemType::Another(value) => value,
    }
  }
}

impl From<i32> for RemoteDataItemType {
  fn from(value: i32) -> Self {
    match value {
      1 => RemoteDataItemType::Money,
      2 => RemoteDataItemType::RealMoney,
      3 => RemoteDataItemType::RealMoneyFree,
      4 => RemoteDataItemType::Member,
      5 => RemoteDataItemType::Weapon,
      6 => RemoteDataItemType::Unknown6,
      8 => RemoteDataItemType::SkipTicket,
      9 => RemoteDataItemType::Stamina,
      10 => RemoteDataItemType::Exp,
      11 => RemoteDataItemType::Character,
      12 => RemoteDataItemType::SpecialSkill,
      14 => RemoteDataItemType::MemberCostume,
      15 => RemoteDataItemType::MaterialWA,
      16 => RemoteDataItemType::MaterialLimit,
      17 => RemoteDataItemType::SkillPotion,
      18 => RemoteDataItemType::PowerPotion,
      19 => RemoteDataItemType::MaterialLove,
      20 => RemoteDataItemType::ExchangeMedal,
      21 => RemoteDataItemType::Ticket,
      23 => RemoteDataItemType::Level,
      24 => RemoteDataItemType::MemberBackground,
      26 => RemoteDataItemType::Another(26), // TODO: 26 - ??
      27 => RemoteDataItemType::BossTicket,
      28 => RemoteDataItemType::SlayerMedal,
      29 => RemoteDataItemType::StaminaItem,
      30 => RemoteDataItemType::DreamTicket,
      32 => RemoteDataItemType::CampaignTicket,
      33 => RemoteDataItemType::GachaTicket,
      35 => RemoteDataItemType::Another(35), // TODO: 35 - assist detail id - requires uniqid
      36 => RemoteDataItemType::AssistTicket,
      37 => RemoteDataItemType::AssistMaterial,
      38 => RemoteDataItemType::DungeonRedraw,
      39 => RemoteDataItemType::CharacterPiece,
      40 => RemoteDataItemType::FameRank,
      42 => RemoteDataItemType::EventTicket,
      43 => RemoteDataItemType::Another(43), // TODO: 43 - skill pa fame unique id
      44 => RemoteDataItemType::FamePotion,
      45 => RemoteDataItemType::FameScroll,
      46 => RemoteDataItemType::CollaborationMedal,
      _ => RemoteDataItemType::Another(value),
    }
  }
}

// See [Wonder_Data_MemberParameter_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberParameter {
  pub id: i32,
  pub lv: i32,
  pub exp: i32,
  pub member_id: i64,
  pub ac_skill_id_a: i64,
  pub ac_skill_lv_a: i32,
  pub ac_skill_val_a: i32,
  pub ac_skill_id_b: i64,
  pub ac_skill_lv_b: i32,
  pub ac_skill_val_b: i32,
  pub ac_skill_id_c: i64,
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
  pub character_id: i64,
  pub passiveskill: i64,
  pub specialattack: i64,
  pub resist_state: i32,
  pub resist_attr: i64,
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
  pub skill_pa_fame_list: Vec<SkillPaFame>,
}

// See [Wonder_Data_MemberParameter_SkillPaFame_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillPaFame {
  pub skill_pa_fame_id: i64,
  pub user_skill_pa_fame_id: i32,
  pub add_status_list: Vec<SkillPaFameAddStatus>,
}

// See [Wonder_Data_MemberParameter_SkillPaFame_AddStatus_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillPaFameAddStatus {
  #[serde(rename = "type")]
  pub kind: i32,
  pub value: i32,
}

// See [Wonder_Data_CharacterParameter_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterParameter {
  pub id: i64,
  pub character_id: i64,
  pub rank: i32,
  pub rank_progress: i32,
  pub sp_skill: Vec<SpSkill>,
  pub character_enhance_stage_id_list: Vec<i32>,
  pub character_piece_board_stage_id_list: Vec<i32>,
  pub is_trial: bool,
}

// See [Wonder_Data_CharacterParameter_SpSkill_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct SpSkill {
  #[serde(deserialize_with = "crate::serde_compat::as_i32")]
  pub group_id: i32,
  #[serde(deserialize_with = "crate::serde_compat::as_i64")]
  pub id: i64,
  #[serde(deserialize_with = "crate::serde_compat::as_i32")]
  pub lv: i32,
  // Absent in the client code
  // pub member_id: String,
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
  // "lock:1" see [Wonder.Util.UserParamItem$$get_isLock]
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

#[derive(Clone)]
pub struct ApiRequest {
  pub params: ApiCallParams,
  pub body: HashMap<String, String>,

  pub state: Arc<AppState>,
}
