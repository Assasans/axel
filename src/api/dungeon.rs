//! Reference: https://youtu.be/5xRIW8bDzc4, https://youtu.be/ViMtYimwca4

use crate::api::battle::{BattleParty, LiveMember};
use crate::api::master_all::get_master_manager;
use crate::api::party_info::{PartyPassiveSkillInfo, SpecialSkillInfo};
use crate::api::surprise::BasicBattlePartyForm;
use crate::api::{MemberFameStats, RemoteDataItemType};
use crate::blob::{AddItem, IntoRemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::member::{FetchUserMemberSkillsIn, FetchUserMembers, FetchUserMembersIn, Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use jwt_simple::prelude::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use tracing::warn;

// See [Wonder_Api_DungeonStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStatusResponse {
  pub area_id: i32,
}

impl CallCustom for DungeonStatusResponse {}

pub async fn dungeon_status() -> impl IntoHandlerResponse {
  Ok(Unsigned(CallResponse::new_success(Box::new(DungeonStatusResponse {
    area_id: 0,
  }))))
}

// See [Wonder_Api_DungeonAreaTopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaTopResponse {
  pub is_practice: bool,
  pub stage_state: DungeonStageState,
  pub party_set: DungeonPartySet,
  pub clear_info: DungeonAreaClearInfo,
  pub unchoosed_benefit_id_list: Vec<i32>,
  pub benefit_re_lottery_count: i32,
  pub is_allow_trial: bool,
}

impl CallCustom for DungeonAreaTopResponse {}

// See [Wonder_Api_DungeonAreaClearInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaClearInfo {
  pub clear_rank: i32,
  pub reward_items: Vec<DungeonAreaClearRewardInfo>,
}

// See [Wonder_Api_DungeonAreaClearRewardInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaClearRewardInfo {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

// See [Wonder_Api_DungeonStageStateResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStageState {
  pub stage_id: i32,
  pub is_challenge: bool,
  pub enemies: Vec<DungeonStageEnemyState>,
}

// See [Wonder_Api_DungeonStageEnemyStateResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStageEnemyState {
  pub enemy_id: i32,
  pub current_hp: i32,
}

// See [Wonder_Api_DungeonEnemyInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonEnemyInfo {
  pub enemy_id: i64,
  pub current_hp: i32,
}

// See [Wonder_Api_DungeonPartySetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonPartySet {
  pub stage_party_set: DungeonStagePartySet,
  pub team_members: Vec<PartyMember>,
  pub team_weapons: Vec<PartyWeapon>,
  pub team_accessories: Vec<PartyAccessory>,
}

// See [Wonder_Api_PartyinfoMembersResponseDto_Fields]
// Extended by [Wonder_Api_DungeonPartyMembersResponseDto_Fields]
// See [Wonder_Api_PartychangeMembersResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct PartyMember {
  pub id: i32,
  pub lv: i32,
  pub exp: i32,
  pub member_id: i64,
  pub ac_skill_lv_a: i32,
  pub ac_skill_val_a: i64,
  pub ac_skill_lv_b: i32,
  pub ac_skill_val_b: i64,
  pub ac_skill_lv_c: i32,
  pub ac_skill_val_c: i64,
  pub hp: i32,
  pub attack: i32,
  pub magicattack: i32,
  pub defense: i32,
  pub magicdefence: i32,
  pub agility: i32,
  pub dexterity: i32,
  pub luck: i32,
  pub limit_break: i32,
  pub character_id: i64,
  pub waiting_room: i32,
  pub ex_flg: i32,
  pub is_undead: i32,
}

// See [Wonder_Api_PartyinfoWeaponsResponseDto_Fields]
// Extended by [Wonder_Api_DungeonPartyWeaponsResponseDto_Fields]
// See [Wonder_Api_PartychangeWeaponsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyWeapon {
  pub id: i64,
  pub weapon_id: i64,
  pub trial: bool,
}

// See [Wonder_Api_PartyinfoAccessoriesResponseDto_Fields]
// See [Wonder_Api_DungeonPartyAccessoriesResponseDto_Fields]
// See [Wonder_Api_PartychangelistAccessoriesResponseDto_Fields]
// See [Wonder_Api_PartychangeAccessoriesResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PartyAccessory {
  pub id: i64,
  pub accessory_id: i64,
}

// See [Wonder_Api_DungeonStagePartySetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStagePartySet {
  pub party: Vec<DungeonStagePartyForm>,
  pub reserved_party: Vec<DungeonStagePartyForm>,
  pub assist: i64,
  pub sub_assists: Vec<i64>,
  pub assist_remain_count: i32,
  pub party_passive_skill: PartyPassiveSkillInfo,
}

// See [Wonder_Api_DungeonStagePartyFormResponseDto_Fields]
// extends [Wonder_Api_DungeonPartyFormResponseDto_Fields]
// extends [Wonder_Api_BasicPartyFormResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStagePartyForm {
  /* Wonder_Api_BasicPartyFormResponseDto_Fields */
  pub id: i32,
  pub form_no: i32,
  pub main: i32,
  pub sub1: i32,
  pub sub2: i32,
  pub weapon: i64,
  pub acc: i64,
  pub strength: i32,
  pub specialskill: SpecialSkillInfo,
  pub skill_pa_fame: i64,

  /* Wonder_Api_DungeonStagePartyFormResponseDto_Fields */
  pub current_hp: i32,
  pub max_hp: i32,
  pub current_sp: i32,
}

#[derive(Debug, Deserialize)]
pub struct DungeonAreaTopRequest {
  pub area_id: i32,
}

pub async fn dungeon_area_top(Params(params): Params<DungeonAreaTopRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_area_top");

  let benefits = get_master_manager().get_master("dungeon_benefit_level");

  Ok(Unsigned(DungeonAreaTopResponse {
    is_practice: false,
    stage_state: DungeonStageState {
      stage_id: 1101,
      is_challenge: false,
      enemies: vec![],
    },
    party_set: DungeonPartySet {
      stage_party_set: DungeonStagePartySet {
        party: vec![],
        reserved_party: vec![],
        assist: 0,
        sub_assists: vec![],
        assist_remain_count: 0,
        party_passive_skill: Default::default(),
      },
      team_members: vec![],
      team_weapons: vec![],
      team_accessories: vec![],
    },
    clear_info: DungeonAreaClearInfo {
      clear_rank: 3,
      reward_items: vec![DungeonAreaClearRewardInfo {
        item_type: RemoteDataItemType::RealMoney.into(),
        item_id: 1,
        item_num: 10000,
      }],
    },
    unchoosed_benefit_id_list: benefits
      .iter()
      .map(|benefit| benefit["id"].as_str().unwrap().parse::<i32>().unwrap())
      .collect(),
    benefit_re_lottery_count: 10,
    is_allow_trial: true,
  }))
}

pub async fn dungeon_area_retire() -> impl IntoHandlerResponse {
  warn!("encountered stub: dungeon_area_retire");

  // See [Wonder_Api_DungeonAreaRetireResponseDto_Fields]
  Ok(Unsigned(()))
}

#[derive(Debug, Deserialize)]
pub struct DungeonTopRequest {
  pub dungeon_id: i32,
}

// See [Wonder_Api_DungeonTopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonTopResponse {
  pub challenging_area_id: i32,
  pub area_info_list: Vec<DungeonAreaInfo>,
  pub challenge_count_info: DungeonChallengeCountInfo,
  pub has_new_dungeon: bool,
}

impl CallCustom for DungeonTopResponse {}

// See [Wonder_Api_DungeonAreaInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaInfo {
  pub area_id: i32,
  pub clear_rank: i32,
  pub is_challengeable: bool,
}

// See [Wonder_Api_DungeonChallengeCountInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonChallengeCountInfo {
  pub bought_count: i32,
  pub available_buy_count: i32,
}

pub async fn dungeon_top(Params(params): Params<DungeonTopRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_top");

  let areas = get_master_manager().get_master("dungeon_area");

  let mut response = CallResponse::new_success(Box::new(DungeonTopResponse {
    challenging_area_id: 0,
    area_info_list: areas
      .iter()
      .filter(|area| area["dungeon_id"].as_str().unwrap().parse::<i32>().unwrap() == params.dungeon_id)
      .map(|area| DungeonAreaInfo {
        area_id: area["id"].as_str().unwrap().parse::<i32>().unwrap(),
        clear_rank: 2,
        is_challengeable: true,
      })
      .collect(),
    challenge_count_info: DungeonChallengeCountInfo {
      bought_count: 4,
      available_buy_count: 4,
    },
    has_new_dungeon: true,
  }));
  // response.remote.push(AddItem::new(RemoteDataItemType::DungeonChallenge, 1, 1, 4).into_remote_data());

  Ok(Unsigned(response))
}

// See [Wonder_Api_DungeonListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonListResponse {
  pub dungeon: DungeonInfo,
  pub regular_dungeons: RegularDungeonsInfo,
}

impl CallCustom for DungeonListResponse {}

// See [Wonder_Api_DungeonInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonInfo {
  pub dungeon_id: i32,
  pub is_completed: bool,
  pub is_new: bool,
}

// See [Wonder_Api_RegularDungeonsInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RegularDungeonsInfo {
  pub dungeon_list: Vec<DungeonInfo>,
  pub challenge_count_info: DungeonChallengeCountInfo,
}

pub async fn dungeon_list() -> impl IntoHandlerResponse {
  let dungeons = get_master_manager().get_master("dungeon");

  Ok(Unsigned(DungeonListResponse {
    dungeon: DungeonInfo {
      dungeon_id: dungeons[0]["id"].as_str().unwrap().parse::<i32>().unwrap(),
      is_completed: false,
      is_new: true,
    },
    regular_dungeons: RegularDungeonsInfo {
      dungeon_list: dungeons
        .iter()
        .map(|dungeon| DungeonInfo {
          dungeon_id: dungeon["id"].as_str().unwrap().parse::<i32>().unwrap(),
          is_completed: false,
          is_new: true,
        })
        .collect(),
      challenge_count_info: DungeonChallengeCountInfo {
        bought_count: 2,
        available_buy_count: 2,
      },
    },
  }))
}

// See [Wonder_Api_DungeonTeamInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonTeamInfoResponse {
  pub team_set: DungeonTeamSet,
}

impl CallCustom for DungeonTeamInfoResponse {}

// See [Wonder_Api_DungeonTeamSetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonTeamSet {
  /// Must be of size [master<dungeon_area>.member_num]
  pub party: Vec<DungeonStagePartyForm>,
  pub members: Vec<PartyMember>,
  pub weapons: Vec<PartyWeapon>,
  pub accessories: Vec<PartyAccessory>,
  pub assist: i64,
  pub sub_assists: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DungeonTeamInfoRequest {
  pub area_id: i32,
}

async fn get_team_set(state: &AppState, session: &Session, member_num: usize) -> anyhow::Result<DungeonTeamSet> {
  let client = state.get_database_client().await?;

  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let mut members = fetch_members.run(session.user_id).await.unwrap();
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut members.iter_mut().collect::<Vec<_>>())
    .await?;

  Ok(DungeonTeamSet {
    party: (1..=member_num)
      .map(|i| DungeonStagePartyForm {
        id: 1,
        form_no: i as i32,
        main: members.get(i - 1).map_or(0, |m| m.id),
        sub1: 0,
        sub2: 0,
        weapon: 0,
        acc: 0,
        strength: 123,
        specialskill: SpecialSkillInfo {
          special_skill_id: 100001,
          trial: false,
        },
        skill_pa_fame: 0,
        current_hp: 100,
        max_hp: 100,
        current_sp: 0,
      })
      .collect(),
    // TODO: We must send only members that are used in the party, otherwise hardlock occurs?
    members: members
      .into_iter()
      .map(|member| member.to_party_member())
      // .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
      .take(member_num)
      .collect(),
    weapons: vec![],
    accessories: vec![],
    assist: 0,
    sub_assists: vec![],
  })
}

pub async fn dungeon_team_info(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonTeamInfoRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_team_info");

  let areas = get_master_manager().get_master("dungeon_area");
  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .context("invalid area_id")?;
  let member_num = area["member_num"].as_str().unwrap().parse::<usize>().unwrap();

  Ok(Unsigned(DungeonTeamInfoResponse {
    team_set: get_team_set(&state, &session, member_num).await?,
  }))
}

// See [Wonder_Api_DungeonAreaChallengeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaChallengeResponse {
  pub stage_state: DungeonStageState,
  pub party_set: DungeonPartySet,
}

impl CallCustom for DungeonAreaChallengeResponse {}

#[derive(Debug, Deserialize)]
pub struct DungeonAreaChallengeRequest {
  pub area_id: i32,
  #[serde(with = "crate::bool_as_int")]
  pub is_practice: bool,
}

pub async fn dungeon_area_challenge(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonAreaChallengeRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_area_challenge");

  let areas = get_master_manager().get_master("dungeon_area");
  let stages = get_master_manager().get_master("dungeon_stage");
  let enemies = get_master_manager().get_master("battle_enemy");

  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .context("invalid area_id")?;
  let member_num = area["member_num"].as_str().unwrap().parse::<usize>().unwrap();

  let mut stages = stages
    .iter()
    .filter(|stage| stage["area_id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id);
  let stage = stages.next().context("no stages found for area")?;

  let client = state.get_database_client().await?;

  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let mut members = fetch_members.run(session.user_id).await.unwrap();
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut members.iter_mut().collect::<Vec<_>>())
    .await?;

  Ok(Unsigned(DungeonAreaChallengeResponse {
    stage_state: DungeonStageState {
      stage_id: stage["id"].as_str().unwrap().parse::<i32>().unwrap(),
      is_challenge: false,
      enemies: enemies
        .iter()
        .take(3)
        .map(|enemy| DungeonStageEnemyState {
          enemy_id: enemy["enemy_id"].as_str().unwrap().parse::<i32>().unwrap(),
          current_hp: 100,
        })
        .collect(),
    },
    party_set: DungeonPartySet {
      stage_party_set: DungeonStagePartySet {
        party: (1..=member_num)
          .map(|i| DungeonStagePartyForm {
            id: 1,
            form_no: i as i32,
            main: members.get(i - 1).map_or(0, |m| m.id),
            sub1: 0,
            sub2: 0,
            weapon: 0,
            acc: 0,
            strength: 123,
            specialskill: SpecialSkillInfo {
              special_skill_id: 100001,
              trial: false,
            },
            skill_pa_fame: 0,
            current_hp: 100,
            max_hp: 100,
            current_sp: 0,
          })
          .collect(),
        reserved_party: vec![],
        assist: 0,
        sub_assists: vec![],
        assist_remain_count: 0,
        party_passive_skill: Default::default(),
      },
      // TODO: We must send only members that are used in the party, otherwise hardlock occurs?
      team_members: members
        .into_iter()
        .map(|member| member.to_party_member())
        // .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
        .take(member_num)
        .collect(),
      team_weapons: vec![],
      team_accessories: vec![],
    },
  }))
}

// See [Wonder_Api_DungeonStagePartyInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStagePartyInfoResponse {
  pub party_set: DungeonPartySet,
  pub is_allow_trial: bool,
}

impl CallCustom for DungeonStagePartyInfoResponse {}

#[derive(Debug, Deserialize)]
pub struct DungeonStagePartyInfoRequest {
  pub area_id: i32,
}

pub async fn dungeon_stage_party_info(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonStagePartyInfoRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_stage_party_info");

  let areas = get_master_manager().get_master("dungeon_area");
  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .context("invalid area_id")?;
  let member_num = area["member_num"].as_str().unwrap().parse::<usize>().unwrap();

  let client = state.get_database_client().await?;

  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let mut members = fetch_members.run(session.user_id).await.unwrap();
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut members.iter_mut().collect::<Vec<_>>())
    .await?;

  Ok(Unsigned(DungeonStagePartyInfoResponse {
    party_set: DungeonPartySet {
      stage_party_set: DungeonStagePartySet {
        party: (1..=member_num)
          .map(|i| DungeonStagePartyForm {
            id: 1,
            form_no: i as i32,
            main: members.get(i - 1).map_or(0, |m| m.id),
            sub1: 0,
            sub2: 0,
            weapon: 0,
            acc: 0,
            strength: 123,
            specialskill: SpecialSkillInfo {
              special_skill_id: 100001,
              trial: false,
            },
            skill_pa_fame: 0,
            current_hp: 100,
            max_hp: 100,
            current_sp: 0,
          })
          .collect(),
        reserved_party: vec![],
        assist: 0,
        sub_assists: vec![],
        assist_remain_count: 0,
        party_passive_skill: Default::default(),
      },
      // TODO: We must send only members that are used in the party, otherwise hardlock occurs?
      team_members: members
        .into_iter()
        .map(|member| member.to_party_member())
        // .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
        .take(member_num)
        .collect(),
      team_weapons: vec![],
      team_accessories: vec![],
    },
    is_allow_trial: true,
  }))
}

// See [Wonder_Api_DungeonBattleStartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonBattleStartResponse {
  pub chest: String,
  pub party: BattleParty,
  pub members: Vec<DungeonBattleMember>,
  pub resume_info: String,
  pub livemembers: Vec<DungeonBattleLiveMember>,
  pub benefit_id_list: Vec<i32>,
  pub enemy_info: Vec<DungeonEnemyInfo>,
  pub hp_increase: i32,
}

impl CallCustom for DungeonBattleStartResponse {}

// See [Wonder_Api_DungeonBattleStartMembersResponseDto_Fields]
// extends [Wonder_Api_BattlestartMembersResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct DungeonBattleMember {
  /* Wonder_Api_BattlestartMembersResponseDto_Fields */
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

  /* Wonder_Api_DungeonBattleStartMembersResponseDto_Fields */
  pub character_piece_board_stage_id_list: Vec<i32>,
}

// See [Wonder_Api_BattleresumeLivemembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonBattleLiveMember {
  pub id: i32,
  pub hp: i32,
  pub form_no: i32,
}

#[derive(Debug, Deserialize)]
pub struct DungeonBattleStartRequest {
  pub stage_id: i32,
}

pub async fn dungeon_battle_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonBattleStartRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_battle_start");

  let areas = get_master_manager().get_master("dungeon_area");
  let stages = get_master_manager().get_master("dungeon_stage");
  let enemies = get_master_manager().get_master("battle_enemy");

  let stage = stages
    .iter()
    .find(|stage| stage["id"].as_str().unwrap().parse::<i32>().unwrap() == params.stage_id)
    .context("invalid stage_id")?;
  let area_id = stage["area_id"].as_str().unwrap().parse::<i32>().unwrap();
  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == area_id)
    .context("invalid area_id")?;
  let member_num = 5; // area["member_num"].as_str().unwrap().parse::<usize>().unwrap();
  let enemy_list: Vec<_> = enemies
    .iter()
    .take(3)
    .map(|enemy| DungeonEnemyInfo {
      enemy_id: enemy["enemy_id"].as_str().unwrap().parse::<i32>().unwrap() as i64,
      current_hp: 100,
    })
    .collect();

  let client = state.get_database_client().await?;

  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let mut members = fetch_members.run(session.user_id).await.unwrap();
  FetchUserMemberSkillsIn::new(&client)
    .await?
    .run(session.user_id, &mut members.iter_mut().collect::<Vec<_>>())
    .await?;

  Ok(Unsigned(DungeonBattleStartResponse {
    chest: "10101111,10101120,10101131".to_string(),
    party: BattleParty {
      party_forms: (1..=5)
        .map(|i| BasicBattlePartyForm {
          party_no: 1,
          id: 1,
          form_no: i as i32,
          main: members.get(i - 1).map_or(0, |m| m.id),
          sub1: 0,
          sub2: 0,
          weapon: 0,
          acc: 0,
          skill_pa_fame: 0,
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
      assist: 0,
      sub_assists: vec![],
      party_passive_skill: Default::default(),
    },
    // TODO: We must send only members that are used in the party, otherwise hardlock occurs?
    members: members
      .iter()
      .map(|member| member.to_dungeon_battle_member())
      // .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
      .take(member_num)
      .collect(),
    resume_info: "".to_string(),
    livemembers: members
      .iter()
      .take(member_num)
      .map(|member| DungeonBattleLiveMember {
        id: member.id,
        hp: 100,
        form_no: 1,
      })
      .collect(),
    benefit_id_list: vec![],
    enemy_info: enemy_list,
    hp_increase: 0,
  }))
}

#[derive(Debug, Deserialize)]
pub struct DungeonBattleDefeatRequest {
  #[serde(with = "crate::bool_as_int")]
  pub is_retire: bool,
  pub assist_remain_count: i32,
  pub enemy_info: Vec<DungeonEnemyInfoRequest>,
}

// See [Wonder_Api_DungeonEnemyInfoRequestDto_Fields]
#[derive(Debug, Deserialize)]
pub struct DungeonEnemyInfoRequest {
  pub enemy_id: i64,
  pub hp: i32,
}

pub async fn dungeon_battle_defeat(
  session: Arc<Session>,
  Params(params): Params<DungeonBattleDefeatRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_battle_defeat");

  // See [Wonder_Api_DungeonBattleDefeatResponseDto_Fields]
  Ok(Unsigned(()))
}

// See [Wonder_Api_DungeonTeamOfferResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonTeamOfferResponse {
  pub team_set: DungeonTeamSet,
}

impl CallCustom for DungeonTeamOfferResponse {}

// body={"area_id": "11", "assist": "1", "sub": "1", "accessory_priority_resistances": "[\"none\",\"none\"]", "elemental": "[\"none\",\"none\"]", "priority_status": "strength", "equip": "1", "main": "1", "skill_pa_fame": "1", "weapon_priority_status": "attack"}
#[derive(Debug, Deserialize)]
pub struct DungeonTeamOfferRequest {
  pub area_id: i32,
  pub assist: i64,
  pub sub: i64,
  pub accessory_priority_resistances: Vec<String>,
  pub elemental: Vec<String>,
  pub priority_status: String,
  pub equip: i64,
  pub main: i64,
  pub skill_pa_fame: i64,
  pub weapon_priority_status: String,
}

pub async fn dungeon_team_offer(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonTeamOfferRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_team_offer");

  let areas = get_master_manager().get_master("dungeon_area");
  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .context("invalid area_id")?;
  let member_num = area["member_num"].as_str().unwrap().parse::<usize>().unwrap();

  Ok(Unsigned(DungeonTeamOfferResponse {
    team_set: get_team_set(&state, &session, member_num).await?,
  }))
}

// See [Wonder_Api_DungeonTeamResetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonTeamResetResponse {
  pub team_set: DungeonTeamSet,
}

impl CallCustom for DungeonTeamResetResponse {}

#[derive(Debug, Deserialize)]
pub struct DungeonTeamResetRequest {
  pub area_id: i32,
}

pub async fn dungeon_team_reset(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<DungeonTeamResetRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_team_reset");

  let areas = get_master_manager().get_master("dungeon_area");
  let area = areas
    .iter()
    .find(|area| area["id"].as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
    .context("invalid area_id")?;
  let member_num = area["member_num"].as_str().unwrap().parse::<usize>().unwrap();

  Ok(Unsigned(DungeonTeamResetResponse {
    team_set: get_team_set(&state, &session, member_num).await?,
  }))
}

// See [Wonder_Api_DungeonAreaSkipResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaSkipResponse {
  pub reward: Vec<BattleSkipReward>,
}

impl CallCustom for DungeonAreaSkipResponse {}

// See [Wonder_Api_BattleskipRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleSkipReward {
  /// Stage number
  pub dropnum: i32,
  pub exp: i32,
  pub money: i32,
  pub itemtype: i32,
  pub itemid: i32,
  /// Negative values are not displayed :(
  pub itemnum: i32,
}

#[derive(Debug, Deserialize)]
pub struct DungeonAreaSkipRequest {
  pub area_id: i32,
  pub skip_count: i32,
}

pub async fn dungeon_area_skip(Params(params): Params<DungeonAreaSkipRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: dungeon_area_skip");

  Ok(Unsigned(DungeonAreaSkipResponse {
    reward: vec![BattleSkipReward {
      dropnum: 1,
      exp: 500,
      money: 1000,
      itemtype: RemoteDataItemType::RealMoney.into(),
      itemid: 1,
      itemnum: 5000,
    }],
  }))
}
