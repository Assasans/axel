use serde::{Deserialize, Serialize};

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

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
  pub party_passive_skill: PartyPassiveSkill,
}

impl Party {
  pub fn new(
    party_forms: Vec<PartyForm>,
    party_no: u32,
    assist: u32,
    sub_assists: Vec<u32>,
    party_passive_skill: Option<PartyPassiveSkill>,
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
  pub specialskill: Specialskill,
  pub skill_pa_fame: u32,
  pub party_no: u32,
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Specialskill {
  pub special_skill_id: u32,
  pub trial: bool,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PartyPassiveSkill {
  pub skill_id: u32,
  pub user_member_id: u32,
}

impl PartyPassiveSkill {
  pub fn new(skill_id: u32, user_member_id: u32) -> Self {
    Self {
      skill_id,
      user_member_id,
    }
  }
}

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

pub async fn route(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let response = include_str!("../party-info.json");
  let response: PartyInfo = serde_json::from_str(response).unwrap();
  Ok((CallResponse::new_success(Box::new(response)), true))
}
