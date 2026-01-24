use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use anyhow::Context;
use rand::seq::IndexedMutRandom;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::api::dungeon::{PartyAccessory, PartyMember, PartyWeapon};
use crate::api::master_all::get_master_manager;
use crate::api::party_info::{party_info, Party};
use crate::api::{ApiRequest, MemberFameStats, NotificationData, RemoteDataItemType};
use crate::blob::{IntoRemoteData, UpdateMember};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::level::get_member_level_calculator;
use crate::member::{
  materialize_member_row, materialize_member_row_impl, FetchUserMemberSkillsIn, FetchUserMembersIn, Member, MemberActiveSkill,
  MemberPrototype, MemberStrength, OptionallyFetched,
};
use crate::user::session::Session;
use crate::AppState;

// See [Wonder_Api_PartymembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartymembersResponseDto {
  pub members: Vec<PartymembersMembersResponseDto>,
}

impl CallCustom for PartymembersResponseDto {}

// See [Wonder_Api_PartymembersMembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartymembersMembersResponseDto {
  pub user_member_id: i64,
  pub main: String,
  pub sub: String,
}

pub async fn party_members(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(PartymembersResponseDto {
    // No idea what to put here, it still displays member list
    members: vec![],
  }))
}

// body={"user_member_id": "1004100", "num2": "0", "num3": "0", "num1": "60"}
#[derive(Debug, Deserialize)]
pub struct GradeUpRequest {
  // [user_*] fields reference [unique_id], in our case it always same as master [member_id].
  pub user_member_id: i64,
  pub num1: i32,
  pub num2: i32,
  pub num3: i32,
}

pub async fn grade_up(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<GradeUpRequest>,
) -> impl IntoHandlerResponse {
  debug!(?params.user_member_id, "upgrading member");

  #[rustfmt::skip]
  let potions_to_use: BTreeMap<i64, i32> = BTreeMap::from([
    (1, params.num1),
    (2, params.num2),
    (3, params.num3)
  ]);

  // XP points are hardcoded in the game code,
  // see [Wonder.UI.Chara.LevelUpCell$$GetExpMaterialPoint]
  #[rustfmt::skip]
  let potion_to_xp: BTreeMap<i64, i32> = BTreeMap::from([
    (1, 300),
    (2, 1500),
    (3, 7500)
  ]);

  let total_xp: i32 = potions_to_use
    .iter()
    .map(|(potion_type, count)| potion_to_xp.get(potion_type).unwrap() * count)
    .sum();

  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;

  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      select member_id, xp, promotion_level
      from user_members
      where user_id = $1 and member_id = $2
      for update
    "#)
    .await
    .context("failed to prepare statement")?;

  let row = transaction
    .query_one(&statement, &[&session.user_id, &params.user_member_id])
    .await
    .context("failed to query user member")?;
  let member_id: i64 = row.get("member_id");
  let xp: i32 = row.get("xp");
  let promotion_level: i32 = row.get("promotion_level");
  // let active_skills: Value = row.get(3);

  let prototype = MemberPrototype::load_from_id(member_id);

  let calculator = get_member_level_calculator();
  let current_level = calculator.get_level(xp, prototype.rarity, promotion_level);
  let max_level = calculator.get_max_level(prototype.rarity, promotion_level);
  let max_xp = calculator.get_xp_for_level(max_level, prototype.rarity);

  // It clamps and the lost XP is not refunded (but technically can be, in 300 XP increments)
  let new_xp = (xp + total_xp).min(max_xp);
  let new_level = calculator.get_level(new_xp, prototype.rarity, promotion_level);
  info!(
    member_id,
    current_xp = ?xp,
    ?current_level,
    add_xp = ?total_xp,
    ?new_xp,
    ?new_level,
    ?max_xp,
    ?max_level,
    "upgrading member xp"
  );

  // update xp in database
  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      update user_members
      set xp = $3
      where user_id = $1 and member_id = $2
    "#)
    .await
    .context("failed to prepare statement")?;
  transaction
    .execute(&statement, &[&session.user_id, &params.user_member_id, &new_xp])
    .await
    .context("failed to update user member xp")?;
  debug!(member_id, ?new_xp, "updated member xp in database");

  let mut member = materialize_member_row_impl(member_id, new_xp, promotion_level, prototype);
  FetchUserMemberSkillsIn::new(&transaction)
    .await?
    .run(session.user_id, &mut [&mut member])
    .await?;

  // Update used items
  // TODO: This does not check if the user has enough potions
  let update = UpdateItemCountBy::new(&transaction).await?;
  let mut item_updates = Vec::new();
  for (item_id, count) in potions_to_use {
    if count > 0 {
      let item = update
        .run(session.user_id, (RemoteDataItemType::PowerPotion, item_id), -count)
        .await
        .context("failed to update item count")?;
      debug!(?item_id, ?count, new_quantity = ?item.quantity, "consumed power potions");

      item_updates.push(item.into_remote_data());
    }
  }

  transaction.commit().await.context("failed to commit transaction")?;

  // See [Wonder_Api_GradeupResponseDto_Fields]
  let mut response = CallResponse::new_success_empty();
  response
    .remote
    .extend(UpdateMember::new(member.to_member_parameter_wire()).into_remote_data());
  response.remote.extend(item_updates.into_iter().flatten());
  Ok(Unsigned(response))
}

// See [Wonder_Api_UpdatePartyFormResponseDto_Fields]
// See [Wonder_Api_PartyinfoResponseDto_Fields]
// See [Wonder_Api_PartyofferResponseDto_Fields]
// See [Wonder_Api_PartyresetResponseDto_Fields]
// See [Wonder_Api_PartychangeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyWire {
  pub party: Vec<Party>,
  pub members: Vec<PartyMember>,
  pub weapons: Vec<PartyWeapon>,
  pub accessories: Vec<PartyAccessory>,
}

impl CallCustom for PartyWire {}

// See [Wonder_Api_PartyFormInfoRequestDto_Fields]
#[derive(Debug, Deserialize)]
pub struct PartyFormInfoRequestDto {
  pub form_no: i32,
  pub main: i64,
  pub sub1: i64,
  pub sub2: i64,
  pub weapon: i64,
  pub acc: i64,
  pub special_skill: SpecialSkillInfoRequestDto,
  pub skill_pa_fame: i64,
}

// See [Wonder_Api_SpecialSkillInfoRequestDto_Fields]
#[derive(Debug, Deserialize)]
pub struct SpecialSkillInfoRequestDto {
  pub special_skill_id: i64,
  pub trial: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePartyFormRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub form_info: Vec<PartyFormInfoRequestDto>,
  pub is_fame_quest: bool,
  pub is_allow_trial: bool,
}

// party_no=1
// is_fame_quest=0
// is_allow_trial=1
// form_info=[{"form_no":1,"main":11,"sub1":10,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0},{"form_no":2,"main":12,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":101001,"trial":false},"skill_pa_fame":0},{"form_no":3,"main":13,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":102001,"trial":false},"skill_pa_fame":0},{"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0},{"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0}]
pub async fn update_party_form(
  state: Arc<AppState>,
  Params(params): Params<UpdatePartyFormRequest>,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  // TODO: I wonder if this is a good way to batch update multiple rows
  let client = state.get_database_client().await?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update user_party_forms
      set main_member_id = form_data.main_member_id,
          sub1_member_id = form_data.sub1_member_id,
          sub2_member_id = form_data.sub2_member_id,
          weapon_id = form_data.weapon_id,
          accessory_id = form_data.accessory_id,
          special_skill_id = form_data.special_skill_id
      from (
        select unnest($3::int8[]) as form_id,
               unnest($4::int8[]) as main_member_id,
               unnest($5::int8[]) as sub1_member_id,
               unnest($6::int8[]) as sub2_member_id,
               unnest($7::int8[]) as weapon_id,
               unnest($8::int8[]) as accessory_id,
               unnest($9::int8[]) as special_skill_id
      ) as form_data
      where user_party_forms.user_id = $1
        and user_party_forms.party_id = $2
        and user_party_forms.form_id = form_data.form_id
    "#)
    .await
    .context("failed to prepare statement")?;
  let rows_modified = client
    .execute(
      &statement,
      &[
        &session.user_id,
        &(params.party_id as i64),
        &params.form_info.iter().map(|f| f.form_no as i64).collect::<Vec<_>>(),
        &params.form_info.iter().map(|f| f.main).collect::<Vec<_>>(),
        &params.form_info.iter().map(|f| f.sub1).collect::<Vec<_>>(),
        &params.form_info.iter().map(|f| f.sub2).collect::<Vec<_>>(),
        &params.form_info.iter().map(|f| f.weapon).collect::<Vec<_>>(),
        &params.form_info.iter().map(|f| f.acc).collect::<Vec<_>>(),
        &params
          .form_info
          .iter()
          .map(|f| f.special_skill.special_skill_id)
          .collect::<Vec<_>>(),
      ],
    )
    .await
    .context("failed to execute query")?;
  info!(?params.party_id, ?rows_modified, "updated party forms");

  // Response is identical to party_info
  Ok(party_info(state, session).await)
}

// trial=1
// weapon_priority_status=attack
// assist=1
// priority_status=strength
// skill_pa_fame=1
// equip=1
// main=1
// sub=1
// party_no=1
// accessory_priority_resistances=["none","none"]
// elemental=["none","none"]
// is_fame_quest=0
/// "Suggest party" button
pub async fn party_offer(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_offer");

  // Response is identical to party_info
  Ok(party_info(state, session).await)
}

// party_no=1
// is_allow_trial=1
// is_fame_quest=0
/// "Tool" -> "Reset" button
pub async fn party_reset(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_reset");

  // Response is identical to party_info
  Ok(party_info(state, session).await)
}

// See [Wonder_Api_PartychangelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartychangelistResponseDto {
  pub members: Vec<ChangePartyMember>,
  pub weapons: Vec<ChangePartyWeapon>,
  pub accessories: Vec<PartyAccessory>,
  pub assists: Vec<ChangePartyAssist>,
}

impl CallCustom for PartychangelistResponseDto {}

// See [Wonder_Api_PartychangelistMembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ChangePartyMember {
  pub id: i64,
  pub lv: i32,
  pub member_id: i64,
  pub character_id: i64,
}

// See [Wonder_Api_PartychangelistWeaponsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ChangePartyWeapon {
  pub id: i64,
  pub weapon_id: i64,
}

// See [Wonder_Api_PartychangelistAssistsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ChangePartyAssist {
  pub id: i64,
}

// update_type=assist
pub async fn party_change_list(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Unsigned(PartychangelistResponseDto {
    members: vec![],
    weapons: vec![],
    accessories: vec![],
    assists: vec![],
  }))
}

#[derive(Debug, Deserialize)]
pub struct PartyNameSetRequest {
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  #[serde(rename = "party_no")]
  pub party_id: i32,
}

/// CLIENT BUG: Party name is not updated in UI (near "Formed party list" button) until the next
/// request that fetches party info.
pub async fn party_name_set(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<PartyNameSetRequest>,
) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update user_parties
      set name = $3
      where user_id = $1 and party_id = $2
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .query(&statement, &[&session.user_id, &(params.party_id as i64), &params.name])
    .await
    .context("failed to execute query")?;

  // See [Wonder_Api_PartynamesetResponseDto_Fields]
  Ok(Unsigned(()))
}

// form_no=0
// party_no=1
// is_fame_quest=0
// is_allow_trial=0
// unique_id=50000010
// update_type=party_passive_skill
// trial=0
pub async fn party_change(
  state: Arc<AppState>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_change");

  // Response is identical to party_info
  Ok(party_info(state, session).await)
}

// See [Wonder_Api_PartyStrengthResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyStrengthResponseDto {
  pub strength: i32,
}

impl CallCustom for PartyStrengthResponseDto {}

// is_fame_quest=0
// party_no=1
// form_info=[{"form_no":1,"main":11,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0},{"form_no":2,"main":12,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":101001,"trial":false},"skill_pa_fame":0},{"form_no":3,"main":13,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":102001,"trial":false},"skill_pa_fame":0},{"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0},{"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0}]
// is_allow_trial=1
pub async fn party_strength(request: ApiRequest) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_strength");

  Ok(Unsigned(PartyStrengthResponseDto { strength: 1000 }))
}

// See [Wonder_Api_LimitbreakResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct LimitBreakResponse {
  pub newlv: i32,
}

impl CallCustom for LimitBreakResponse {}

// body={"break_count": "1", "target_card_id": "1004116"}
#[derive(Debug, Deserialize)]
pub struct LimitBreakRequest {
  #[serde(rename = "target_card_id")]
  pub member_id: i64,
  #[serde(rename = "break_count")]
  pub levels: i32,
}

pub async fn limit_break(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<LimitBreakRequest>,
) -> impl IntoHandlerResponse {
  debug!(?params, ?params, "promote member");

  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;
  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      update user_members
      set promotion_level = promotion_level + $3
      where user_id = $1 and member_id = $2
      returning member_id, xp, promotion_level
    "#)
    .await
    .context("failed to prepare statement")?;
  let row = transaction
    .query(&statement, &[&session.user_id, &params.member_id, &params.levels])
    .await
    .context("failed to update user member promotion level")?
    .into_iter()
    .next()
    .context("no such member")?;

  // TODO: Consume items
  transaction.commit().await.context("failed to commit transaction")?;

  let mut member = materialize_member_row(row);
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut [&mut member])
    .await?;

  let mut response = CallResponse::new_success(Box::new(LimitBreakResponse {
    newlv: member.promotion_level,
  }));
  response
    .remote
    .extend(UpdateMember::new(member.to_member_parameter_wire()).into_remote_data());
  Ok(Unsigned(response))
}

// See [Wonder_Api_MemberskillupResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MemberSkillUpResponse {
  #[serde(rename = "skilllvs")]
  pub skill_levels: Vec<MemberSkillLevelUp>,
}

impl CallCustom for MemberSkillUpResponse {}

// See [Wonder_Api_MemberskillupSkilllvsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MemberSkillLevelUp {
  pub skill: i32,
  pub lvup: i32,
}

// See [Wonder_Api_MemberskillupRequest_Fields]
// body={"user_member_id": "1004100", "type": "0", "num": "1"}
#[derive(Debug, Deserialize)]
pub struct MemberSkillUpRequest {
  // [user_*] fields reference [unique_id], in our case it always same as master [member_id].
  pub user_member_id: i64,
  // 0 - reserve members (3x), 1 - skill potions (1x)
  #[serde(rename = "type")]
  pub kind: i32,
  #[serde(rename = "num")]
  pub amount: i32,
}

// Reference: https://youtu.be/2zcAVWr9u4k
pub async fn member_skill_up(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MemberSkillUpRequest>,
) -> impl IntoHandlerResponse {
  let rolls = params.amount
    * match params.kind {
      0 => 3, // reserve members
      1 => 1, // skill potions
      _ => todo!("unknown member skill up kind {}", params.kind),
    };

  let mut client = state.get_database_client().await?;
  let fetch_members = FetchUserMembersIn::new(&client).await?;
  let mut member = fetch_members
    .run(session.user_id, &[params.user_member_id])
    .await?
    .into_iter()
    .next()
    .context("member not found")?;
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut [&mut member])
    .await?;

  let skills = match &mut member.active_skills {
    OptionallyFetched::Fetched(skills) => skills,
    OptionallyFetched::Unfetched => panic!("active skills not fetched for member {}", member.id),
  };

  let mut transaction = client.transaction().await.context("failed to start transaction")?;
  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      update user_member_skills
      set level = $4
      where user_id = $1 and member_id = $2 and skill_id = $3
    "#)
    .await
    .context("failed to prepare statement")?;

  // Take random skill with level < 5 and upgrade it
  let mut upgrades = HashMap::new();
  for roll in 0..rolls {
    let mut upgradable_skills = skills.iter_mut().flatten().filter(|s| s.level < 5).collect::<Vec<_>>();
    if upgradable_skills.is_empty() {
      debug!(?roll, "no more upgradable skills for member {}", member.id);
      break;
    }

    let skill_to_upgrade = upgradable_skills.choose_mut(&mut rand::rng()).unwrap();
    skill_to_upgrade.level += 1;
    upgrades
      .entry(skill_to_upgrade.prototype.id)
      .and_modify(|new_levels| *new_levels += 1)
      .or_insert(1);
    transaction
      .execute(
        &statement,
        &[
          &session.user_id,
          &(member.id as i64),
          &skill_to_upgrade.prototype.id,
          &skill_to_upgrade.level,
        ],
      )
      .await
      .context("failed to update member skill level")?;

    debug!(
      ?roll,
      skill_id = ?skill_to_upgrade.prototype.id,
      new_level = ?skill_to_upgrade.level,
      "upgraded member skill"
    );
  }

  transaction.commit().await.context("failed to commit transaction")?;

  // Map skill IDs to client indices, 1 is rightmost skill, 3 is leftmost.
  // Indices are same as in Member.active_skills vector.
  // TODO: This is some weird code, should refactor later...
  let skill_levels: Vec<MemberSkillLevelUp> = skills
    .iter()
    .enumerate()
    .map(|(i, skill_opt)| {
      let skill = skill_opt.as_ref().unwrap();
      MemberSkillLevelUp {
        skill: (skills.len() - i) as i32,
        lvup: *upgrades.get(&skill.prototype.id).unwrap_or(&0),
      }
    })
    .filter(|s| s.lvup > 0)
    .collect();

  let mut response = CallResponse::new_success(Box::new(MemberSkillUpResponse { skill_levels }));
  response.add_remote_data(UpdateMember::new(member.to_member_parameter_wire()).into_remote_data());

  Ok(Unsigned(response))
}
