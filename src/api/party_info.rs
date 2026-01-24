use crate::api::battle::BattleParty;
use crate::api::dungeon::PartyMember;
use crate::api::party::PartyWire;
use crate::api::surprise::BasicBattlePartyForm;
use crate::api::MemberFameStats;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::member::{FetchUserMemberSkillsIn, FetchUserMembers, Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInfo {
  pub party: Vec<Party>,
  pub members: Vec<PartyMember>,
  pub weapons: Vec<()>,
  pub accessories: Vec<()>,
}

impl CallCustom for PartyInfo {}

// See [Wonder_Api_PartyinfoPartyResponseDto_Fields]
// Thanks to https://youtu.be/Vv9r8wrDsZ8
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

  pub fn to_battle_party(&self) -> BattleParty {
    BattleParty {
      party_forms: self
        .party_forms
        .iter()
        .map(|ref form| form.to_basic_battle_party_form())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
      assist: self.assist as i32,
      sub_assists: self.sub_assists.iter().map(|sub_assist| *sub_assist as i32).collect(),
      party_passive_skill: self.party_passive_skill.clone(),
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

  pub fn to_basic_battle_party_form(&self) -> BasicBattlePartyForm {
    BasicBattlePartyForm {
      id: self.id,
      party_no: self.party_no,
      form_no: self.form_no,
      main: self.main,
      sub1: self.sub1,
      sub2: self.sub2,
      weapon: self.weapon as i32,
      acc: self.acc as i32,
      skill_pa_fame: self.skill_pa_fame,
    }
  }
}

// See [Wonder_Api_SpecialSkillInfoResponseDto_Fields]
/// ## Special skills
///
/// Client loads special skills from `skill_sp` master, keyed by `character_id`.
/// Skills are upgraded based on member's promotion level.
/// See `skill_sp_details.unlock_intimacy_lv`.
/// `skill_sp.pattern_number` can be 1 or 2.
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecialSkillInfo {
  /// Must be non-zero if main member is set.
  ///
  /// CLIENT QUIRK: Client shows list of skills for the character *of the skill itself*,
  /// not the main character of the party form. For example, sending Kazuma's skill ID
  /// will show his skills in selection screen, even if the main character is Megumin.
  pub special_skill_id: i32,
  pub trial: bool,
}

// See [Wonder_Api_PartyPassiveSkillInfoResponseDto_Fields]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

pub async fn party_info(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  // let response = include_str!("../party-info.json");
  // let response: Value = serde_json::from_str(response).unwrap();
  // return Ok(Signed(response, session));

  // let member_prototypes = get_master_manager()
  //   .get_master("member")
  //   .iter()
  //   .map(|data| MemberPrototype::load_from_id(data["id"].as_str().unwrap().parse::<i64>().unwrap()))
  //   .collect::<Vec<_>>();

  let client = state.get_database_client().await?;

  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let mut members = fetch_members.run(session.user_id).await.unwrap();
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut members.iter_mut().collect::<Vec<_>>())
    .await?;

  let statement = client
    .prepare(
      /* language=postgresql */
      r#"
      select
        up.party_id,
        -- Incidentally, client expects party name to be inside each form,
        -- which is exactly how JOIN returns it.
        up.name,
        upf.form_id,
        upf.main_member_id,
        upf.sub1_member_id,
        upf.sub2_member_id,
        upf.weapon_id,
        upf.accessory_id,
        upf.special_skill_id
      from user_parties up
        join user_party_forms upf
          on up.user_id = upf.user_id and up.party_id = upf.party_id
      where up.user_id = $1
      order by up.party_id, upf.form_id
    "#,
    )
    .await
    .context("failed to prepare statement")?;
  let parties = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;

  Ok(Signed(
    PartyWire {
      party: parties
        .iter()
        .chunk_by(|row| {
          let party_id: i64 = row.get(0);
          party_id as i32
        })
        .into_iter()
        .map(|(party_id, forms)| {
          let forms = forms.collect::<Vec<_>>();
          let forms = forms
            .into_iter()
            .map(|row| {
              let party_name: String = row.get(1);
              let form_id: i64 = row.get(2);
              let main_member_id: i64 = row.get(3);
              let sub1_member_id: i64 = row.get(4);
              let sub2_member_id: i64 = row.get(5);
              let weapon_id: i64 = row.get(6);
              let accessory_id: i64 = row.get(7);
              let special_skill_id: i64 = row.get(8);

              PartyForm {
                id: form_id as i32,
                form_no: form_id as i32,
                party_no: party_id,
                main: main_member_id as i32,
                sub1: sub1_member_id as i32,
                sub2: sub2_member_id as i32,
                weapon: weapon_id,
                acc: accessory_id,
                name: party_name,
                strength: 12300,
                specialskill: SpecialSkillInfo {
                  special_skill_id: special_skill_id as i32,
                  trial: false,
                },
                skill_pa_fame: 0,
              }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

          Party::new(forms, party_id)
        })
        .collect::<Vec<_>>(),
      // members: vec![
      //   MemberPrototype::load_from_id(1001100).create_party_member_wire(11),
      //   //   MemberPrototype::load_from_id(1064100).create_party_member_wire(12),
      // ],
      members: members
        .iter()
        .map(|member| member.to_party_member())
        .collect(),
      weapons: vec![],
      accessories: vec![],
    },
    session,
  ))
}
