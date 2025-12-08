use std::sync::Arc;

use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

// See [Wonder_Api_LoginbonusResponseDto_Fields]
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

// See [Wonder_Api_LoginbonusGoodsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct LoginBonusGood {
  pub login_bonus_master_id: i32,
  pub day_count: i32,
  pub itemtype: i32,
  pub itemid: i32,
  pub itemnum: i32,
}

impl LoginBonusGood {
  pub fn new(login_bonus_master_id: i32, day_count: i32, itemtype: i32, itemid: i32, itemnum: i32) -> Self {
    Self {
      login_bonus_master_id,
      day_count,
      itemtype,
      itemid,
      itemnum,
    }
  }
}

// See [Wonder_Api_LoginbonusOmikujiResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Omikuji {
  pub omikuji_id: i32,
  pub fortune_id: i32,
}

// See [Wonder_Api_RandomLoginbonusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RandomLoginBonus {
  pub random_loginbonus_id: i32,
  pub lot_id: i32,
  pub story_id: i32,
  pub user_story_id: i32,
  pub days: Vec<RandomLoginBonusDay>,
}

// See [Wonder_Api_RandomLoginbonusDaysResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RandomLoginBonusDay {
  pub day: i32,
  pub pattern_id: i32,
}

// See [Wonder_Api_RouletteLoginbonusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RouletteLoginBonus {
  pub roulette_loginbonus_id: i32,
  pub result_pattern_id: i32,
  pub roulette_view_id: i32,
  pub days: Vec<RouletteLoginBonusDay>,
  pub sns_share_results: Vec<RouletteLoginBonusSnsShareResult>,
}

// See [Wonder_Api_RouletteLoginbonusDaysResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RouletteLoginBonusDay {
  pub day: i32,
  pub result_pattern_id: i32,
}

// See [Wonder_Api_RouletteLoginbonusSnsShareResultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct RouletteLoginBonusSnsShareResult {
  pub day: i32,
  pub result_pattern_id: i32,
}

pub async fn login_bonus(session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Signed(
    LoginBonus {
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
    },
    session,
  ))
}
