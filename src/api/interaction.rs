use std::sync::Arc;

use jwt_simple::prelude::Serialize;
use serde::Serializer;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::intimacy_rank::get_intimacy_rank_calculator;
use crate::user::session::Session;

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
    get_intimacy_rank_calculator().get_rank(self.rank_progress)
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

pub async fn interaction(session: Arc<Session>) -> impl IntoHandlerResponse {
  let max_xp = get_intimacy_rank_calculator().get_xp_for_rank(50).unwrap();

  Ok(Signed(
    InteractionResponse {
      characters: vec![
        Character::new(100, max_xp, "".to_owned(), 1000101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(101, max_xp, "".to_owned(), 1010101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(102, max_xp, "".to_owned(), 1020101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(103, max_xp, "".to_owned(), 1030101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(106, max_xp, "".to_owned(), 1060101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(108, max_xp, "".to_owned(), 1080101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(109, max_xp, "".to_owned(), 1090101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(112, max_xp, "".to_owned(), 1120101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(113, max_xp, "".to_owned(), 1130101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(115, max_xp, "".to_owned(), 1150101, 0, [0, 0, 0, 0], "".to_owned()),
        Character::new(128, max_xp, "".to_owned(), 1280101, 0, [0, 0, 0, 0], "".to_owned()),
      ],
    },
    session,
  ))
}
