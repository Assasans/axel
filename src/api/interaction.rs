use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct Interaction {
  pub characters: Vec<Character>,
}

impl CallCustom for Interaction {}

#[derive(Debug, Serialize)]
pub struct Character {
  pub character_id: u32,
  pub rank: u32,
  pub rank_progress: u32,
  pub voice: String,
  pub character_enhance_stage_id: u32,
  pub character_enhance_badge: u32,
  pub character_enhance_released_count: [u32; 4],
  pub bg: String,
}

impl Character {
  pub fn new(
    character_id: u32,
    rank: u32,
    rank_progress: u32,
    voice: String,
    character_enhance_stage_id: u32,
    character_enhance_badge: u32,
    character_enhance_released_count: [u32; 4],
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

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(Interaction {
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
    })),
    true,
  ))
}
