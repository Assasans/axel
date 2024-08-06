use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Default, Debug, Serialize)]
pub struct GachaInfo {
  pub gacha: Vec<GachaItem>,
}

impl CallCustom for GachaInfo {}

#[derive(Default, Debug, Serialize)]
pub struct GachaItem {
  pub gachaid: u32,
  #[serde(with = "crate::bool_as_int")]
  pub daily: bool,
  pub type1: String,
  pub val1: i64,
  pub type10: String,
  pub val10: i64,
  pub ticket: i64,
  pub ticket_num: i64,
  pub draw_count: i64,
  pub remain_draw_count: i64,
  pub upperlimitcount: i64,
  pub user_story_id: i64,
  pub stepup_bonus: Option<()>,
  pub random_bonus: Option<()>,
  pub stepup_info: Option<()>,
  pub select_info: Option<()>,
  pub continuation_info: Option<()>,
  pub member_select_info: Option<()>,
  pub first_free_ids: Vec<()>,
}

impl GachaItem {
  pub fn new_simple(id: u32) -> Self {
    Self {
      gachaid: id,
      ticket: 2020,
      ..Default::default()
    }
  }
}

#[derive(Debug, Serialize)]
pub struct GachaTutorial {
  pub gacha_id: u32,
  pub goods: Vec<GachaGoodItem>,
}

impl CallCustom for GachaTutorial {}

#[derive(Debug, Serialize)]
pub struct GachaGoodItem {
  pub item_type: u8,
  pub item_id: u32,
  pub item_num: u32,
  #[serde(with = "crate::bool_as_int")]
  pub is_new: bool,
}

impl GachaGoodItem {
  pub fn new(item_type: u8, item_id: u32, item_num: u32, is_new: bool) -> Self {
    Self {
      item_type,
      item_id,
      item_num,
      is_new,
    }
  }
}
