use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::AppState;
use crate::api::dungeon::{PartyAccessory, PartyMember, PartyWeapon};
use crate::api::party_info::{Party, party_info};
use crate::api::{ApiRequest, MemberFameStats, RemoteData, RemoteDataItemType};
use crate::blob::{IntoRemoteData, UpdateItem, UpdateMember};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::level::get_member_level_calculator;
use crate::member::{Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;

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
  let member_id: i64 = row.get(0);
  let xp: i32 = row.get(1);
  let promotion_level: i32 = row.get(2);
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

  let member = Member {
    id: prototype.id as i32,
    prototype: &prototype,
    xp: new_xp,
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
    stats: prototype.stats.clone(),
    main_strength: MemberStrength::default(),
    sub_strength: MemberStrength::default(),
    sub_strength_bonus: MemberStrength::default(),
    fame_stats: MemberFameStats::default(),
    skill_pa_fame_list: vec![],
  };

  // Update used items
  // TODO: This does not check if the user has enough potions
  // TODO: Maybe some UpdateDatabaseItem struct would be cleaner here
  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      update user_items
      set quantity = quantity - $3
      where user_id = $1 and item_id = $2
      returning item_type, item_id, quantity
    "#)
    .await
    .context("failed to prepare statement")?;
  let mut item_updates = Vec::new();
  for (item_id, count) in potions_to_use {
    if count > 0 {
      let rows = transaction
        .query(&statement, &[&session.user_id, &item_id, &count])
        .await
        .context("failed to update user items")?;
      let row = rows.first().unwrap();
      let item_type: i64 = row.get(0);
      let item_id: i64 = row.get(1);
      let quantity: i32 = row.get(2);
      debug!(?item_id, ?item_type, ?quantity, "consumed power potions");

      let item_update =
        UpdateItem::new(RemoteDataItemType::from(item_type as i32), 0, item_id, quantity).into_remote_data();
      item_updates.push(item_update);
    }
  }

  transaction.commit().await.context("failed to commit transaction")?;

  // See [Wonder_Api_GradeupResponseDto_Fields]
  // TODO: This probably should send remote data to update member: level in UI rolls back after animation
  let mut response = CallResponse::new_success_empty();
  response
    .remote
    .extend([UpdateMember::new(member.to_member_parameter_wire()).into_remote_data()]);
  response.remote.extend(item_updates);
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
          accessory_id = form_data.accessory_id
      from (
        select unnest($3::int8[]) as form_id,
               unnest($4::int8[]) as main_member_id,
               unnest($5::int8[]) as sub1_member_id,
               unnest($6::int8[]) as sub2_member_id,
               unnest($7::int8[]) as weapon_id,
               unnest($8::int8[]) as accessory_id
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
