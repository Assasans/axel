use crate::api::master_all::get_masters;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::level::get_intimacy_level_calculator;
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use jwt_simple::prelude::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

// See [Wonder_Api_InteractionResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct InteractionResponse {
  pub characters: Vec<Character>,
}

impl CallCustom for InteractionResponse {}

#[derive(Debug)]
pub struct Character {
  pub character_id: i64,
  /// "Affinity points". Does not wrap to zero when reaching next level.
  /// See 'intimacy_exp' master data.
  pub rank_progress: i32,
  pub voice: String,
  pub character_enhance_stage_id: i32,
  pub character_enhance_badge: i32,
  pub character_enhance_released_count: [i32; 4],
  pub bg: String,
}

// See [Wonder_Api_InteractionCharactersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct CharacterWire {
  pub character_id: i64,
  /// "Affinity"
  pub rank: i32,
  /// "Affinity points".
  pub rank_progress: i32,
  pub voice: String,
  pub character_enhance_stage_id: i32,
  pub character_enhance_badge: i32,
  pub character_enhance_released_count: [i32; 4],
  pub bg: String,
}

impl Character {
  pub fn new(
    character_id: i64,
    rank_progress: i32,
    voice: String,
    character_enhance_stage_id: i32,
    character_enhance_badge: i32,
    character_enhance_released_count: [i32; 4],
    bg: String,
  ) -> Self {
    Self {
      character_id,
      rank_progress,
      voice,
      character_enhance_stage_id,
      character_enhance_badge,
      character_enhance_released_count,
      bg,
    }
  }

  pub fn rank(&self) -> i32 {
    get_intimacy_level_calculator().get_level(self.rank_progress)
  }
}

impl Serialize for Character {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let wire = CharacterWire {
      character_id: self.character_id,
      rank: self.rank(),
      rank_progress: self.rank_progress,
      voice: self.voice.clone(),
      character_enhance_stage_id: self.character_enhance_stage_id,
      character_enhance_badge: self.character_enhance_badge,
      character_enhance_released_count: self.character_enhance_released_count,
      bg: self.bg.clone(),
    };
    wire.serialize(serializer)
  }
}

pub fn parse_date(master_date: &str) -> Option<chrono::NaiveDateTime> {
  if master_date == "0" {
    return None;
  }

  let formats = ["%Y/%m/%d %H:%M:%S", "%Y/%m/%d %H:%M"];
  for format in &formats {
    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(master_date, format) {
      return Some(date);
    }
  }
  todo!("unhandled date format: {}", master_date);
}

pub async fn interaction(state: Arc<AppState>, session: Arc<Session>) -> impl IntoHandlerResponse {
  // let masters = get_masters().await;
  // let characters: Vec<Value> = serde_json::from_str(&masters["character"].master_decompressed).unwrap();

  let client = state.pool.get().await.context("failed to get database connection")?;
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
  info!(?rows, "get friend info query executed");

  Ok(Signed(
    InteractionResponse {
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
        .collect(),
    },
    session,
  ))
}
