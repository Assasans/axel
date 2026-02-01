use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, warn};

// See [Wonder_Api_HomeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Home {
  pub multi_battle_invitation: Option<MultiBattleInvitation>,
  pub member_info: MemberInfo,
  pub advertisement_data: AdvertisementData,
  pub display_plan_map: bool,
}

impl CallCustom for Home {}

// See [Wonder_Api_MultiBattleInvitationResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitation {
  pub event_id: i32,
  pub rooms: Vec<MultiBattleRoom>,
}

// See [Wonder_Api_MultiBattleInvitationRoomResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoom {
  pub room_no: i32,
  pub quest_id: i32,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
}

// See [Wonder_Api_HomeMemberInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MemberInfo {
  /// Should be present in [member_ids], otherwise first element is used.
  pub current_member_id: i64,
  // Must have 5 elements, pad with 0s if less.
  // Otherwise, client incorrectly displays empty slots.
  pub member_ids: [i64; 5],
}

// See [Wonder_Api_AdvertisementDataResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AdvertisementData {
  pub id: i32,
  pub reward_type: i32,
  pub status: i32,
}

pub async fn home(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;

  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select home_current_illustration_id
      from users
      where id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let row = client
    .query_one(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;
  // The column is nullable in the database, but it should be non-null after migrations have run.
  let home_current_illustration_id: i64 = row.get("home_current_illustration_id");

  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select member_id
      from user_home_illustrations
      where user_id = $1
      order by slot
    "#)
    .await
    .context("failed to prepare statement")?;
  let members = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;
  let members = members
    .into_iter()
    .map(|row| row.get::<_, Option<i64>>("member_id"))
    .collect::<Vec<_>>();
  let members = {
    let mut array = [0i64; 5];
    for (index, &member_id) in members.iter().enumerate() {
      array[index] = member_id.unwrap_or(0);
    }
    array
  };
  if !members.contains(&home_current_illustration_id) {
    warn!(
      "home_current_illustration_id {} not in member_ids {:?}",
      home_current_illustration_id, members
    );
  }

  Ok(Signed(
    Home {
      multi_battle_invitation: None,
      member_info: MemberInfo {
        current_member_id: home_current_illustration_id,
        member_ids: members,
      },
      advertisement_data: AdvertisementData {
        id: 10006,
        reward_type: 1,
        status: 0,
      },
      display_plan_map: false,
    },
    session,
  ))
}

// body={"illustration_ids": "[1011100,10242131,1064200,0,0]"}
// See [Wonder_Api_HomeMembersSetRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct HomeMembersSetRequest {
  pub illustration_ids: [i64; 5],
}

/// These are not member IDs as the method name suggests,
/// but illustration IDs from 'member_illustration' master.
pub async fn home_members_set(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<HomeMembersSetRequest>,
) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
  // Update user_home_illustrations: replace slot 1-5 with the provided illustration_ids.
  // Probably could be done more elegantly...
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_home_illustrations (user_id, slot, member_id)
      values ($1, 1, $2),
             ($1, 2, $3),
             ($1, 3, $4),
             ($1, 4, $5),
             ($1, 5, $6)
      on conflict (user_id, slot) do update set member_id = excluded.member_id
    "#)
    .await
    .context("failed to prepare statement")?;

  let illustration_ids = params.illustration_ids.map(|id| if id == 0 { None } else { Some(id) });
  client
    .execute(
      &statement,
      &[
        &session.user_id,
        &illustration_ids[0],
        &illustration_ids[1],
        &illustration_ids[2],
        &illustration_ids[3],
        &illustration_ids[4],
      ],
    )
    .await
    .context("failed to execute query")?;
  info!(?illustration_ids, "updated home members");

  // See [Wonder_Api_HomeMembersSetResponseDto_Fields]
  Ok(Signed((), session))
}

// body={"illustration_ids": "1024213"}
// See [Wonder_Api_HomeCurrentMemberSetRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct HomeCurrentMemberSetRequest {
  pub illustration_id: i64,
}

/// Called when moving from home screen if character was changed.
pub async fn home_current_member_set(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<HomeCurrentMemberSetRequest>,
) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set home_current_illustration_id = $1
      where id = $2
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&params.illustration_id, &session.user_id])
    .await
    .context("failed to execute query")?;
  info!(?params.illustration_id, "updated home current member");

  // See [Wonder_Api_HomeCurrentMemberSetResponseDto_Fields]
  Ok(Signed((), session))
}

// body={"user_character_id": "1024213"}
#[derive(Debug, Deserialize)]
pub struct MissionHomeRequest {
  pub user_character_id: i32,
}

pub async fn mission_home(Params(params): Params<MissionHomeRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: mission_home");

  // See [Wonder_Api_MissionHomeResponseDto_Fields]
  Ok(Unsigned(()))
}
