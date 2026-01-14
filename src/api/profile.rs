use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use serde_json::Value;
use tracing::{info, trace, warn};

use crate::api::master_all::get_masters;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;

// See [Wonder_Api_ProfileResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Profile {
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile: String,
  /// "Favorite character" in game
  pub icon: i64,
  pub honor_id: i64,
  pub display_play_data: Vec<DisplayPlayData>,
}

impl CallCustom for Profile {}

// See [Wonder_Api_ProfileDisplayPlayDataResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DisplayPlayData {
  #[serde(rename = "type")]
  pub kind: i32,
  pub value: i64,
  pub display_status: i32,
}

impl DisplayPlayData {
  pub fn new(kind: i32, value: i64, display_status: i32) -> Self {
    Self {
      kind,
      value,
      display_status,
    }
  }
}

pub async fn profile(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select
        users.username,
        users.about_me,
        users.favorite_member,
        users.honor,
        (select max(last_used) from user_devices where user_devices.user_id = users.id) as most_recent_last_used
      from users
      where id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let rows = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;
  trace!(?rows, "get profile query executed");
  let row = rows
    .first()
    .ok_or_else(|| anyhow::anyhow!("no profile found for user"))?;

  let username: Option<String> = row.get(0);
  let about_me: Option<String> = row.get(1);
  let favorite_member: i64 = row.get(2);
  let honor: i64 = row.get(3);
  let last_used: Option<DateTime<Utc>> = row.get(4);
  let last_used = last_used.unwrap_or_else(Utc::now);

  Ok(Signed(
    Profile {
      // If for some reason username was not set during tutorial, use empty string
      name: username.unwrap_or_default(),
      profile: about_me.unwrap_or_default(),
      icon: favorite_member,
      honor_id: honor,
      display_play_data: vec![
        // "Player rank"
        DisplayPlayData::new(1, 2, 1),
        // "Character gallery characters"
        DisplayPlayData::new(4, 14, 1),
        // "Party power"
        DisplayPlayData::new(2, -1, 1),
        // "Total crowns earned"
        DisplayPlayData::new(3, 3, 1),
        // "Latest login", clamped at 1 month at the client
        DisplayPlayData::new(5, last_used.timestamp(), 1),
        // "Arena ranking": -2 - calculating ranking, -1 - unranked, 0 - hide, 1+ - rank
        DisplayPlayData::new(6, -2, 1),
        // "Affinity"
        DisplayPlayData::new(7, 1, 1),
      ],
    },
    session,
  ))
}

#[derive(Debug, Deserialize)]
pub struct SetProfileRequest {
  #[serde(with = "crate::string_as_base64")]
  pub profile: String,
}

// profile=V2FoaGghwqBLYXp1bWEswqBoZSHCoEthenVtYSzCoGhlwqB3YWhoaCE
/// Set "about me"
pub async fn set_profile(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<SetProfileRequest>,
) -> impl IntoHandlerResponse {
  // For some reason client sends 0x20 SPACE as 0xA0 NBSP
  let about_me = params.profile.replace('\u{a0}', " ");

  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set about_me = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &about_me])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?about_me, "about me updated");

  // See [Wonder_Api_SetprofileResponseDto_Fields]
  Ok(Unsigned(()))
}

#[derive(Debug, Deserialize)]
pub struct SetNameRequest {
  #[serde(rename = "name", with = "crate::string_as_base64")]
  pub username: String,
}

pub async fn set_name(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<SetNameRequest>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set username = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &params.username])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.username, "username updated");

  session.set_cached_username(Some(params.username.clone()));

  Ok(Signed((), session))
}

pub async fn delete_account(session: Arc<Session>) -> impl IntoHandlerResponse {
  warn!("encountered stub: delete_account");

  // See [Wonder_Api_DeleteAccountResponseDto_Fields]
  Ok(Signed((), session))
}

#[derive(Debug, Serialize)]
pub struct HonorList {
  pub honor_list: Vec<HonorItem>,
}

impl CallCustom for HonorList {}

#[derive(Debug, Serialize)]
pub struct HonorItem {
  pub honor_id: u32,
  pub is_selected: bool,
  pub is_new: bool,
}

impl HonorItem {
  pub fn new(honor_id: u32, is_selected: bool, is_new: bool) -> Self {
    Self {
      honor_id,
      is_selected,
      is_new,
    }
  }
}

pub async fn honor_list(session: Arc<Session>) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let honors: Vec<Value> = serde_json::from_str(&masters["honor"].master_decompressed).unwrap();

  let selected_honor_id = 62010250;

  Ok(Signed(
    HonorList {
      honor_list: honors
        .iter()
        .map(|honor| {
          let honor_id: u32 = honor["id"].as_str().unwrap().parse().unwrap();
          HonorItem::new(honor_id, honor_id == selected_honor_id, false)
        })
        .collect(),
    },
    session,
  ))
}

#[derive(Debug, Deserialize)]
pub struct HonorSetRequest {
  pub honor_id: u32,
}

// honor_id=60000000
pub async fn honor_set(
  state: Arc<AppState>,
  Params(params): Params<HonorSetRequest>,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set honor = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &(params.honor_id as i64)])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.honor_id, "honor updated");

  // See [Wonder_Api_HonorSetResponseDto_Fields]
  Ok(Unsigned(()))
}

#[derive(Debug, Deserialize)]
pub struct SetIconRequest {
  pub illustration_id: u32,
}

// illustration_id=1143127
pub async fn set_icon(
  state: Arc<AppState>,
  Params(params): Params<SetIconRequest>,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set favorite_member = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &(params.illustration_id as i64)])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.illustration_id, "icon updated");

  // See [Wonder_Api_HonorSetResponseDto_Fields]
  Ok(Unsigned(()))
}
