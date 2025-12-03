use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInfo {
  pub party: Vec<Party>,
  pub members: Vec<Member>,
  pub weapons: Vec<()>,
  pub accessories: Vec<()>,
}

impl CallCustom for PartyInfo {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
  pub party_forms: Vec<PartyForm>,
  pub party_no: u32,
  pub assist: u32,
  pub sub_assists: Vec<u32>,
  pub party_passive_skill: PartyPassiveSkillInfo,
}

impl Party {
  pub fn new(
    party_forms: Vec<PartyForm>,
    party_no: u32,
    assist: u32,
    sub_assists: Vec<u32>,
    party_passive_skill: Option<PartyPassiveSkillInfo>,
  ) -> Self {
    Self {
      party_forms,
      party_no,
      assist,
      sub_assists,
      party_passive_skill: party_passive_skill.unwrap_or_default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyForm {
  pub id: u32,
  pub form_no: u32,
  pub main: u32,
  pub sub1: u32,
  pub sub2: u32,
  pub weapon: u32,
  pub acc: u32,
  pub strength: u32,
  pub specialskill: SpecialSkillInfo,
  pub skill_pa_fame: u32,
  pub party_no: u32,
  pub name: String,
}

// See [Wonder_Api_SpecialSkillInfoResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecialSkillInfo {
  pub special_skill_id: i32,
  pub trial: bool,
}

// See [Wonder_Api_PartyPassiveSkillInfoResponseDto_Fields]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PartyPassiveSkillInfo {
  pub skill_id: i64,
  pub user_member_id: i64,
}

impl PartyPassiveSkillInfo {
  pub fn new(skill_id: i64, user_member_id: i64) -> Self {
    Self {
      skill_id,
      user_member_id,
    }
  }
}

// TODO: DungeonPartyMember has correct types, should move fields to this struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
  pub id: u32,
  pub lv: u32,
  pub exp: u32,
  pub member_id: u32,
  pub ac_skill_lv_a: u32,
  pub ac_skill_val_a: u32,
  pub ac_skill_lv_b: u32,
  pub ac_skill_val_b: u32,
  pub ac_skill_lv_c: u32,
  pub ac_skill_val_c: u32,
  pub hp: u32,
  pub attack: u32,
  pub magicattack: u32,
  pub defense: u32,
  pub magicdefence: u32,
  pub agility: u32,
  pub dexterity: u32,
  pub luck: u32,
  pub limit_break: u32,
  pub character_id: u32,
  pub waiting_room: u32,
  pub ex_flg: u32,
  pub is_undead: u32,
}

/// Party power (force) is calculated in [Wonder.UI.Data.PartyData$$get_Force], with formula being:
/// `party_forms.sum(strength) + (4 * party_assist.level)`
pub async fn party_info(_request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let response = include_str!("../party-info.json");
  let response: Value = serde_json::from_str(response).unwrap();
  return Ok(Signed(CallResponse::new_success(Box::new(response)), session));
  Ok(Signed(
    CallResponse::new_success(Box::new(json!({
      "party": [
        {
          "party_forms": [
            {
              "id": 1000001,
              "form_no": 1,
              "main": 1011100,
              "sub1": 1011100,
              "sub2": 1011100,
              "weapon": 42090,
              "acc": 36014,
              "strength": 393,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 1,
              "name": "Party1"
            }
          ],
          "party_no": 1,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        }
      ],
      "members": [
        {
          "id": 1011100,
          "lv": 1,
          "exp": 0,
          "member_id": 1011100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 93,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 128,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 122,
          "hp": 239,
          "attack": 25,
          "magicattack": 32,
          "defense": 24,
          "magicdefence": 24,
          "agility": 71,
          "dexterity": 74,
          "luck": 72,
          "limit_break": 0,
          "character_id": 101,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        }
      ],
      "weapons": [
        {
          "id": 42090,
          "weapon_id": 42090,
          "trial": false,
        }
      ],
      "accessories": [
        {
          "id": 36014,
          "accessory_id": 36014
        }
      ]
    }))),
    session,
  ))
}
