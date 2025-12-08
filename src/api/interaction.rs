use std::sync::Arc;

use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

// See [Wonder_Api_InteractionResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct InteractionResponse {
  pub characters: Vec<Character>,
}

impl CallCustom for InteractionResponse {}

// See [Wonder_Api_InteractionCharactersResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Character {
  pub character_id: i64,
  pub rank: i32,
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
    rank: i32,
    rank_progress: i32,
    voice: String,
    character_enhance_stage_id: i32,
    character_enhance_badge: i32,
    character_enhance_released_count: [i32; 4],
    bg: String,
  ) -> Self {
    Self {
      character_id,
      rank,
      rank_progress,
      voice,
      character_enhance_stage_id,
      character_enhance_badge,
      character_enhance_released_count,
      bg,
    }
  }
}

pub async fn interaction(session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Signed(
    InteractionResponse {
      characters: vec![
        Character::new(100, 1, 4, "".to_owned(), 1000101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(101, 1, 4, "".to_owned(), 1010101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(102, 1, 0, "".to_owned(), 1020101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(103, 1, 0, "".to_owned(), 1030101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(106, 1, 4, "".to_owned(), 1060101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(108, 1, 0, "".to_owned(), 1080101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(109, 1, 0, "".to_owned(), 1090101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(112, 1, 0, "".to_owned(), 1120101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(113, 1, 0, "".to_owned(), 1130101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(115, 1, 0, "".to_owned(), 1150101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(128, 1, 0, "".to_owned(), 1280101, 0, [0, 0, 0, 0], "".to_owned()),
      ],
    },
    session,
  ))
}
