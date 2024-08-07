use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct LoginBonus {
  pub goods: Vec<LoginBonusGood>,
  pub omikuji: Omikuji,
  #[serde(rename = "random_loginbonus_result")]
  pub random_login_bonus: RandomLoginBonus,
  #[serde(rename = "roulette_loginbonus_result")]
  pub roulette_login_bonus: RouletteLoginBonus,
}

impl CallCustom for LoginBonus {}

#[derive(Debug, Serialize)]
pub struct LoginBonusGood {
  pub login_bonus_master_id: u32,
  pub day_count: u32,
  pub itemtype: u32,
  pub itemid: u32,
  pub itemnum: u32,
}

impl LoginBonusGood {
  pub fn new(login_bonus_master_id: u32, day_count: u32, itemtype: u32, itemid: u32, itemnum: u32) -> Self {
    Self {
      login_bonus_master_id,
      day_count,
      itemtype,
      itemid,
      itemnum,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct Omikuji {
  pub omikuji_id: u32,
  pub fortune_id: u32,
}

#[derive(Debug, Serialize)]
pub struct RandomLoginBonus {
  pub random_loginbonus_id: u32,
  pub lot_id: u32,
  pub story_id: u32,
  pub user_story_id: u32,
  pub days: Vec<()>,
}

#[derive(Debug, Serialize)]
pub struct RouletteLoginBonus {
  pub roulette_loginbonus_id: u32,
  pub result_pattern_id: u32,
  pub roulette_view_id: u32,
  pub days: Vec<()>,
  pub sns_share_results: Vec<()>,
}
