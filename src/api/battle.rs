use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::{ApiRequest, NotificationData};
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::notification::{IntoNotificationData, MissionDone};

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

// quest_id=101011
// party_no=1
// auto_progression_info={"is_start":false,"stop_setting":0,"incomplete_setting":0}
pub async fn battle_start(_request: ApiRequest) -> impl IntoHandlerResponse {
  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "chest": "10101111,10101120,10101131",
    "party": {
      "party_forms": [
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 1,
          "main": 11,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 2,
          "main": 12,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 3,
          "main": 10,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 4,
          "main": 0,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        },
        {
          "id": 666431194,
          "party_no": 1,
          "form_no": 5,
          "main": 0,
          "sub1": 0,
          "sub2": 0,
          "weapon": 0,
          "acc": 0,
          "skill_pa_fame": 0
        }
      ],
      "assist": 0,
      "sub_assists": [],
      "party_passive_skill": {
        "skill_id": 0,
        "user_member_id": 0
      }
    },
    "members": [
      {
        "id": 11,
        "lv": 1,
        "exp": 0,
        "member_id": 1001100,
        "ac_skill_id_a": 10000100,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 110,
        "ac_skill_id_b": 54000130,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 0,
        "ac_skill_id_c": 22000140,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 130,
        "hp": 245,
        "magicattack": 26,
        "defense": 20,
        "magicdefence": 19,
        "agility": 72,
        "dexterity": 78,
        "luck": 88,
        "limit_break": 0,
        "character_id": 100,
        "passiveskill": 210201,
        "specialattack": 100001,
        "resist_state": 210201,
        "resist_attr": 150000000,
        "attack": 27,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      },
      {
        "id": 12,
        "lv": 1,
        "exp": 0,
        "member_id": 1011100,
        "ac_skill_id_a": 10000100,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 110,
        "ac_skill_id_b": 31002024,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 170,
        "ac_skill_id_c": 12000146,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 152,
        "hp": 252,
        "magicattack": 31,
        "defense": 21,
        "magicdefence": 23,
        "agility": 66,
        "dexterity": 76,
        "luck": 10,
        "limit_break": 0,
        "character_id": 101,
        "passiveskill": 220001,
        "specialattack": 101001,
        "resist_state": 220001,
        "resist_attr": 155000000,
        "attack": 28,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      },
      {
        "id": 10,
        "lv": 1,
        "exp": 0,
        "member_id": 1064217,
        "ac_skill_id_a": 210000000000010000i64,
        "ac_skill_lv_a": 1,
        "ac_skill_val_a": 100,
        "ac_skill_id_b": 210042000000312082i64,
        "ac_skill_lv_b": 1,
        "ac_skill_val_b": 128,
        "ac_skill_id_c": 212200000000030074i64,
        "ac_skill_lv_c": 1,
        "ac_skill_val_c": 173,
        "hp": 247,
        "magicattack": 36,
        "defense": 22,
        "magicdefence": 26,
        "agility": 69,
        "dexterity": 67,
        "luck": 68,
        "limit_break": 0,
        "character_id": 106,
        "passiveskill": 212004,
        "specialattack": 106001,
        "resist_state": 212004,
        "resist_attr": 100005000,
        "attack": 29,
        "ex_flg": 0,
        "is_undead": 0,
        "special_skill_lv": 1
      }
    ]
  })));
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
