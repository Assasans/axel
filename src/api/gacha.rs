use jwt_simple::prelude::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::api::{ApiRequest, RemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::master;

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
  pub val1: u32,
  pub type10: String,
  pub val10: u32,
  pub ticket: u32,
  pub ticket_num: u32,
  pub draw_count: u32,
  pub remain_draw_count: u32,
  pub upperlimitcount: u32,
  pub user_story_id: u32,
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
      ticket: 20,
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

#[derive(Default, Debug, Serialize)]
pub struct GachaChain {
  pub gacha_id: u32,
  pub goods: Vec<GachaChainGood>,
  pub bonus_info: BonusInfo,
  pub bonus_step: Option<()>,
}

impl CallCustom for GachaChain {}

#[derive(Default, Debug, Serialize)]
pub struct GachaChainGood {
  pub itemtype: u8,
  pub itemid: u32,
  pub itemnum: u32,
  #[serde(with = "crate::bool_as_int")]
  pub isnew: bool,
}

impl GachaChainGood {
  pub fn new(itemtype: u8, itemid: u32, itemnum: u32, isnew: bool) -> Self {
    Self {
      itemtype,
      itemid,
      itemnum,
      isnew,
    }
  }
}

#[derive(Default, Debug, Serialize)]
pub struct BonusInfo {
  pub items: Vec<BonusItem>,
  pub rare: u32,
  pub bonus_type: u32,
  pub bonus_animation: String,
}

#[derive(Default, Debug, Serialize)]
pub struct BonusItem {
  pub item_type: u32,
  pub item_id: u32,
  pub item_num: u32,
}

pub async fn gacha_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let master = &get_masters().await["gacha"].master_decompressed;
  let master: Vec<master::gacha::Gacha> = serde_json::from_str(master).unwrap();
  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaInfo {
    gacha: master
      .iter()
      .map(|gacha| GachaItem::new_simple(gacha.gacha_id.parse().unwrap()))
      .collect(),
    // gacha: vec![
    //   GachaItem::new_simple(323083, 0),
    // ]
  }));
  response.add_notifications(vec![
    // NotificationData::new(1, 12, 19, 200012, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410535, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410536, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410553, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410123, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410436, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410565, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410433, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410564, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 12, 19, 410554, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 26, 200012, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 26, 323083, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 26, 410436, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 26, 410536, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 26, 410554, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 27, 410535, 1, "".to_owned(), "".to_owned()),
    // NotificationData::new(1, 27, 410554, 0, "".to_owned(), "".to_owned()),
  ]);
  Ok((response, false))
}

pub async fn gacha_tutorial(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  if request.body["type"] == "1" {
    Ok((
      CallResponse::new_success(Box::new(GachaTutorial {
        gacha_id: 100002,
        goods: vec![
          GachaGoodItem::new(4, 1032102, 1, true),
          GachaGoodItem::new(4, 1692100, 1, true),
          GachaGoodItem::new(4, 1182100, 1, true),
          GachaGoodItem::new(4, 1092100, 1, true),
          GachaGoodItem::new(4, 1024126, 1, true),
          GachaGoodItem::new(4, 1092100, 1, true),
          GachaGoodItem::new(4, 1002100, 1, true),
          GachaGoodItem::new(4, 1052102, 1, true),
          GachaGoodItem::new(4, 1083100, 1, true),
          GachaGoodItem::new(4, 1174130, 1, true),
        ],
      })),
      false,
    ))
  } else {
    Ok((
      CallResponse::new_success(Box::new(GachaTutorial {
        gacha_id: 100002,
        goods: vec![],
      })),
      false,
    ))
  }
}

pub async fn gacha_tutorial_reward(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let response = include_str!("../gacha-tutorial-reward.json");
  let response: Value = serde_json::from_str(response).unwrap();
  Ok((CallResponse::new_success(Box::new(response)), false))
}

pub async fn gacha_chain(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let gacha_id: u32 = request.body["gacha_id"].parse().unwrap();
  let money_type: u8 = request.body["money_type"].parse().unwrap();

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(GachaChain {
    gacha_id,
    goods: vec![
      GachaChainGood::new(4, 1032102, 1, true),
      GachaChainGood::new(4, 1012100, 1, true),
      GachaChainGood::new(4, 1013100, 1, true),
      GachaChainGood::new(4, 1012100, 1, true),
      GachaChainGood::new(4, 1013116, 1, true),
      GachaChainGood::new(4, 1012102, 1, true),
      GachaChainGood::new(4, 1012102, 1, true),
      GachaChainGood::new(4, 1012102, 1, true),
      GachaChainGood::new(4, 1012100, 1, true),
      GachaChainGood::new(4, 1012102, 1, true),
      GachaChainGood::new(4, 1013116, 1, true),
    ],
    bonus_info: BonusInfo {
      items: vec![BonusItem {
        item_type: 49,
        item_id: 4,
        item_num: 1,
      }],
      rare: 0,
      bonus_type: 1,
      bonus_animation: "".to_owned(),
    },
    bonus_step: None,
  }));
  response.add_remote_data(vec![
    RemoteData::new(1, 7, 2, 11, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 14, 1, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 3, 3, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 13, 7, 0, 0, "".to_string()),
    RemoteData::new(1, 7, 34, 2, 0, 0, "show_button".to_string()),
    RemoteData::new(1, 6, 1, 30030001, 0, 0, "".to_string()),
    RemoteData::new(1, 10, 230731, 52307325, 0, 0, "".to_string()),
    RemoteData::new(1, 10, 230831, 52308305, 0, 0, "".to_string()),
  ]);

  Ok((response, false))
}
