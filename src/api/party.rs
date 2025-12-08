use std::sync::Arc;

use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::api::dungeon::{PartyAccessory, PartyMember, PartyWeapon};
use crate::api::party_info::{party_info, Party};
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};
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

// num3=2
// num2=2
// num1=1
// user_member_id=10
pub async fn grade_up(_request: ApiRequest) -> impl IntoHandlerResponse {
  // See [Wonder_Api_GradeupResponseDto_Fields]
  // TODO: This probably should send remote data to update member: level in UI rolls back after animation
  Ok(Unsigned(()))
}

// See [Wonder_Api_UpdatePartyFormResponseDto_Fields]
// See [Wonder_Api_PartyinfoResponseDto_Fields]
// See [Wonder_Api_PartyofferResponseDto_Fields]
// See [Wonder_Api_PartyresetResponseDto_Fields]
// See [Wonder_Api_PartychangeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyForm {
  pub party: Party,
  pub members: PartyMember,
  pub weapons: PartyWeapon,
  pub accessories: PartyAccessory,
}

impl CallCustom for PartyForm {}

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

// party_no=1
// is_fame_quest=0
// is_allow_trial=1
// form_info=[{"form_no":1,"main":11,"sub1":10,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":100001,"trial":false},"skill_pa_fame":0},{"form_no":2,"main":12,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":101001,"trial":false},"skill_pa_fame":0},{"form_no":3,"main":13,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":102001,"trial":false},"skill_pa_fame":0},{"form_no":4,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0},{"form_no":5,"main":0,"sub1":0,"sub2":0,"weapon":0,"acc":0,"special_skill":{"special_skill_id":0,"trial":false},"skill_pa_fame":0}]
pub async fn update_party_form(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();
  let form_info: Vec<PartyFormInfoRequestDto> = serde_json::from_str(&request.body["form_info"]).unwrap();

  warn!(?party_no, "encountered stub: update_party_form");

  // Response is identical to party_info
  Ok(party_info(session).await)
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
pub async fn party_offer(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_offer");

  // Response is identical to party_info
  Ok(party_info(session).await)
}

// party_no=1
// is_allow_trial=1
// is_fame_quest=0
/// "Tool" -> "Reset" button
pub async fn party_reset(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_reset");

  // Response is identical to party_info
  Ok(party_info(session).await)
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

// name=R09WTk8
// party_no=2
pub async fn party_name_set(_request: ApiRequest) -> impl IntoHandlerResponse {
  let name = &_request.body["name"];
  let name = BASE64_STANDARD_NO_PAD
    .decode(name)
    .context("failed to decode 'name' from base64")?;
  let name = String::from_utf8(name).context("name is not valid UTF-8")?;
  let party_no: i32 = _request.body["party_no"].parse().unwrap();

  warn!(?party_no, ?name, "encountered stub: party_name_set");

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
pub async fn party_change(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let party_no: i32 = request.body["party_no"].parse().unwrap();

  warn!(?party_no, "encountered stub: party_change");

  // Response is identical to party_info
  Ok(party_info(session).await)
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
