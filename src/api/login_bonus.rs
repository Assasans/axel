use std::sync::Arc;

use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

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

pub async fn login_bonus(_request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Signed(
    CallResponse::new_success(Box::new(LoginBonus {
      goods: vec![
        // LoginBonusGood::new(20001, 1, 3, 1, 1000),
        // LoginBonusGood::new(40266, 1, 21, 17, 1),
        // LoginBonusGood::new(40293, 1, 21, 17, 1),
        // LoginBonusGood::new(40294, 1, 21, 17, 1),
        // LoginBonusGood::new(80029, 1, 8, 1, 800),
      ],
      omikuji: Omikuji {
        omikuji_id: 0,
        fortune_id: 0,
      },
      random_login_bonus: RandomLoginBonus {
        random_loginbonus_id: 0,
        lot_id: 0,
        story_id: 0,
        user_story_id: 0,
        days: vec![],
      },
      roulette_login_bonus: RouletteLoginBonus {
        roulette_loginbonus_id: 0,
        result_pattern_id: 0,
        roulette_view_id: 0,
        days: vec![],
        sns_share_results: vec![],
      },
    })),
    session,
  ))
}
