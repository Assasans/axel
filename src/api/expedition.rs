use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::AppState;
use crate::api::interaction::Character;
use crate::api::master_all::get_masters;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;

// See [Wonder_Api_ExpeditiontopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionTop {
  #[serde(rename = "expeditioninfo")]
  pub expeditions: Vec<ExpeditionInfo>,
  #[serde(rename = "bonuspack")]
  pub bonus_pack: i32,
}

impl CallCustom for ExpeditionTop {}

// See [Wonder_Api_ExpeditiontopExpeditioninfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionInfo {
  pub expedition_id: i32,
  pub character_id: i64,
  /// - 0 - invalid;
  /// - 1 - "Working", claim available if >= 864 seconds passed since [start_time];
  /// - 2 - "Now Hiring", character can be assigned;
  /// - 3 - "Job Complete", claim available (max).
  pub status: i32,
  // See [Wonder.UI.Data.ExpeditionData$$GetElapsedTime].
  // See [Wonder.UI.Data.ExpeditionData$$IsReward]:
  // > `return System_TimeSpan__get_TotalSeconds(v11, 0LL) >= 864.0;`
  /// Start time, relative to server time.
  #[serde(rename = "starttime")]
  pub start_time: String,
}

// Reference: https://youtu.be/yhJJ8oCST-4
pub async fn expedition_top() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let expeditions: Vec<Value> = serde_json::from_str(&masters["expedition"].master_decompressed).unwrap();

  Ok(Unsigned(ExpeditionTop {
    expeditions: vec![
      ExpeditionInfo {
        expedition_id: 4,
        character_id: 0,
        status: 2,
        start_time: "".to_string(),
      },
      ExpeditionInfo {
        expedition_id: 1,
        character_id: 101,
        status: 1,
        start_time: "2025/12/23 12:00".to_string(),
      },
      ExpeditionInfo {
        expedition_id: 2,
        character_id: 102,
        status: 3,
        start_time: "2025/12/23 13:00".to_string(),
      },
    ],
    // expeditions: expeditions
    // .iter()
    // .map(|expedition| ExpeditionInfo {
    //   expedition_id: expedition["expedition_id"].as_str().unwrap().parse().unwrap(),
    //   character_id: 102,
    //   start_time: "".to_string(),
    // })
    // .collect(),
    bonus_pack: 1,
  }))
}

// See [Wonder_Api_ExpeditioncharacterResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionCharacter {
  pub characters: Vec<ExpeditionCharacterInfo>,
}

impl CallCustom for ExpeditionCharacter {}

// See [Wonder_Api_ExpeditioncharacterCharactersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionCharacterInfo {
  /// Opaque value to uniquely identify selected character.
  /// Used in [ExpeditionSetRequest] and [ExpeditionUpdate].
  pub id: i32,
  pub character_id: i64,
  pub rank: i32,
  pub rank_progress: i32,
}

pub async fn expedition_character(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  let client = state.get_database_client().await?;
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

  Ok(Unsigned(ExpeditionCharacter {
    characters: rows
      .iter()
      .map(|row| {
        Character::new(
          row.get(0),
          row.get(1),
          "".to_string(),
          0,
          0,
          [0, 0, 0, 0],
          "".to_string(),
        )
      })
      .map(|character| ExpeditionCharacterInfo {
        id: character.character_id as i32,
        character_id: character.character_id,
        rank: character.rank(),
        rank_progress: character.rank_progress,
      })
      .collect(),
  }))
}

// See [Wonder_Api_ExpeditionsetResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionSet {
  /// Does not seem to be used anywhere
  pub updates: Vec<ExpeditionUpdate>,
  /// Displayed only if [items] is not empty
  pub money: i32,
  /// If not empty, shows a "Job Payment Received" popup, also see [expedition_set] docs.
  pub items: Vec<ExpeditionItem>,
}

impl CallCustom for ExpeditionSet {}

// See [Wonder_Api_ExpeditionsetUpdatesResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionUpdate {
  pub expedition_id: i32,
  pub user_character_id: i32,
  pub love: i32,
  #[serde(rename = "starttime")]
  pub start_time: String,
}

// See [Wonder_Api_ExpeditionsetItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

/// If both [expedition_id] and [user_character_id] are zero, "Claim All" is performed.
#[derive(Debug, Deserialize)]
pub struct ExpeditionSetRequest {
  pub expedition_id: i32,
  pub user_character_id: i32,
}

/// Called when either assigning a character or claiming rewards.
/// If claiming rewards, "Job Payment Received" popup is always shown even if [ExpeditionSet.items] is empty.
pub async fn expedition_set(Params(params): Params<ExpeditionSetRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: expedition_set");

  Ok(Unsigned(ExpeditionSet {
    updates: vec![
      // ExpeditionUpdate {
      //   expedition_id: 4,
      //   user_character_id: 102,
      //   love: 5,
      //   start_time: "2025/12/23 13:00".to_string(),
      // }
    ],
    // - "[...] so I owe nearly 100,000 eris in this bar!"
    money: -100_000,
    items: vec![
      // ExpeditionItem {
      //   item_type: RemoteDataItemType::Money.into(),
      //   item_id: 1,
      //   item_num: 42,
      // }
    ],
  }))
}
