use anyhow::Context;
use jwt_simple::prelude::Serialize;
use rand::seq::IteratorRandom;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::api::master_all::get_master_manager;
use crate::api::{NotificationData, RemoteData, RemoteDataCommand, RemoteDataItemType};
use crate::blob::{AddMember, IntoRemoteData, UpdateMember};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::member::MemberPrototype;
use crate::user::session::Session;
use crate::{master, AppState};

// See [Wonder_Api_GachaInfoResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaInfo {
  pub gacha: Vec<GachaItem>,
}

impl CallCustom for GachaInfo {}

// See [Wonder_Api_GachainfoGachaResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct GachaItem {
  #[serde(rename = "gachaid")]
  pub gacha_id: i32,
  #[serde(with = "crate::bool_as_int")]
  pub daily: bool,
  pub type1: String,
  pub val1: i32,
  pub type10: String,
  pub val10: i32,
  pub ticket: i32,
  pub ticket_num: i32,
  pub draw_count: i32,
  pub remain_draw_count: i32,
  pub upperlimitcount: i32,
  pub user_story_id: i32,
  pub stepup_bonus: Option<StepupBonus>,
  pub random_bonus: Option<RandomBonus>,
  pub stepup_info: Option<StepupInfo>,
  pub select_info: Option<SelectInfo>,
  pub continuation_info: Option<ContinuationInfo>,
  pub member_select_info: Option<MemberSelectInfo>,
  pub first_free_ids: Vec<i32>,
}

// See [Wonder_Api_GachainfoStepupBonusResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct StepupBonus {
  pub gacha_item_id: u32,
  pub stepup_bonus_id: u32,
  pub step: u32,
}

// See [Wonder_Api_GachainfoRandomBonusResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct RandomBonus {
  pub gacha_item_id: i32,
  pub items: Vec<RandomBonusItem>,
}

// See [Wonder_Api_GachainfoRandomBonusItemResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct RandomBonusItem {
  pub pack_id: i64,
  pub rate: i32,
}

// See [Wonder_Api_GachainfoStepupResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct StepupInfo {
  pub step: i32,
  #[serde(rename = "loop")]
  pub loop_count: i32,
  pub is_drawable: bool,
}

// See [Wonder_Api_GachainfoSelectResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct SelectInfo {
  pub select_character_id: i32,
  pub select_character_id_list: Vec<i32>,
}

// See [Wonder_Api_GachainfoContinuationResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct ContinuationInfo {
  pub remain_sns_share_count: i32,
  pub next_draw: bool,
}

// See [Wonder_Api_GachainfoMemberSelectResponseDto_Fields]
#[derive(Debug, Clone, Serialize)]
pub struct MemberSelectInfo {
  pub select_member_id_list: Vec<i64>,
}

impl GachaItem {
  pub fn new_simple(gacha_id: i32) -> Self {
    Self {
      gacha_id,
      daily: true,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 1,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    }
  }
}

// See [Wonder_Api_GachaTutorialResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GachaTutorial {
  pub gacha_id: i32,
  pub goods: Vec<GachaGoodItem>,
}

impl CallCustom for GachaTutorial {}

// See [Wonder_Api_GachaTutorialGoodsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GachaGoodItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
  #[serde(with = "crate::bool_as_int")]
  pub is_new: bool,
}

impl GachaGoodItem {
  pub fn new(item_type: i32, item_id: i64, item_num: i32, is_new: bool) -> Self {
    Self {
      item_type,
      item_id,
      item_num,
      is_new,
    }
  }
}

// See [Wonder_Api_GachanormalResponseDto_Fields]
// See [Wonder_Api_GachachainResponseDto_Fields]
// See [Wonder_Api_GachadailyResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaResult {
  pub gacha_id: i32,
  pub goods: Vec<GachaGood>,
  pub bonus_info: Option<BonusInfo>,
  pub bonus_step: Option<StepupBonusStep>,
}

impl CallCustom for GachaResult {}

// See [Wonder_Api_GachaStepupBonusStepResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct StepupBonusStep {
  pub stepup_bonus_id: i32,
  pub old_step: i32,
  pub new_step: i32,
}

// See [Wonder_Api_GachanormalGoodsResponseDto_Fields]
// See [Wonder_Api_GachachainGoodsResponseDto_Fields]
// See [Wonder_Api_GachadailyGoodsResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaGood {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  #[serde(with = "crate::bool_as_int")]
  pub isnew: bool,
}

impl GachaGood {
  pub fn new(itemtype: i32, itemid: i64, itemnum: i32, isnew: bool) -> Self {
    Self {
      itemtype,
      itemid,
      itemnum,
      isnew,
    }
  }
}

// See [Wonder_Api_GachaBonusInfoResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct BonusInfo {
  pub items: Vec<BonusItem>,
  pub rare: i32,
  pub bonus_type: i32,
  pub bonus_animation: String,
}

// See [Wonder_Api_GachaBonusItemResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct BonusItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

pub async fn gacha_info() -> impl IntoHandlerResponse {
  let gacha_items = vec![
    GachaItem {
      gacha_id: 100001,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 6,
      ticket_num: 10,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410321,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 24,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 323083,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 8,
      ticket_num: 30,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 200021,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 17,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 323083,
      daily: true,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 1,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410211,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 19,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 500007,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 1,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410248,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 20,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410305,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 21,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410317,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 23,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410321,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 24,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410326,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 25,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410353,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 26,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410364,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 27,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410393,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 28,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410395,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 29,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: Some(RandomBonus {
        gacha_item_id: 41039501,
        items: vec![
          RandomBonusItem {
            pack_id: 241039501,
            rate: 100,
          },
          RandomBonusItem {
            pack_id: 241039502,
            rate: 1000,
          },
          RandomBonusItem {
            pack_id: 241039503,
            rate: 5000,
          },
          RandomBonusItem {
            pack_id: 241039504,
            rate: 7900,
          },
          RandomBonusItem {
            pack_id: 241039505,
            rate: 8000,
          },
          RandomBonusItem {
            pack_id: 241039506,
            rate: 10000,
          },
          RandomBonusItem {
            pack_id: 241039507,
            rate: 10000,
          },
          RandomBonusItem {
            pack_id: 241039508,
            rate: 10000,
          },
          RandomBonusItem {
            pack_id: 241039509,
            rate: 16000,
          },
          RandomBonusItem {
            pack_id: 241039510,
            rate: 16000,
          },
          RandomBonusItem {
            pack_id: 241039511,
            rate: 16000,
          },
        ],
      }),
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410402,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 30,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410403,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 31,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410410,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 32,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410430,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 33,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410433,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 1,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: Some(MemberSelectInfo {
        select_member_id_list: vec![],
      }),
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410436,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 1,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: Some(SelectInfo {
        select_character_id: 0,
        select_character_id_list: vec![
          100, 101, 102, 103, 104, 105, 106, 107, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 108, 151, 128,
          169,
        ],
      }),
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410437,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 34,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410441,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 35,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410458,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 36,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410486,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 37,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410490,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 38,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410509,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 39,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410522,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 40,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410531,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 41,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410535,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: "limit".to_string(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 5,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410536,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 1,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410544,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 42,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410546,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 43,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410548,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 44,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410550,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 45,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410552,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 46,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410553,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: "limit".to_string(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 3,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410554,
      daily: false,
      type1: "step".to_string(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 0,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: Some(StepupInfo {
        step: 0,
        loop_count: 0,
        is_drawable: true,
      }),
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410627,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 53,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410639,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 54,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410653,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 56,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410661,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 57,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
    GachaItem {
      gacha_id: 410670,
      daily: false,
      type1: String::new(),
      val1: 0,
      type10: String::new(),
      val10: 0,
      ticket: 58,
      ticket_num: 0,
      draw_count: 0,
      remain_draw_count: 0,
      upperlimitcount: 0,
      user_story_id: 0,
      stepup_bonus: None,
      random_bonus: None,
      stepup_info: None,
      select_info: None,
      continuation_info: None,
      member_select_info: None,
      first_free_ids: vec![],
    },
  ];

  let master = get_master_manager().get_master("gacha");
  let master = master
    .iter()
    .map(|gacha| serde_json::from_value::<master::gacha::Gacha>(gacha.clone()).unwrap());
  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaInfo {
    gacha: master
      // .filter(|gacha| gacha.gacha_id != "323083")
      .map(|gacha| GachaItem::new_simple(gacha.gacha_id.parse().unwrap()))
      .collect(),
    // gacha: gacha_items,
  }));
  // response.time = Some(1723059410); // 7 Aug 2024
  // response.time = Some(1722718800); // 4 Aug 2024
  #[rustfmt::skip]
  response.add_notifications(vec![
    // also present 100001 and 500007
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 200012, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410535, msgkey: String::new(), tag: String::new(), },
    NotificationData { cmd: 1, kind: 12, key: 19, value: 323041, msgkey: String::new(), tag: String::new(), }, // missing
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410553, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410123, msgkey: String::new(), tag: String::new(), },
    NotificationData { cmd: 1, kind: 12, key: 19, value: 410436, msgkey: String::new(), tag: String::new(), }, // present
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410565, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410433, msgkey: String::new(), tag: String::new(), }, // present
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410564, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410554, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410554, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410554, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410554, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 12, key: 19, value: 410554, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 26, key: 200012, value: 1, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 26, key: 323083, value: 1, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 26, key: 410436, value: 1, msgkey: String::new(), tag: String::new(), }, // present ref
    NotificationData { cmd: 1, kind: 26, key: 323041, value: 1, msgkey: String::new(), tag: String::new(), }, // missing ref
    // NotificationData { cmd: 1, kind: 26, key: 410554, value: 1, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 27, key: 410535, value: 0, msgkey: String::new(), tag: String::new(), },
    // NotificationData { cmd: 1, kind: 27, key: 410554, value: 0, msgkey: String::new(), tag: String::new(), },
  ]);
  Unsigned(response)
}

#[derive(Debug, Deserialize)]
pub struct GachaTutorialRequest {
  #[serde(rename = "type")]
  pub kind: i32,
}

pub async fn gacha_tutorial(Params(params): Params<GachaTutorialRequest>) -> impl IntoHandlerResponse {
  if params.kind == 1 {
    Unsigned(GachaTutorial {
      gacha_id: 100002,
      goods: vec![
        // Commented out for testing purposes because animations are slow
        GachaGoodItem::new(4, 1032102, 1, true),
        // GachaGoodItem::new(4, 1692100, 1, true),
        // GachaGoodItem::new(4, 1182100, 1, true),
        // GachaGoodItem::new(4, 1092100, 1, true),
        // GachaGoodItem::new(4, 1024126, 1, true),
        // GachaGoodItem::new(4, 1092100, 1, true),
        // GachaGoodItem::new(4, 1002100, 1, true),
        // GachaGoodItem::new(4, 1052102, 1, true),
        // GachaGoodItem::new(4, 1083100, 1, true),
        // GachaGoodItem::new(4, 1174130, 1, true),
      ],
    })
  } else {
    Unsigned(GachaTutorial {
      gacha_id: 100002,
      goods: vec![],
    })
  }
}

pub async fn gacha_tutorial_reward() -> impl IntoHandlerResponse {
  let response = include_str!("../gacha-tutorial-reward.json");
  let response: Value = serde_json::from_str(response).unwrap();
  Unsigned(CallResponse::new_success(Box::new(response)))
}

async fn gacha_impl(
  state: Arc<AppState>,
  session: Arc<Session>,
  gacha_id: i32,
  money_type: i32,
  amount: usize,
) -> impl IntoHandlerResponse {
  let members = get_master_manager().get_master("member");
  let members = members
    .iter()
    .filter(|member| member["rare"].as_str().unwrap().parse::<i32>().unwrap() >= 2)
    .choose_multiple(&mut rand::rng(), amount)
    .into_iter()
    .map(|member| member["id"].as_str().unwrap().parse::<i64>().unwrap())
    .map(MemberPrototype::load_from_id)
    .collect::<Vec<_>>();

  let members = members
    .into_iter()
    .map(|prototype| prototype.create_member(prototype.id as i32))
    .collect::<Vec<_>>();

  let mut client = state.get_database_client().await.unwrap();
  let transaction = client.transaction().await.unwrap();

  #[rustfmt::skip]
  let statement = transaction
    .prepare(/* language=postgresql */ r#"
      with new_member as (
        -- Insert into user_members, if not exists
        insert into user_members (user_id, member_id, xp, promotion_level)
          values ($1, $2, 0, 0)
          on conflict do nothing
          returning 1
      )
      -- Otherwise, insert into user_members_reserve
      insert into user_members_reserve (user_id, member_id)
      select $1, $2
      where not exists (select 1 from new_member);
    "#)
    .await
    .context("failed to prepare statement").unwrap();

  let character_piece_rate = get_master_manager()
    .get_master("gacha_character_piece_rate")
    .iter()
    .map(|item| {
      let rarity: i32 = item["rare"].as_str().unwrap().parse().unwrap();
      let rate: i32 = item["character_piece_rate"].as_str().unwrap().parse().unwrap();
      (rarity, rate)
    })
    .collect::<BTreeMap<_, _>>();
  let character_piece_mapping = get_master_manager()
    .get_master("character_piece")
    .iter()
    .map(|item| {
      let character_id: i64 = item["character_id"].as_str().unwrap().parse().unwrap();
      let piece_id: i64 = item["piece_id"].as_str().unwrap().parse().unwrap();
      (character_id, piece_id)
    })
    .collect::<BTreeMap<_, _>>();

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaResult {
    gacha_id,
    goods: members
      .iter()
      .map(|member| GachaGood::new(RemoteDataItemType::Member.into(), member.id as i64, 1, true))
      .collect::<Vec<_>>(),
    bonus_info: None,
    bonus_step: None,
  }));

  let update = UpdateItemCountBy::new(&transaction).await.unwrap();
  for member in &members {
    let pieces = character_piece_rate.get(&member.prototype.rarity).copied().unwrap_or(0);
    if pieces > 0 {
      let piece_id = character_piece_mapping
        .get(&member.prototype.character_id)
        .copied()
        .unwrap();
      let item = update
        .run(session.user_id, (RemoteDataItemType::CharacterPiece, piece_id), pieces)
        .await
        .context("failed to execute query")
        .unwrap();
      info!(?item, character_id = ?member.prototype.character_id, "granted character pieces");
      response.add_remote_data(item.into_remote_data());
    }

    let rows_affected = transaction
      .execute(&statement, &[&session.user_id, &(member.id as i64)])
      .await
      .context("failed to execute statement")
      .unwrap();
    let is_reserve = rows_affected != 0;
    info!(?member.id, is_reserve, "granted member");
    if is_reserve {
      response.add_remote_data(AddMember::reserve(member.to_member_parameter_wire()).into_remote_data());
    } else {
      response.add_remote_data(AddMember::normal(member.to_member_parameter_wire()).into_remote_data());
    }
  }

  transaction.commit().await.unwrap();

  // Fixed very old bug back from commit [baa1fae0],
  // this was intended to be notification data and not remote data.
  // #[rustfmt::skip]
  // response.add_notifications(vec![
  //   NotificationData::new(1, 7, 2, 11, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 3, 3, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 13, 7, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 7, 34, 2, "show_button".to_string(), "".to_string()),
  //   NotificationData::new(1, 6, 1, 30030001, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 10, 230731, 52307325, "".to_string(), "".to_string()),
  //   NotificationData::new(1, 10, 230831, 52308305, "".to_string(), "".to_string()),
  // ]);

  Unsigned(response)
}

#[derive(Debug, Deserialize)]
pub struct GachaChainRequest {
  pub gacha_id: i32,
  pub money_type: i32,
}

/// 10x pull
pub async fn gacha_chain(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<GachaChainRequest>,
) -> impl IntoHandlerResponse {
  gacha_impl(state, session, params.gacha_id, params.money_type, 10).await
}

#[derive(Debug, Deserialize)]
pub struct GachaNormalRequest {
  pub gacha_id: i32,
  pub money_type: i32,
}

/// 1x pull
pub async fn gacha_normal(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<GachaNormalRequest>,
) -> impl IntoHandlerResponse {
  gacha_impl(state, session, params.gacha_id, params.money_type, 1).await
}

// See [Wonder_Api_GachadailyRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct GachaDailyRequest {
  pub gacha_id: i32,
  pub money_type: i32,
}

pub async fn gacha_daily(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<GachaDailyRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: gacha_daily");

  gacha_impl(state, session, params.gacha_id, params.money_type, 1).await
}

#[derive(Debug, Deserialize)]
pub struct GachaRateRequest {
  pub gacha_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct GachaRateAssistRequest {
  pub gacha_id: i32,
}

// See [Wonder_Api_GacharateAssistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GachaRateAssist {
  pub gacha_id: i32,
  pub rate: Vec<GachaRateRate>,
  pub limit_rate: Vec<GachaRateRate>,
  pub rare_rate: Vec<GachaRateRare>,
  pub limit_rare_rate: Vec<GachaRateRare>,
}

impl CallCustom for GachaRateAssist {}

pub async fn gacha_rate_assist(Params(params): Params<GachaRateAssistRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: gacha_rate_assist");

  Ok(Unsigned(GachaRateAssist {
    gacha_id: params.gacha_id,
    rate: vec![],
    limit_rate: vec![],
    rare_rate: vec![],
    limit_rare_rate: vec![],
  }))
}

// See [Wonder_Api_GachaAssistLogResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GachaAssistLog {
  pub goods: Vec<GachaLogItem>,
}

impl CallCustom for GachaAssistLog {}

// See [Wonder_Api_GachalogGoodsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GachaLogItem {
  #[serde(rename = "itemtype")]
  pub item_type: i32,
  #[serde(rename = "itemid")]
  pub item_id: i64,
  #[serde(rename = "itemnum")]
  pub item_num: i32,
  pub time: String,
  #[serde(rename = "gachaid")]
  pub gacha_id: i32,
}

pub async fn gacha_assist_log() -> impl IntoHandlerResponse {
  warn!("encountered stub: gacha_assist_log");

  Ok(Unsigned(GachaAssistLog { goods: vec![] }))
}

#[derive(Debug)]
pub struct DatabaseGachaRate {
  pub gacha_id: i64,
  pub member: Arc<MemberPrototype>,
  pub probability: Decimal,
  pub probability_pity: Option<Decimal>,
  pub is_rate_up: bool,
  pub details_priority: Option<i32>,
}

// IDA static analysis, not real data
// CLIENT BUG: Clicking "Details" and immediately pressing back causes hard lock.
// TODO: Per-rarity probabilities do not sum to 100% and per-item probabilities can fluctuate,
//  e.g. 0.015% and 0.014%. Maybe original server did some corrections for per-rarity values.
pub async fn gacha_rate(state: Arc<AppState>, Params(params): Params<GachaRateRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: gacha_rate");

  let client = state.get_database_client().await?;
  let rates = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select gacha_id, item_id, probability, probability_pity, is_rate_up, details_priority
        from gacha.rates
        where gacha_id = $1
      "#)
      .await
      .context("failed to prepare statement")?;
    let rows = client
      .query(&statement, &[&(params.gacha_id as i64)])
      .await
      .context("failed to execute query")?;
    rows
      .into_iter()
      .map(|row| DatabaseGachaRate {
        gacha_id: row.get("gacha_id"),
        member: MemberPrototype::load_from_id(row.get("item_id")),
        probability: row.get("probability"),
        probability_pity: row.get("probability_pity"),
        is_rate_up: row.get("is_rate_up"),
        details_priority: row.get("details_priority"),
      })
      .collect::<Vec<_>>()
  };

  let bonus_packs = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select pack_id, probability
        from gacha.bonus_rates
        where gacha_id = $1
      "#)
      .await
      .context("failed to prepare statement")?;
    let rows = client
      .query(&statement, &[&(params.gacha_id as i64)])
      .await
      .context("failed to execute query")?;
    rows
      .into_iter()
      .map(|row| GachaRateBonusItem {
        pack_id: row.get("pack_id"),
        rate: (row.get::<_, Decimal>("rate") * Decimal::from(1000))
          .round()
          .to_i32()
          .unwrap(),
      })
      .collect::<Vec<_>>()
  };

  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaRate {
    gacha_id: params.gacha_id,
    rate: rates
      .iter()
      .map(|rate| GachaRateRate {
        rarity: rate.member.rarity,
        member_id: rate.member.id,
        rate: (rate.probability * Decimal::from(1000)).round().to_i32().unwrap(),
        is_rate_up: rate.is_rate_up,
        is_details_visible: rate.details_priority.is_some(),
        details_priority: rate.details_priority.unwrap_or(0),
      })
      .collect(),
    limitrate: rates
      .iter()
      .filter_map(|rate| {
        rate.probability_pity.map(|pity| GachaRateRate {
          rarity: rate.member.rarity,
          member_id: rate.member.id,
          rate: (pity * Decimal::from(1000)).round().to_i32().unwrap(),
          is_rate_up: rate.is_rate_up,
          is_details_visible: rate.details_priority.is_some(),
          details_priority: rate.details_priority.unwrap_or(0),
        })
      })
      .collect(),
    rarerate: rates
      .iter()
      .fold(BTreeMap::new(), |mut acc, rate| {
        *acc.entry(rate.member.rarity).or_insert(Decimal::ZERO) += rate.probability;
        acc
      })
      .into_iter()
      .map(|(rare, rate)| GachaRateRare {
        rare,
        rate: (rate * Decimal::from(100)).round().to_i32().unwrap(),
      })
      .collect(),
    limitrarerate: rates
      .iter()
      .filter_map(|rate| rate.probability_pity)
      .fold(BTreeMap::new(), |mut acc, pity| {
        let rare = rates
          .iter()
          .find(|r| r.probability_pity == Some(pity))
          .map(|r| r.member.rarity)
          .unwrap_or(0);
        *acc.entry(rare).or_insert(Decimal::ZERO) += pity;
        acc
      })
      .into_iter()
      .map(|(rare, rate)| GachaRateRare {
        rare,
        rate: (rate * Decimal::from(100)).round().to_i32().unwrap(),
      })
      .collect(),
    bonus_per_draw_count: 42,
    bonusrate: bonus_packs,
  }));

  Ok(Unsigned(response))
}

// See [Wonder_Api_GacharateResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaRate {
  pub gacha_id: i32,
  /// "10x recruit (1st-9th use) and 1x recruit appearance rates"
  pub rate: Vec<GachaRateRate>,
  /// "10x recruit (10th use) appearance rates".
  pub limitrate: Vec<GachaRateRate>,
  /// "10x recruit (1st-9th use) and 1x recruit". Derived from [rate].
  pub rarerate: Vec<GachaRateRare>,
  /// "10x recruit (10th use)". Derived from [limitrate].
  pub limitrarerate: Vec<GachaRateRare>,
  pub bonus_per_draw_count: i32,
  /// "10x recruit (draw [bonus_per_draw_count]) set item appearance rates"
  pub bonusrate: Vec<GachaRateBonusItem>,
}

// See [Wonder_Api_GacharateRateResponseDto_Fields]
// See [Wonder_Api_GacharateLimitrateResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaRateRate {
  #[serde(rename = "rare")]
  pub rarity: i32,
  #[serde(rename = "itemid")]
  pub member_id: i64,
  /// Three decimal places, e.g. 10.500 = 10.500%
  pub rate: i32,
  /// Displays "appearance rates up" in UI
  #[serde(rename = "pickup", with = "crate::bool_as_int")]
  pub is_rate_up: bool,
  /// Displays character in "character details" tab
  #[serde(rename = "detailview", with = "crate::bool_as_int")]
  pub is_details_visible: bool,
  #[serde(rename = "detailpriority")]
  pub details_priority: i32,
}

pub struct Rate(pub i32);

// See [Wonder_Api_GacharateRarerateResponseDto_Fields]
// See [Wonder_Api_GacharateLimitrarerateResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaRateRare {
  pub rare: i32,
  /// Three decimal places, e.g. 12.234 = 123.450%
  pub rate: i32,
}

// See [Wonder_Api_GacharateBonusrateResponseDto_Fields]
#[derive(Default, Debug, Serialize)]
pub struct GachaRateBonusItem {
  pub pack_id: i64,
  /// Three decimal places, e.g. 12.345 = 12.345%
  pub rate: i32,
}

impl CallCustom for GachaRate {}

pub async fn gacha_log() -> impl IntoHandlerResponse {
  Unsigned(json!({
    "goods":[{"itemtype":4,"itemid":1063113,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1034100,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1152102,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1083110,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1122100,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1093100,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1132100,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1002102,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1282100,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1064217,"itemnum":1,"time":"2024-08-05 15:03:20","gachaid":100002},{"itemtype":4,"itemid":1192102,"itemnum":1,"time":"2024-08-05 20:17:42","gachaid":200021},{"itemtype":4,"itemid":1102102,"itemnum":1,"time":"2024-08-05 20:17:42","gachaid":200021},{"itemtype":4,"itemid":1143127,"itemnum":1,"time":"2024-08-05 20:17:42","gachaid":200021},{"itemtype":4,"itemid":1162100,"itemnum":1,"time":"2024-08-05 20:17:42","gachaid":200021},{"itemtype":4,"itemid":1192102,"itemnum":1,"time":"2024-08-05 20:17:42","gachaid":200021},{"itemtype":4,"itemid":1012100,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1013100,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012100,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1013116,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012102,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012102,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012102,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012100,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1012102,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535},{"itemtype":4,"itemid":1013116,"itemnum":1,"time":"2024-08-07 19:31:30","gachaid":410535}],"status":0,"time":1723059245,"remotedata":[],"notificationdata":[]
  }))
}
