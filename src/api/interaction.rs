use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

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
