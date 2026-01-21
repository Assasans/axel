use crate::api::battle_multi::{BattleCharacterLove, BattleClearReward, BattleMemberExp};
use crate::api::master_all::get_master_manager;
use crate::api::party_info::{Party, PartyForm, PartyPassiveSkillInfo, SpecialSkillInfo};
use crate::api::quest_hunting::{extract_items, BattleReward};
use crate::api::surprise::BasicBattlePartyForm;
use crate::api::{ApiRequest, MemberFameStats, NotificationData, RemoteDataItemType};
use crate::blob::IntoRemoteData;
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::member::{FetchUserMembersIn, Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::notification::{IntoNotificationData, MissionDone};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

// See [Wonder_Api_BattlestartMembersResponseDto_Fields]
// See [Wonder_Api_SurpriseQuestStartMembersResponseDto_Fields]
// See [Wonder_Api_ScorechallengestartMembersResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStartMembersResponseDto_Fields]
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
// See [Wonder_Api_ScorechallengestartPartyResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStartPartyResponseDto_Fields]
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

pub async fn make_battle_start(state: &AppState, session: &Session, party_id: i32) -> impl IntoHandlerResponse + use<> {
  let client = state
    .get_database_client()
    .await
    .context("failed to get database connection")?;

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
      where up.user_id = $1 and up.party_id = $2
      order by upf.form_id
    "#,
    )
    .await
    .context("failed to prepare statement")?;
  let forms = client
    .query(&statement, &[&session.user_id, &(party_id as i64)])
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
        strength: 123,
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

  let party = Party::new(forms, party_id);

  // We must send only members that are used in the party, otherwise hardlock occurs
  let fetch_members = FetchUserMembersIn::new(&client).await.unwrap();
  #[rustfmt::skip]
  let members = fetch_members.run(
    session.user_id,
    &party.party_forms.iter().map(|form| form.main as i64).collect::<Vec<_>>(),
  ).await.unwrap();

  let mut response = CallResponse::new_success(Box::new(BattleStartResponse {
    chest: "10101111,10101120,10101131".to_owned(),
    party: party.to_battle_party(),
    members: members
      .into_iter()
      .map(|member| {
        let form = party.party_forms.iter().find(|form| form.main == member.id).unwrap();
        member.to_battle_member(form)
      })
      .collect(),
  }));
  response.add_notifications(vec![NotificationData::new(1, 7, 6, 0, "".to_string(), "".to_string())]);
  Ok(Unsigned(response))
}

// quest_id=101011
// party_no=1
// auto_progression_info={"is_start":false,"stop_setting":0,"incomplete_setting":0}
pub async fn battle_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleStartRequest>,
) -> impl IntoHandlerResponse {
  make_battle_start(&state, &session, params.party_id).await
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

// See [Wonder_Api_ResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleResultResponse {
  pub limit: i32,
  pub exp: i32,
  pub lvup: i32,
  pub money: i32,
  pub storyunlock: Vec<i32>,
  /// Must contain all characters used in the battle
  pub love: Vec<BattleCharacterLove>,
  /// Must contain all members used in the battle
  pub member_exp: Vec<BattleMemberExp>,
  pub mission: Vec<i32>,
  pub reward: Vec<BattleReward>,
  pub clearreward: Vec<BattleClearReward>,
  pub auto_progression_result: AutoProgressionResultResponse,
  pub firstclear: bool,
}

impl CallCustom for BattleResultResponse {}

#[derive(Debug, Serialize)]
pub struct AutoProgressionResultResponse {
  pub auto_count: i32,
  pub is_continue: bool,
  pub stop_reason: i32,
  pub stamina_all: i32,
  pub reward_all: Vec<BattleReward>,
  pub clearreward_all: Vec<BattleClearReward>,
}

// body={"wave": "3", "party_no": "1", "win": "1", "clearquestmission": "[12,0,0]", "memcheckcount": "0", "quest_id": "104041", "auto_progression_stop": "1"}
#[derive(Debug, Deserialize)]
pub struct BattleResultRequest {
  pub quest_id: i32,
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub win: i32,
  pub wave: i32,
  pub clearquestmission: Vec<i32>,
  pub auto_progression_stop: i32,
  pub memcheckcount: i32,
}

pub async fn battle_result(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleResultRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_result");

  let rewards = get_master_manager()
    .get_master("mainquest_stage_itemreward")
    .into_iter()
    .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
    .collect::<HashMap<_, _>>();
  let mut rewards = extract_items(&rewards[&params.quest_id]);
  for item in &mut rewards {
    item.item_num *= 20;
  }

  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;
  let update = UpdateItemCountBy::new(&transaction).await?;
  let mut update_items = Vec::new();
  for item in &rewards {
    let item = update
      .run(
        session.user_id,
        (RemoteDataItemType::from(item.item_type), item.item_id),
        item.item_num,
      )
      .await
      .context("failed to execute query")?;
    debug!(?item, "granted main quest reward");

    update_items.push(item.into_remote_data());
  }
  transaction.commit().await.context("failed to commit transaction")?;

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
      let special_skill_id: i64 = row.get(8);

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
          special_skill_id: special_skill_id as i32,
          trial: false,
        },
        skill_pa_fame: 0,
      }
    })
    .collect::<Vec<_>>()
    .try_into()
    .unwrap();

  let party = Party::new(forms, params.party_id);

  // We must send only members that are used in the party, otherwise hardlock occurs
  let fetch_members = FetchUserMembersIn::new(&client).await.unwrap();
  #[rustfmt::skip]
  let members = fetch_members.run(
    session.user_id,
    &party.party_forms.iter().map(|form| form.main as i64).collect::<Vec<_>>(),
  ).await.unwrap();

  let characters = party
    .party_forms
    .iter()
    .map(|form| {
      vec![
        members
          .iter()
          .find(|member| member.id == form.main)
          .map(|m| m.prototype.character_id),
        members
          .iter()
          .find(|member| member.id == form.sub1)
          .map(|m| m.prototype.character_id),
        members
          .iter()
          .find(|member| member.id == form.sub2)
          .map(|m| m.prototype.character_id),
      ]
    })
    .flatten()
    .flatten()
    .collect::<Vec<_>>();
  let characters = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select
          character_id,
          intimacy
        from user_characters
        where user_id = $1
      "#)
      .await
      .context("failed to prepare statement")?;
    let rows = client
      .query(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")?;
    rows
      .iter()
      .map(|row| {
        let character_id: i64 = row.get(0);
        let intimacy: i32 = row.get(1);
        (character_id, intimacy)
      })
      .filter(|(id, _)| characters.contains(id))
      .collect::<HashMap<_, _>>()
  };

  let mut response = CallResponse::new_success(Box::new(BattleResultResponse {
    limit: 0,
    exp: 5,
    lvup: 0,
    money: 85000,
    storyunlock: vec![],
    love: characters
      .iter()
      .map(|(character_id, intimacy)| BattleCharacterLove {
        character_id: *character_id,
        love: *intimacy + 10,
      })
      .collect(),
    member_exp: members
      .iter()
      .map(|member| BattleMemberExp {
        member_id: member.id as i64,
        exp: 230,
      })
      .collect(),
    mission: params.clearquestmission,
    reward: rewards
      .iter()
      .map(|item| BattleReward {
        itemtype: item.item_type,
        itemid: item.item_id,
        itemnum: item.item_num,
        is_rare: item.item_rare,
      })
      .collect(),
    clearreward: vec![],
    auto_progression_result: AutoProgressionResultResponse {
      auto_count: 0,
      is_continue: false,
      stop_reason: 0,
      stamina_all: 0,
      reward_all: vec![],
      clearreward_all: vec![],
    },
    firstclear: true,
  }));
  response.remote.extend(update_items.into_iter().flatten());
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
