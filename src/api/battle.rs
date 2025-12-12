use crate::api::party_info::{Party, PartyForm, PartyPassiveSkillInfo, SpecialSkillInfo};
use crate::api::surprise::BasicBattlePartyForm;
use crate::api::{ApiRequest, MemberFameStats, NotificationData};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::member::{Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::notification::{IntoNotificationData, MissionDone};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

// See [Wonder_Api_BattlestartMembersResponseDto_Fields]
// See [Wonder_Api_SurpriseQuestStartMembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleMember {
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
  pub ex_flg: i32,
  pub is_undead: i32,
  pub special_skill_lv: i32,
}

// See [Wonder_Api_BattlestartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleStartResponse {
  pub chest: String,
  pub party: BattleParty,
  pub members: Vec<BattleMember>,
}

impl CallCustom for BattleStartResponse {}

// See [Wonder_Api_BattlestartPartyResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleParty {
  pub party_forms: [BasicBattlePartyForm; 5],
  pub assist: i32,
  pub sub_assists: Vec<i32>,
  pub party_passive_skill: PartyPassiveSkillInfo,
}

#[derive(Debug, Deserialize)]
pub struct BattleStartRequest {
  pub quest_id: i32,
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub auto_progression_info: AutoProgressionInfo,
}

#[derive(Debug, Deserialize)]
pub struct AutoProgressionInfo {
  pub is_start: bool,
  pub stop_setting: i32,
  pub incomplete_setting: i32,
}

// quest_id=101011
// party_no=1
// auto_progression_info={"is_start":false,"stop_setting":0,"incomplete_setting":0}
pub async fn battle_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleStartRequest>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select
        member_id,
        xp,
        promotion_level
      from user_members
      where user_id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let members = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;

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
        upf.accessory_id
      from user_parties up
        join user_party_forms upf
          on up.user_id = upf.user_id and up.party_id = upf.party_id
      where up.user_id = $1 and up.party_id = $2
      order by upf.form_id
    "#,
    )
    .await
    .context("failed to prepare statement")?;
  let forms = client
    .query(&statement, &[&session.user_id, &(params.party_id as i64)])
    .await
    .context("failed to execute query")?;
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

      PartyForm {
        id: form_id as i32,
        form_no: form_id as i32,
        party_no: params.party_id,
        main: main_member_id as i32,
        sub1: sub1_member_id as i32,
        sub2: sub2_member_id as i32,
        weapon: weapon_id,
        acc: accessory_id,
        name: party_name,
        strength: 123,
        specialskill: SpecialSkillInfo {
          special_skill_id: 100001,
          trial: false,
        },
        skill_pa_fame: 0,
      }
    })
    .collect::<Vec<_>>()
    .try_into()
    .unwrap();

  let party = Party::new(forms, params.party_id);

  let members = members
    .iter()
    .enumerate()
    .map(|(index, row)| {
      let member_id: i64 = row.get(0);
      let xp: i32 = row.get(1);
      let promotion_level: i32 = row.get(2);
      // let active_skills: Value = row.get(3);
      let prototype = MemberPrototype::load_from_id(member_id);

      Member {
        id: prototype.id as i32,
        prototype: &prototype,
        xp,
        promotion_level,
        active_skills: prototype
          .active_skills
          .iter()
          .map(|skill_opt| {
            skill_opt.as_ref().map(|skill| MemberActiveSkill {
              prototype: skill,
              level: 1,
              value: skill.value.max,
            })
          })
          .collect::<Vec<_>>()
          .try_into()
          .unwrap(),
        // active_skills: prototype
        //   .active_skills
        //   .iter()
        //   .enumerate()
        //   .map(|(index, prototype)| {
        //     // TODO: Wrong
        //     let active_skill = active_skills.get(index).unwrap();
        //     // let skill_id = active_skill["id"].as_i64().unwrap();
        //     let level = active_skill["level"].as_i64().unwrap() as i32;
        //     let value = active_skill["value"].as_i64().unwrap() as i32;
        //     Some(MemberActiveSkill {
        //       prototype: &prototype,
        //       level,
        //       value,
        //     })
        //   })
        //   .try_into()
        //   .unwrap(),
        stats: prototype.stats.clone(),
        main_strength: MemberStrength::default(),
        sub_strength: MemberStrength::default(),
        sub_strength_bonus: MemberStrength::default(),
        fame_stats: MemberFameStats::default(),
        skill_pa_fame_list: vec![],
      }
      .to_battle_member()
    })
    .collect::<Vec<_>>();

  let mut response = CallResponse::new_success(Box::new(BattleStartResponse {
    chest: "10101111,10101120,10101131".to_owned(),
    party: party.to_battle_party(),
    // We must send only members that are used in the party, otherwise hardlock occurs
    members: members
      .into_iter()
      .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
      .collect(),
  }));
  response.add_notifications(vec![NotificationData::new(1, 7, 6, 0, "".to_string(), "".to_string())]);
  Ok(Unsigned(response))
}

#[derive(Debug, Deserialize)]
pub struct LiveMember {
  pub id: i64,
  pub hp: i32,
  pub form_no: i32,
}

#[derive(Debug, Deserialize)]
pub struct ResumeInfo {
  pub resumeMembers: Vec<ResumeMember>,
  pub assistRemainCount: i32,
  pub assistCoolTime: i32,
  pub reportInfo: ReportInfo,
}

// See [Wonder_Battle_ResumeReportInfo_Fields]
#[derive(Debug, Deserialize)]
pub struct ReportInfo {
  pub PendingBonusIds: Vec<i32>,
  pub SuccessBonusIds: Vec<i32>,
}

// See [Wonder_Battle_ResumeMember_Fields]
#[derive(Debug, Deserialize)]
pub struct ResumeMember {
  pub memberId: i64,
  pub spLevel: f32,
  pub skill1Time: i32,
  pub skill2Time: i32,
  pub stateInfoArray: Vec<StateInfo>,
  pub CurrentAgiLevel: f32,
  pub CurrentAgiLevelSecondary: f32,
  pub PassiveInfoArray: Vec<ResumePassiveInfo>,
}

// See [Wonder_Battle_StateInfo_Fields]
#[derive(Debug, Deserialize)]
pub struct StateInfo {
  pub skillId: i64,
  pub AffectValue: i32,
  pub SecondaryAffectValue: i32,
  pub TertiaryAffectValue: i32,
  pub Accuracy: i32,
  pub Attr: String,
  pub TargetNum: i32,
  pub Type: String,
  pub AffectTurn: i32,
  pub AffectTime: f32,
  pub Target: String,
  pub ShakeMode: String,
  pub EffectSize: i32,
  pub BuffRank: i32,
  pub IsBlow: bool,
  pub IsWater: bool,
  pub PassedTime: f32,
  pub AdditionalTime: f32,
  pub Remain: i32,
  pub IntervalTime: f32,
  pub UnitId: i32,
}

// See [Wonder_Battle_ResumePassiveInfo_Fields]
#[derive(Debug, Deserialize)]
pub struct ResumePassiveInfo {
  pub SkillId: i64,
  pub ExecuteCount: i32,
  pub NeedCount: i32,
  pub RemainCount: i32,
  pub PassiveCount: i32,
  pub MaxStackCount: i32,
  pub StackCount: i32,
}

// wave=2
// livemembers=[{"id":11,"hp":245,"form_no":1},{"id":12,"hp":252,"form_no":2},{"id":10,"hp":208,"form_no":3}]
// battletime=12
// resume_info={"resumeMembers":[{"memberId":1001100,"spLevel":0.4000000059604645,"skill1Time":0,"skill2Time":0,"stateInfoArray":[],"CurrentAgiLevel":10023.0,"CurrentAgiLevelSecondary":10008.0,"PassiveInfoArray":[]},{"memberId":1011100,"spLevel":0.5,"skill1Time":0,"skill2Time":0,"stateInfoArray":[],"CurrentAgiLevel":10032.0,"CurrentAgiLevelSecondary":5742.0,"PassiveInfoArray":[]},{"memberId":1064217,"spLevel":0.6000000238418579,"skill1Time":18,"skill2Time":0,"stateInfoArray":[],"CurrentAgiLevel":10005.0,"CurrentAgiLevelSecondary":34.5,"PassiveInfoArray":[]}],"assistRemainCount":0,"assistCoolTime":0,"reportInfo":{"PendingBonusIds":[],"SuccessBonusIds":[]}}
pub async fn battle_wave_result(request: ApiRequest) -> impl IntoHandlerResponse {
  let wave: i32 = request.body["wave"].parse().unwrap();
  let live_members: Vec<LiveMember> = serde_json::from_str(&request.body["livemembers"]).unwrap();
  let battle_time: i64 = request.body["battletime"].parse().unwrap();
  let resume_info: ResumeInfo = serde_json::from_str(&request.body["resume_info"]).unwrap();

  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "chest": 10101120
  })));
  Ok(Unsigned(response))
}

// quest_id=101011
// party_no=1
// win=1
// wave=3
// clearquestmission=[12,13,15]
// auto_progression_stop=1
// memcheckcount=0
pub async fn battle_result(request: ApiRequest) -> impl IntoHandlerResponse {
  let quest_id: i32 = request.body["quest_id"].parse().unwrap();
  let party_no: i32 = request.body["party_no"].parse().unwrap();
  let win: i32 = request.body["win"].parse().unwrap();
  let wave: i32 = request.body["wave"].parse().unwrap();
  let clear_quest_mission: Vec<i32> = serde_json::from_str(&request.body["clearquestmission"]).unwrap();
  let auto_progression_stop: i32 = request.body["auto_progression_stop"].parse().unwrap();
  let mem_check_count: i32 = request.body["memcheckcount"].parse().unwrap();

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "limit": 0,
    "exp": 5,
    "lvup": 0,
    "money": 720,
    "storyunlock": [],
    "love": [
      {
        "character_id": 100,
        "love": 4
      },
      {
        "character_id": 101,
        "love": 4
      },
      {
        "character_id": 106,
        "love": 4
      }
    ],
    "member_exp": [
      {
        "member_id": 1001100,
        "exp": 150
      },
      {
        "member_id": 1011100,
        "exp": 150
      },
      {
        "member_id": 1064217,
        "exp": 150
      }
    ],
    "mission": [
      1,
      1,
      1
    ],
    "reward": [
      {
        "itemtype": 15,
        "itemid": 5001,
        "itemnum": 4,
        "is_rare": 0
      },
      {
        "itemtype": 18,
        "itemid": 1,
        "itemnum": 1,
        "is_rare": 0
      },
      {
        "itemtype": 16,
        "itemid": 151,
        "itemnum": 1,
        "is_rare": 0
      },
      {
        "itemtype": 18,
        "itemid": 2,
        "itemnum": 2,
        "is_rare": 0
      },
      {
        "itemtype": 27,
        "itemid": 230831,
        "itemnum": 3,
        "is_rare": 0
      }
    ],
    "clearreward": [
      {
        "itemtype": 15,
        "itemid": 1100,
        "itemnum": 3,
        "mission": 1
      },
      {
        "itemtype": 4,
        "itemid": 1061100,
        "itemnum": 1,
        "mission": 1
      },
      {
        "itemtype": 3,
        "itemid": 1,
        "itemnum": 50,
        "mission": 3
      }
    ],
    "firstclear": true,
    "auto_progression_result": {
      "auto_count": 0,
      "is_continue": false,
      "stop_reason": 0,
      "stamina_all": 0,
      "reward_all": [],
      "clearreward_all": []
    }
  })));
  // TODO: Send remote data to actually update characters stats
  response.add_notifications(vec![
    NotificationData::new(1, 16, 1, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 22, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 20, 1, "".to_string(), "".to_string()),
    MissionDone::new(11210001).into_notification_data(),
    NotificationData::new(1, 7, 3, 2, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 13, 7, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 34, 1, "show_button".to_string(), "".to_string()),
    NotificationData::new(1, 6, 1, 30030001, "".to_string(), "".to_string()),
    NotificationData::new(1, 10, 230731, 52307325, "".to_string(), "".to_string()),
    NotificationData::new(1, 10, 230831, 52308305, "".to_string(), "".to_string()),
  ]);

  Ok(Unsigned(response))
}

// See [Wonder_Api_BattleretireResponseDto_Fields]
pub async fn battle_retire(request: ApiRequest) -> impl IntoHandlerResponse {
  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({})));
  Ok(Unsigned(response))
}
