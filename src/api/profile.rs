use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use jwt_simple::prelude::Serialize;
use tracing::info;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct Profile {
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile: String,
  /// "Favorite character" in game
  pub icon: u32,
  pub honor_id: u32,
  pub display_play_data: Vec<DisplayPlayData>,
}

impl CallCustom for Profile {}

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

pub async fn profile(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select
        users.username,
        users.about_me,
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
  info!(?rows, "get profile query executed");
  let row = rows
    .first()
    .ok_or_else(|| anyhow::anyhow!("no profile found for user"))?;

  let username: Option<String> = row.get(0);
  let about_me: Option<String> = row.get(1);
  let last_used: Option<DateTime<Utc>> = row.get(2);
  let last_used = last_used.unwrap_or_else(|| Utc::now());

  Ok(Signed(
    Profile {
      // If for some reason username was not set during tutorial, use empty string
      name: username.unwrap_or_default(),
      profile: about_me.unwrap_or_default(),
      icon: 1001100,
      honor_id: 62010030,
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
