use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::dungeon::PartyMember;
use crate::api::master_all::get_master_manager;
use crate::api::party::PartyWire;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::member::MemberPrototype;
use crate::user::session::Session;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInfo {
  pub party: Vec<Party>,
  pub members: Vec<PartyMember>,
  pub weapons: Vec<()>,
  pub accessories: Vec<()>,
}

impl CallCustom for PartyInfo {}

// See [Wonder_Api_PartyinfoPartyResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
  /// ## Party formations
  /// Front 1, Front 2, Front 3, Rear 1, Rear 2.
  /// Rear members enter battle after front member is down.
  pub party_forms: [PartyForm; 5],
  pub party_no: i32,
  /// Assist stats affect the stats of main party members.
  pub assist: i64,
  pub sub_assists: Vec<i64>,
  /// "Party Trait"
  pub party_passive_skill: PartyPassiveSkillInfo,
}

impl Party {
  pub fn new(party_forms: [PartyForm; 5], party_no: i32) -> Self {
    Self {
      party_forms,
      party_no,
      assist: 0,
      sub_assists: vec![],
      party_passive_skill: PartyPassiveSkillInfo {
        skill_id: 0,
        user_member_id: 0,
      },
    }
  }
}

// See [Wonder_Api_PartyinfoPartyFormResponseDto_Fields]
// extends [Wonder_Api_BasicPartyFormResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct PartyForm {
  /* Wonder_Api_BasicPartyFormResponseDto_Fields */
  pub id: i32,
  pub form_no: i32,
  pub main: i32,
  /// ## Sub-members
  /// 30% of sub-member stats and passive skills are applied to the main member.
  /// If sub-member and main member are the same character, main member gets additional 10% stat boost.
  pub sub1: i32,
  pub sub2: i32,
  pub weapon: i64,
  pub acc: i64,
  pub strength: i32,
  pub specialskill: SpecialSkillInfo,
  /// "Fame Trait"
  pub skill_pa_fame: i64,
  /* Wonder_Api_PartyinfoPartyFormResponseDto_Fields */
  pub party_no: i32,
  /// Must not be empty
  pub name: String,
}

impl PartyForm {
  pub fn new(id: i32, form_no: i32, party_no: i32, main: i32) -> Self {
    Self {
      id,
      form_no,
      main,
      sub1: 0,
      sub2: 0,
      weapon: 0,
      acc: 0,
      strength: 1,
      specialskill: SpecialSkillInfo {
        special_skill_id: 100001,
        trial: false,
      },
      skill_pa_fame: 0,
      party_no,
      name: format!("Party{}", party_no),
    }
  }
}

// See [Wonder_Api_SpecialSkillInfoResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecialSkillInfo {
  /// Must be non-zero if main member is set
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

impl Party {
  /// See [Wonder.UI.Data.PartyData$$get_Force]
  pub fn power(&self, assist_level: u32) -> u32 {
    let party_forms_strength: u32 = self.party_forms.iter().map(|form| form.strength as u32).sum();
    party_forms_strength + (4 * assist_level)
  }
}

pub async fn party_info(session: Arc<Session>) -> impl IntoHandlerResponse {
  // let response = include_str!("../party-info.json");
  // let response: Value = serde_json::from_str(response).unwrap();
  // return Ok(Signed(response, session));

  let members = get_master_manager().get_master("member");

  Ok(Signed(
    PartyWire {
      party: vec![Party::new(
        [
          PartyForm::new(666431194, 1, 1, 11),
          PartyForm::new(666431194, 2, 1, 12),
          PartyForm::new(666431194, 3, 1, 13),
          PartyForm::new(666431194, 4, 1, 14),
          PartyForm::new(666431194, 5, 1, 0),
        ],
        1,
      )],
      // members: vec![
      //   MemberPrototype::load_from_id(1001100).create_party_member_wire(11),
      //   //   MemberPrototype::load_from_id(1064100).create_party_member_wire(12),
      // ],
      members: members
        .iter()
        .enumerate()
        .map(|(index, member)| {
          MemberPrototype::load_from_id(member["id"].as_str().unwrap().parse::<i64>().unwrap())
            .create_party_member_wire(index as i32 + 1)
        })
        .collect::<Vec<_>>(),
      weapons: vec![],
      accessories: vec![],
    },
    session,
  ))
}
