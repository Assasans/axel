use jwt_simple::prelude::Serialize;
use serde_json::json;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct PartyInfo {
  pub party: Vec<Party>,
  pub members: Vec<Member>,
  pub weapons: Vec<()>,
  pub accessories: Vec<()>,
}

impl CallCustom for PartyInfo {}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct Specialskill {
  pub special_skill_id: u32,
  pub trial: bool,
}

#[derive(Default, Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    // CallResponse::new_success(Box::new(PartyInfo {
    //   party: vec![
    //   ],
    //   members: vec![],
    //   weapons: vec![],
    //   accessories: vec![],
    // })),
    CallResponse::new_success(Box::new(json!({
      "party": [
        {
          "party_forms": [
            {
              "id": 666431194,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 1,
              "name": "Party1"
            },
            {
              "id": 666431194,
              "form_no": 2,
              "main": 12,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 477,
              "specialskill": {
                "special_skill_id": 101001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 1,
              "name": "Party1"
            },
            {
              "id": 666431194,
              "form_no": 3,
              "main": 10,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 487,
              "specialskill": {
                "special_skill_id": 106001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 1,
              "name": "Party1"
            },
            {
              "id": 666431194,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 1,
              "name": "Party1"
            },
            {
              "id": 666431194,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
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
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 2,
              "name": "Party2"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 2,
              "name": "Party2"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 2,
              "name": "Party2"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 2,
              "name": "Party2"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 2,
              "name": "Party2"
            }
          ],
          "party_no": 2,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 3,
              "name": "Party3"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 3,
              "name": "Party3"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 3,
              "name": "Party3"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 3,
              "name": "Party3"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 3,
              "name": "Party3"
            }
          ],
          "party_no": 3,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 4,
              "name": "Party4"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 4,
              "name": "Party4"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 4,
              "name": "Party4"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 4,
              "name": "Party4"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 4,
              "name": "Party4"
            }
          ],
          "party_no": 4,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 5,
              "name": "Party5"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 5,
              "name": "Party5"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 5,
              "name": "Party5"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 5,
              "name": "Party5"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 5,
              "name": "Party5"
            }
          ],
          "party_no": 5,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 6,
              "name": "Party6"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 6,
              "name": "Party6"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 6,
              "name": "Party6"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 6,
              "name": "Party6"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 6,
              "name": "Party6"
            }
          ],
          "party_no": 6,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 7,
              "name": "Party7"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 7,
              "name": "Party7"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 7,
              "name": "Party7"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 7,
              "name": "Party7"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 7,
              "name": "Party7"
            }
          ],
          "party_no": 7,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 8,
              "name": "Party8"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 8,
              "name": "Party8"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 8,
              "name": "Party8"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 8,
              "name": "Party8"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 8,
              "name": "Party8"
            }
          ],
          "party_no": 8,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 9,
              "name": "Party9"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 9,
              "name": "Party9"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 9,
              "name": "Party9"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 9,
              "name": "Party9"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 9,
              "name": "Party9"
            }
          ],
          "party_no": 9,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 10,
              "name": "Party10"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 10,
              "name": "Party10"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 10,
              "name": "Party10"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 10,
              "name": "Party10"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 10,
              "name": "Party10"
            }
          ],
          "party_no": 10,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 11,
              "name": "Party11"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 11,
              "name": "Party11"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 11,
              "name": "Party11"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 11,
              "name": "Party11"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 11,
              "name": "Party11"
            }
          ],
          "party_no": 11,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 12,
              "name": "Party12"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 12,
              "name": "Party12"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 12,
              "name": "Party12"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 12,
              "name": "Party12"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 12,
              "name": "Party12"
            }
          ],
          "party_no": 12,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 13,
              "name": "Party13"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 13,
              "name": "Party13"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 13,
              "name": "Party13"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 13,
              "name": "Party13"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 13,
              "name": "Party13"
            }
          ],
          "party_no": 13,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 14,
              "name": "Party14"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 14,
              "name": "Party14"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 14,
              "name": "Party14"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 14,
              "name": "Party14"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 14,
              "name": "Party14"
            }
          ],
          "party_no": 14,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 15,
              "name": "Party15"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 15,
              "name": "Party15"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 15,
              "name": "Party15"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 15,
              "name": "Party15"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 15,
              "name": "Party15"
            }
          ],
          "party_no": 15,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 16,
              "name": "Party16"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 16,
              "name": "Party16"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 16,
              "name": "Party16"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 16,
              "name": "Party16"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 16,
              "name": "Party16"
            }
          ],
          "party_no": 16,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 17,
              "name": "Party17"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 17,
              "name": "Party17"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 17,
              "name": "Party17"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 17,
              "name": "Party17"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 17,
              "name": "Party17"
            }
          ],
          "party_no": 17,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 18,
              "name": "Party18"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 18,
              "name": "Party18"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 18,
              "name": "Party18"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 18,
              "name": "Party18"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 18,
              "name": "Party18"
            }
          ],
          "party_no": 18,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 19,
              "name": "Party19"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 19,
              "name": "Party19"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 19,
              "name": "Party19"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 19,
              "name": "Party19"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 19,
              "name": "Party19"
            }
          ],
          "party_no": 19,
          "assist": 0,
          "sub_assists": [],
          "party_passive_skill": {
            "skill_id": 0,
            "user_member_id": 0
          }
        },
        {
          "party_forms": [
            {
              "id": 0,
              "form_no": 1,
              "main": 11,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 444,
              "specialskill": {
                "special_skill_id": 100001,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 20,
              "name": "Party20"
            },
            {
              "id": 0,
              "form_no": 2,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 20,
              "name": "Party20"
            },
            {
              "id": 0,
              "form_no": 3,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 20,
              "name": "Party20"
            },
            {
              "id": 0,
              "form_no": 4,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 20,
              "name": "Party20"
            },
            {
              "id": 0,
              "form_no": 5,
              "main": 0,
              "sub1": 0,
              "sub2": 0,
              "weapon": 0,
              "acc": 0,
              "strength": 0,
              "specialskill": {
                "special_skill_id": 0,
                "trial": false
              },
              "skill_pa_fame": 0,
              "party_no": 20,
              "name": "Party20"
            }
          ],
          "party_no": 20,
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
          "id": 11,
          "lv": 4,
          "exp": 150,
          "member_id": 1001100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 0,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 130,
          "hp": 277,
          "attack": 32,
          "magicattack": 31,
          "defense": 24,
          "magicdefence": 22,
          "agility": 72,
          "dexterity": 78,
          "luck": 88,
          "limit_break": 0,
          "character_id": 100,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 8,
          "lv": 1,
          "exp": 0,
          "member_id": 1002102,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 20,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 130,
          "hp": 257,
          "attack": 28,
          "magicattack": 28,
          "defense": 21,
          "magicdefence": 20,
          "agility": 73,
          "dexterity": 79,
          "luck": 87,
          "limit_break": 0,
          "character_id": 100,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 12,
          "lv": 4,
          "exp": 150,
          "member_id": 1011100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 170,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 152,
          "hp": 285,
          "attack": 33,
          "magicattack": 37,
          "defense": 25,
          "magicdefence": 27,
          "agility": 66,
          "dexterity": 76,
          "luck": 10,
          "limit_break": 0,
          "character_id": 101,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 13,
          "lv": 1,
          "exp": 0,
          "member_id": 1021100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 20,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 152,
          "hp": 202,
          "attack": 26,
          "magicattack": 30,
          "defense": 18,
          "magicdefence": 21,
          "agility": 68,
          "dexterity": 71,
          "luck": 72,
          "limit_break": 0,
          "character_id": 102,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 14,
          "lv": 1,
          "exp": 0,
          "member_id": 1031100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 127,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 150,
          "hp": 281,
          "attack": 29,
          "magicattack": 24,
          "defense": 24,
          "magicdefence": 24,
          "agility": 68,
          "dexterity": 10,
          "luck": 64,
          "limit_break": 0,
          "character_id": 103,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 2,
          "lv": 1,
          "exp": 0,
          "member_id": 1034100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 102,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 130,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 150,
          "hp": 330,
          "attack": 35,
          "magicattack": 29,
          "defense": 28,
          "magicdefence": 28,
          "agility": 68,
          "dexterity": 10,
          "luck": 64,
          "limit_break": 0,
          "character_id": 103,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 15,
          "lv": 1,
          "exp": 0,
          "member_id": 1061100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 154,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 122,
          "hp": 214,
          "attack": 25,
          "magicattack": 30,
          "defense": 19,
          "magicdefence": 22,
          "agility": 69,
          "dexterity": 68,
          "luck": 67,
          "limit_break": 0,
          "character_id": 106,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 1,
          "lv": 1,
          "exp": 0,
          "member_id": 1063113,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 122,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 138,
          "hp": 237,
          "attack": 28,
          "magicattack": 34,
          "defense": 21,
          "magicdefence": 25,
          "agility": 70,
          "dexterity": 69,
          "luck": 66,
          "limit_break": 0,
          "character_id": 106,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 10,
          "lv": 3,
          "exp": 150,
          "member_id": 1064217,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 128,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 173,
          "hp": 270,
          "attack": 33,
          "magicattack": 41,
          "defense": 25,
          "magicdefence": 29,
          "agility": 69,
          "dexterity": 67,
          "luck": 68,
          "limit_break": 0,
          "character_id": 106,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 4,
          "lv": 1,
          "exp": 0,
          "member_id": 1083110,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 165,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 165,
          "hp": 292,
          "attack": 29,
          "magicattack": 34,
          "defense": 25,
          "magicdefence": 25,
          "agility": 61,
          "dexterity": 66,
          "luck": 63,
          "limit_break": 0,
          "character_id": 108,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 6,
          "lv": 1,
          "exp": 0,
          "member_id": 1093100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 170,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 105,
          "hp": 266,
          "attack": 30,
          "magicattack": 32,
          "defense": 22,
          "magicdefence": 24,
          "agility": 68,
          "dexterity": 67,
          "luck": 65,
          "limit_break": 0,
          "character_id": 109,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 5,
          "lv": 1,
          "exp": 0,
          "member_id": 1122100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 110,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 139,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 128,
          "hp": 282,
          "attack": 32,
          "magicattack": 24,
          "defense": 23,
          "magicdefence": 19,
          "agility": 71,
          "dexterity": 70,
          "luck": 62,
          "limit_break": 0,
          "character_id": 112,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 7,
          "lv": 1,
          "exp": 0,
          "member_id": 1132100,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 154,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 122,
          "hp": 247,
          "attack": 25,
          "magicattack": 31,
          "defense": 19,
          "magicdefence": 22,
          "agility": 69,
          "dexterity": 73,
          "luck": 73,
          "limit_break": 0,
          "character_id": 113,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 3,
          "lv": 1,
          "exp": 0,
          "member_id": 1152102,
          "ac_skill_lv_a": 1,
          "ac_skill_val_a": 100,
          "ac_skill_lv_b": 1,
          "ac_skill_val_b": 170,
          "ac_skill_lv_c": 1,
          "ac_skill_val_c": 152,
          "hp": 247,
          "attack": 27,
          "magicattack": 31,
          "defense": 21,
          "magicdefence": 23,
          "agility": 71,
          "dexterity": 74,
          "luck": 70,
          "limit_break": 0,
          "character_id": 115,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        },
        {
          "id": 9,
          "lv": 1,
          "exp": 0,
          "member_id": 1282100,
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
          "character_id": 128,
          "waiting_room": 0,
          "ex_flg": 0,
          "is_undead": 0
        }
      ],
      "weapons": [],
      "accessories": []
    }))),
    true,
  ))
}
