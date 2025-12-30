//! Reference: https://youtu.be/R80TMWhTdik
//! Stamps reference: https://youtu.be/sDF9jb8TIvY

use crate::AppState;
use crate::api::battle::{BattleMember, BattleParty};
use crate::api::dungeon::{DungeonStagePartyForm, DungeonTeamSet};
use crate::api::home::{MultiBattleInvitation, MultiBattleRoom};
use crate::api::party_info::{Party, PartyForm, SpecialSkillInfo};
use crate::api::{MemberFameStats, RemoteDataItemType};
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::member::{Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

// See [Wonder_Api_MultiBattleInvitationListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitationListResponse {
  pub multi_battle_invitation: MultiBattleInvitation,
}

impl CallCustom for MultiBattleInvitationListResponse {}

// TODO: BROKEN - Client starts lagging, displays stub (?) text, and does not respond to selection
pub async fn multi_battle_invitation_list() -> impl IntoHandlerResponse {
  warn!("encountered stub: multi_battle_invitation_list");

  Ok(Unsigned(MultiBattleInvitationListResponse {
    multi_battle_invitation: MultiBattleInvitation {
      event_id: 24011,
      rooms: vec![MultiBattleRoom {
        room_no: 1,
        quest_id: 500101,
        user_icon: 1083110,
        user_name: "Megumin".to_string(),
      }],
    },
  }))
}

// See [Wonder_Api_MultiBattleRoomInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoomInfoResponse {
  pub quest_id: i32,
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  pub icon: i64,
  #[serde(with = "crate::bool_as_int")]
  pub is_lock: bool,
}

impl CallCustom for MultiBattleRoomInfoResponse {}

#[derive(Debug, Deserialize)]
pub struct MultiBattleRoomInfoRequest {
  pub room_no: i32,
}

// TODO: BROKEN - Client does not show anything
pub async fn multi_battle_room_info(Params(params): Params<MultiBattleRoomInfoRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_room_info");

  Ok(Unsigned(MultiBattleRoomInfoResponse {
    quest_id: 0,
    name: "Megumin".to_string(),
    icon: 1083110,
    is_lock: false,
  }))
}

// See [Wonder_Api_MultiBattleCreateRoomResponseDto_Fields]
// See [Wonder_Api_MultiBattleSearchAndJoinRoomResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoomResponse {
  pub room_no: i32,
  pub members: Vec<MultiBattleRoomMember>,
  pub room_status: i32,
  pub open_flag: i32,
  pub invite_flag: i32,
  pub read_only_token: String,
}

impl CallCustom for MultiBattleRoomResponse {}

// See [Wonder_Api_MultiBattleCreateRoomMembersResponseDto_Fields]
// See [Wonder_Api_MultiBattleSearchAndJoinRoomMembersResponseDto_Fields]
// See [Wonder_Api_MultiBattleRoomStatusMembersResponseDto_Fields]
// See [Wonder_Api_MultiBattleJoinRoomMembersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoomMember {
  pub user_no: String,
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  pub icon: i64,
  pub user_rank: i32,
  pub strength: i32,
  pub is_host: i32,
  pub honor_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct MultiBattleCreateRoomRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub quest_id: i32,
}

pub async fn multi_battle_create_room(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MultiBattleCreateRoomRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_create_room");

  Ok(Unsigned(MultiBattleRoomResponse {
    room_no: 1,
    members: vec![MultiBattleRoomMember {
      user_no: "-1".to_owned(),
      name: "Megumin".to_owned(),
      icon: 1083110,
      user_rank: 100,
      strength: 50000,
      is_host: 1,
      honor_id: 62010250,
    }],
    room_status: 0,
    open_flag: 1,
    invite_flag: 1,
    read_only_token: "stub_token".to_string(),
  }))
}

// body={"party_no": "1", "quest_id": "513894"}
#[derive(Debug, Deserialize)]
pub struct MultiBattleSearchAndJoinRoomRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub quest_id: i32,
}

pub async fn multi_battle_search_and_join_room(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MultiBattleSearchAndJoinRoomRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_search_and_join_room");

  Ok(Unsigned(MultiBattleRoomResponse {
    room_no: 1,
    members: vec![MultiBattleRoomMember {
      user_no: "-1".to_owned(),
      name: "Megumin".to_owned(),
      icon: 1083110,
      user_rank: 100,
      strength: 50000,
      is_host: 1,
      honor_id: 62010250,
    }],
    room_status: 0,
    open_flag: 1,
    invite_flag: 1,
    read_only_token: "stub_token".to_string(),
  }))
}

// See [Wonder_Api_MultiBattleRoomStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoomStatusResponse {
  pub members: Vec<MultiBattleRoomMember>,
  /// 0 - waiting (invalid?), 1 - waiting, 2 - start battle
  pub room_status: i32,
  pub open_flag: i32,
  pub invite_flag: i32,
}

impl CallCustom for MultiBattleRoomStatusResponse {}

// body={"read_only_token": "stub_token", "room_no": "1"}
#[derive(Debug, Deserialize)]
pub struct MultiBattleRoomStatusRequest {
  pub read_only_token: String,
  pub room_no: i32,
}

/// Sent every 4 seconds while in a multi battle room.
pub async fn multi_battle_room_status(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MultiBattleRoomStatusRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_room_status");

  Ok(Unsigned(MultiBattleRoomStatusResponse {
    members: vec![
      MultiBattleRoomMember {
        user_no: "-1".to_owned(),
        name: "Megumin".to_owned(),
        icon: 1083110,
        user_rank: 100,
        strength: 50000,
        is_host: 1,
        honor_id: 62010250,
      },
      MultiBattleRoomMember {
        user_no: "-2".to_owned(),
        name: "Aqua".to_owned(),
        icon: 1083110,
        user_rank: 1000000032,
        strength: 34000,
        is_host: 0,
        honor_id: 62010250,
      },
      MultiBattleRoomMember {
        user_no: "-3".to_owned(),
        name: "Kazuma".to_owned(),
        icon: 1083110,
        user_rank: 16,
        strength: 4,
        is_host: 0,
        honor_id: 62010250,
      },
      MultiBattleRoomMember {
        user_no: "-4".to_owned(),
        name: "Darkness".to_owned(),
        icon: 1083110,
        user_rank: 8900,
        strength: 90000,
        is_host: 0,
        honor_id: 62010250,
      },
    ],
    room_status: 2,
    open_flag: 1,
    invite_flag: 1,
  }))
}

// See [Wonder_Api_MarathonMultiStartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiStartResponse {
  pub user_host: Vec<MarathonMultiStartUser>,
  pub user_guest1: Vec<MarathonMultiStartUser>,
  pub user_guest2: Vec<MarathonMultiStartUser>,
  pub user_guest3: Vec<MarathonMultiStartUser>,
  pub chest: String,
  pub party: BattleParty,
  pub members: Vec<BattleMember>,
  pub battle_id: i64,
  pub will_use_ticket: i32,
  pub read_only_token: String,
  pub get_log: bool,
}

// See [Wonder_Api_MarathonMultiStartUserHostResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStartUserGuest1ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStartUserGuest2ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStartUserGuest3ResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiStartUser {
  pub user_no: String,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  pub hp: i32,
  pub icon: i32,
  pub strength: i32,
  pub status: i32,
}

impl CallCustom for MarathonMultiStartResponse {}

// body={"party_no": "1", "event_id": "24011", "quest_id": "513894", "ticket_ratio": "1", "room_no": "1"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiStartRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub event_id: i32,
  pub quest_id: i32,
  pub ticket_ratio: i32,
  pub room_no: i32,
}

pub async fn marathon_multi_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiStartRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_start");

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

  Ok(Unsigned(MarathonMultiStartResponse {
    user_host: vec![
      MarathonMultiStartUser {
        user_no: "-1".to_owned(),
        user_name: "Megumin".to_owned(),
        hp: 100,
        icon: 1083110,
        strength: 50000,
        status: 0,
      },
      MarathonMultiStartUser {
        user_no: "-2".to_owned(),
        user_name: "Aqua".to_owned(),
        hp: 100,
        icon: 1083110,
        strength: 34000,
        status: 0,
      },
      MarathonMultiStartUser {
        user_no: "-3".to_owned(),
        user_name: "Kazuma".to_owned(),
        hp: 100,
        icon: 1083110,
        strength: 4,
        status: 0,
      },
      MarathonMultiStartUser {
        user_no: "-4".to_owned(),
        user_name: "Darkness".to_owned(),
        hp: 100,
        icon: 1083110,
        strength: 90000,
        status: 0,
      },
    ],
    user_guest1: vec![],
    user_guest2: vec![],
    user_guest3: vec![],
    chest: "10101111,10101120,10101131".to_string(),
    party: party.to_battle_party(),
    // We must send only members that are used in the party, otherwise hardlock occurs
    members: members
      .into_iter()
      .filter(|member| party.party_forms.iter().any(|form| form.main == member.id))
      .collect(),
    battle_id: 1,
    will_use_ticket: 1,
    read_only_token: "stub_token".to_string(),
    get_log: true,
  }))
}

// See [Wonder_Api_MarathonMultiBattlingResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiBattlingResponse {
  pub user_host: Vec<MarathonMultiBattlingUser>,
  pub user_guest1: Vec<MarathonMultiBattlingUser>,
  pub user_guest2: Vec<MarathonMultiBattlingUser>,
  pub user_guest3: Vec<MarathonMultiBattlingUser>,
  pub enemy: Vec<i32>,
  pub battletime: i32,
  pub battle_id: i64,
  pub is_timeup: i32,
}

impl CallCustom for MarathonMultiBattlingResponse {}

// See [Wonder_Api_MarathonMultiBattlingUserHostResponseDto_Fields]
// See [Wonder_Api_MarathonMultiBattlingUserGuest1ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiBattlingUserGuest2ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiBattlingUserGuest3ResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiBattlingUser {
  pub user_no: String,
  pub hp: i32,
  pub stamp: i32,
  pub damage: i32,
  pub attack_type: i32,
  pub status: i32,
}

// body={"damage": "254", "battle_id": "1", "battletime": "9", "read_only_token": "stub_token", "stamp": "0", "event_id": "24011", "party_hp": "3525", "attack_type": "2", "room_no": "1"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiBattlingRequest {
  pub damage: i32,
  pub battle_id: i64,
  pub battletime: i32,
  pub read_only_token: String,
  pub stamp: i64,
  pub event_id: i32,
  pub party_hp: i32,
  pub attack_type: i32,
  #[serde(rename = "room_no")]
  pub room_id: i32,
}

pub async fn marathon_multi_battling(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiBattlingRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_battling");

  Ok(Unsigned(MarathonMultiBattlingResponse {
    user_host: vec![
      MarathonMultiBattlingUser {
        user_no: "-1".to_owned(),
        hp: 3525,
        stamp: 0,
        damage: params.damage,
        attack_type: params.attack_type,
        status: 0,
      },
      MarathonMultiBattlingUser {
        user_no: "-2".to_owned(),
        hp: 4000,
        stamp: 0,
        damage: 1234,
        attack_type: 1,
        status: 0,
      },
      MarathonMultiBattlingUser {
        user_no: "-3".to_owned(),
        hp: 5000,
        stamp: 0,
        damage: 5678,
        attack_type: 2,
        status: 0,
      },
      MarathonMultiBattlingUser {
        user_no: "-4".to_owned(),
        hp: 6000,
        stamp: 0,
        damage: 91011,
        attack_type: 1,
        status: 0,
      },
    ],
    user_guest1: vec![],
    user_guest2: vec![],
    user_guest3: vec![],
    enemy: vec![17800029, 17800029],
    battletime: 0,
    battle_id: params.battle_id,
    is_timeup: 0,
  }))
}

// See [Wonder_Api_MarathonMultiResultConfirmResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiResultConfirmResponse {
  pub battle_status: i32,
  pub confirmed_user: i32,
}

impl CallCustom for MarathonMultiResultConfirmResponse {}

// body={"battle_id": "1", "win": "2", "room_no": "1", "event_id": "24011"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiResultConfirmRequest {
  pub battle_id: i64,
  pub win: i32,
  #[serde(rename = "room_no")]
  pub room_id: i32,
  pub event_id: i32,
}

pub async fn marathon_multi_result_confirm(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiResultConfirmRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_result_confirm");

  Ok(Unsigned(MarathonMultiResultConfirmResponse {
    // 0 - invalid, 1 - boss defeated, 2 - complete, 3 - invalid
    battle_status: 1,
    confirmed_user: 1,
  }))
}

// body={"event_id": "24011", "quest_id": "513894", "room_no": "1", "log": "party front,1014117,754,111,119,76,81,65,78,10,party front,1024186,764,125,150,82,99,68,71,72,party front,1044110,779,112,112,80,83,76,77,92,party back,1004205,627,85,84,62,58,73,80,86,party back,1004200,601,82,79,57,55,74,78,90,w1,attack,1044110,0,72000172,0,0,142,0,0,0,0,0,w1,attack,1024186,0,211042100000310134,0,0,172,0,0,0,0,0,w1,attack,1014117,0,152540020001330154,0,0,113,0,0,0,0,0"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiLogRequest {
  pub event_id: i32,
  pub quest_id: i32,
  #[serde(rename = "room_no")]
  pub room_id: i32,
  #[serde(deserialize_with = "crate::serde_compat::comma_separated_string")]
  pub log: Vec<String>,
}

pub async fn marathon_multi_log(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiLogRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_log");

  // See [Wonder_Api_MarathonMultiLogResponseDto_Fields]
  Ok(Unsigned(()))
}

// See [Wonder_Api_MarathonMultiResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiResultResponse {
  pub exp: i32,
  pub lvup: i32,
  pub money: i32,
  pub friend_count: i32,
  pub request_count: i32,
  pub love: Vec<BattleCharacterLove>,
  pub member_exp: Vec<BattleMemberExp>,
  pub reward: Vec<MultiBattleReward>,
  pub clearreward: Vec<BattleClearReward>,
  pub user_host: Vec<MarathonMultiResultUser>,
  pub user_guest1: Vec<MarathonMultiResultUser>,
  pub user_guest2: Vec<MarathonMultiResultUser>,
  pub user_guest3: Vec<MarathonMultiResultUser>,
  pub battle_id: i64,
}

impl CallCustom for MarathonMultiResultResponse {}

// See [Wonder_Api_MarathonMultiResultLoveResponseDto_Fields]
// See [Wonder_Api_BattlehuntingresultLoveResponseDto_Fields]
// See [Wonder_Api_ResultLoveResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleCharacterLove {
  pub character_id: i64,
  pub love: i32,
}

// See [Wonder_Api_MarathonMultiResultMemberExpResponseDto_Fields]
// See [Wonder_Api_BattlehuntingresultMemberExpResponseDto_Fields]
// See [Wonder_Api_ResultMemberExpResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleMemberExp {
  pub member_id: i64,
  pub exp: i32,
}

// See [Wonder_Api_MarathonMultiResultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  pub is_rare: i32,
  pub is_mvp: i32,
}

// See [Wonder_Api_MarathonMultiResultClearrewardResponseDto_Fields]
// See [Wonder_Api_BattlehuntingresultClearrewardResponseDto_Fields]
// See [Wonder_Api_ResultClearrewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleClearReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  pub mission: i32,
}

// See [Wonder_Api_MarathonMultiResultUserHostResponseDto_Fields]
// See [Wonder_Api_MarathonMultiResultUserGuest1ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiResultUserGuest2ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiResultUserGuest3ResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiResultUser {
  pub user_no: String,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
  pub damage: i32,
  pub friend_state: i32,
  pub friend_count: i32,
  pub request_received_count: i32,
  pub icon: i64,
  pub is_mvp: i32,
  pub status: i32,
}

// body={"win": "0", "event_id": "24011", "battle_time": "-2147483647", "room_no": "1", "battle_id": "1"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiResultRequest {
  pub win: i32,
  pub event_id: i32,
  pub battle_time: i32,
  #[serde(rename = "room_no")]
  pub room_id: i32,
  pub battle_id: i64,
}

pub async fn marathon_multi_result(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiResultRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_result");

  // See [Wonder_Api_MarathonMultiLogResponseDto_Fields]
  Ok(Unsigned(MarathonMultiResultResponse {
    exp: 500,
    lvup: 1,
    money: 1000,
    friend_count: 0,
    request_count: 0,
    love: vec![],
    member_exp: vec![],
    reward: vec![MultiBattleReward {
      itemtype: RemoteDataItemType::RealMoney.into(),
      itemid: 1,
      itemnum: 100,
      is_rare: 1,
      is_mvp: 1,
    }],
    clearreward: vec![BattleClearReward {
      itemtype: RemoteDataItemType::RealMoneyFree.into(),
      itemid: 1,
      itemnum: 5000,
      mission: 1,
    }],
    user_host: vec![
      MarathonMultiResultUser {
        user_no: "-1".to_owned(),
        user_name: "Megumin".to_owned(),
        damage: 12345,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 1,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-2".to_owned(),
        user_name: "Aqua".to_owned(),
        damage: 23456,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-3".to_owned(),
        user_name: "Kazuma".to_owned(),
        damage: 34567,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-4".to_owned(),
        user_name: "Darkness".to_owned(),
        damage: 45678,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
    ],
    user_guest1: vec![
      MarathonMultiResultUser {
        user_no: "-1".to_owned(),
        user_name: "Megumin".to_owned(),
        damage: 12345,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 1,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-2".to_owned(),
        user_name: "Aqua".to_owned(),
        damage: 23456,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-3".to_owned(),
        user_name: "Kazuma".to_owned(),
        damage: 34567,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-4".to_owned(),
        user_name: "Darkness".to_owned(),
        damage: 45678,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
    ],
    user_guest2: vec![
      MarathonMultiResultUser {
        user_no: "-1".to_owned(),
        user_name: "Megumin".to_owned(),
        damage: 12345,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 1,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-2".to_owned(),
        user_name: "Aqua".to_owned(),
        damage: 23456,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-3".to_owned(),
        user_name: "Kazuma".to_owned(),
        damage: 34567,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-4".to_owned(),
        user_name: "Darkness".to_owned(),
        damage: 45678,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
    ],
    user_guest3: vec![
      MarathonMultiResultUser {
        user_no: "-1".to_owned(),
        user_name: "Megumin".to_owned(),
        damage: 12345,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 1,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-2".to_owned(),
        user_name: "Aqua".to_owned(),
        damage: 23456,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-3".to_owned(),
        user_name: "Kazuma".to_owned(),
        damage: 34567,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
      MarathonMultiResultUser {
        user_no: "-4".to_owned(),
        user_name: "Darkness".to_owned(),
        damage: 45678,
        friend_state: 0,
        friend_count: 0,
        request_received_count: 0,
        icon: 1083110,
        is_mvp: 0,
        status: 0,
      },
    ],
    battle_id: params.battle_id,
  }))
}

// See [Wonder_Api_MarathonMultiStampResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiStampResponse {
  pub user_host: Vec<MarathonMultiStampUser>,
  pub user_guest1: Vec<MarathonMultiStampUser>,
  pub user_guest2: Vec<MarathonMultiStampUser>,
  pub user_guest3: Vec<MarathonMultiStampUser>,
}

impl CallCustom for MarathonMultiStampResponse {}

// See [Wonder_Api_MarathonMultiStampUserHostResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStampUserGuest1ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStampUserGuest2ResponseDto_Fields]
// See [Wonder_Api_MarathonMultiStampUserGuest3ResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonMultiStampUser {
  pub user_no: String,
  pub stamp: i32,
}

// body={"read_only_token": "stub_token", "battle_id": "1", "stamp": "0"}
#[derive(Debug, Deserialize)]
pub struct MarathonMultiStampRequest {
  pub read_only_token: String,
  pub battle_id: i64,
  pub stamp: i32,
}

pub async fn marathon_multi_stamp(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MarathonMultiStampRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: marathon_multi_stamp");

  Ok(Unsigned(MarathonMultiStampResponse {
    user_host: vec![],
    user_guest1: vec![],
    user_guest2: vec![],
    user_guest3: vec![],
  }))
}

// See [Wonder_Api_MultiBattleJoinRoomResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleJoinRoomResponse {
  pub members: Vec<MultiBattleRoomMember>,
  /// 0 - waiting (invalid?), 1 - waiting, 2 - start battle
  pub room_status: i32,
  pub open_flag: i32,
  pub invite_flag: i32,
  pub read_only_token: String,
}

impl CallCustom for MultiBattleJoinRoomResponse {}

// body={"party_no": "1", "room_no": "1"}
#[derive(Debug, Deserialize)]
pub struct MultiBattleJoinRoomRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  #[serde(rename = "room_no")]
  pub room_id: i32,
}

pub async fn multi_battle_join_room(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MultiBattleJoinRoomRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_join_room");

  Ok(Unsigned(MultiBattleJoinRoomResponse {
    members: vec![MultiBattleRoomMember {
      user_no: "-1".to_owned(),
      name: "Megumin".to_owned(),
      icon: 1083110,
      user_rank: 100,
      strength: 50000,
      is_host: 1,
      honor_id: 62010250,
    }],
    room_status: 0,
    open_flag: 1,
    invite_flag: 1,
    read_only_token: "stub_token".to_string(),
  }))
}

// body={"room_no": "1"}
#[derive(Debug, Deserialize)]
pub struct MultiBattleRoomLeaveRequest {
  #[serde(rename = "room_no")]
  pub room_id: i32,
}

pub async fn multi_battle_room_leave(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<MultiBattleRoomLeaveRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_room_leave");

  // See [Wonder_Api_MultiBattleRoomLeaveResponseDto_Fields]
  Ok(Unsigned(()))
}
