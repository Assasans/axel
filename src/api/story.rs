use std::iter;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;

// See [Wonder_Api_StorylistResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct StoryList {
  // These field names...
  #[serde(rename = "storys")]
  pub story_list: Vec<StoryData>,
  pub gettable_sp_story_member_id_list: Vec<i64>,
}

impl CallCustom for StoryList {}

// See [Wonder_Api_StorylistStorysResponseDto_Fields]. Whoever was naming these...
#[derive(Debug, Serialize, Deserialize)]
pub struct StoryData {
  pub user_story_id: i32,
  pub story_type: StoryType,
  pub story_id: i32,
  pub force_release: bool,
  pub selections: Vec<StorySelection>,
  pub status: StoryStatus,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StoryStatus {
  Locked = 0,
  Unlocked = 1,
  Done = 2,
}

// See [Wonder.UI.Data.StoryDataManager$$CreateList]
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StoryType {
  /// "Main Story"
  Main = 1,
  /// "Characters"
  Member = 2,
  /// "KonoSuba"
  Reminiscence = 3,
  /// "Events"
  Event = 4,
  /// "Events" -> "Recruit"
  Gacha = 5,
  /// "Events" -> "Other"
  Other = 6,
}

// See [Wonder_Api_StorylistSelectionsResponseDto_Fields]
#[derive(Debug, Serialize, Deserialize)]
pub struct StorySelection {
  pub selection: Vec<bool>,
}

#[derive(Debug, Deserialize)]
pub struct StoryListRequest {
  /// Seems to be always 0
  #[serde(rename = "type")]
  pub kind: i32,
}

pub async fn story_list(Params(params): Params<StoryListRequest>) -> impl IntoHandlerResponse {
  let index = 0;

  let masters = get_masters().await;
  let parse = |master_name: &str| {
    serde_json::from_str::<Vec<Value>>(&masters[&format!("story_{master_name}")].master_decompressed).unwrap()
  };

  let stories = parse("main")
    .iter()
    .zip(iter::repeat(StoryType::Main))
    .chain(parse("reminiscence").iter().zip(iter::repeat(StoryType::Reminiscence)))
    .chain(parse("etc").iter().zip(iter::repeat(StoryType::Other)))
    .chain(parse("event").iter().zip(iter::repeat(StoryType::Event)))
    .chain(parse("gacha").iter().zip(iter::repeat(StoryType::Gacha)))
    .chain(parse("member").iter().zip(iter::repeat(StoryType::Member)))
    .chain(parse("unique").iter().zip(iter::repeat(StoryType::Reminiscence)))
    .enumerate()
    .map(|(index, (story, kind))| StoryData {
      user_story_id: index as i32,
      story_type: kind,
      story_id: story.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
      force_release: false,
      selections: vec![StorySelection { selection: vec![] }],
      status: StoryStatus::Done,
    })
    .collect::<Vec<_>>();

  // Original dump:
  // let stories = vec![
  //   StoryData {
  //     user_story_id: 1,
  //     story_type: StoryType::Main,
  //     story_id: 100001,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Done,
  //   },
  //   StoryData {
  //     user_story_id: 2,
  //     story_type: StoryType::Main,
  //     story_id: 100101,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Unlocked,
  //   },
  //   StoryData {
  //     user_story_id: 3,
  //     story_type: StoryType::Main,
  //     story_id: 100102,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 4,
  //     story_type: StoryType::Main,
  //     story_id: 100103,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 5,
  //     story_type: StoryType::Main,
  //     story_id: 100104,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 6,
  //     story_type: StoryType::Main,
  //     story_id: 100105,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 7,
  //     story_type: StoryType::Main,
  //     story_id: 100106,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 8,
  //     story_type: StoryType::Main,
  //     story_id: 100201,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 9,
  //     story_type: StoryType::Main,
  //     story_id: 100202,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 10,
  //     story_type: StoryType::Main,
  //     story_id: 100203,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 11,
  //     story_type: StoryType::Main,
  //     story_id: 100204,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 12,
  //     story_type: StoryType::Main,
  //     story_id: 100205,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 13,
  //     story_type: StoryType::Main,
  //     story_id: 100206,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 14,
  //     story_type: StoryType::Main,
  //     story_id: 100301,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 15,
  //     story_type: StoryType::Main,
  //     story_id: 100302,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 16,
  //     story_type: StoryType::Main,
  //     story_id: 100303,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 17,
  //     story_type: StoryType::Main,
  //     story_id: 100304,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 18,
  //     story_type: StoryType::Main,
  //     story_id: 100305,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 19,
  //     story_type: StoryType::Main,
  //     story_id: 100306,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 20,
  //     story_type: StoryType::Main,
  //     story_id: 100401,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 21,
  //     story_type: StoryType::Main,
  //     story_id: 100402,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 22,
  //     story_type: StoryType::Main,
  //     story_id: 100403,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 23,
  //     story_type: StoryType::Main,
  //     story_id: 100404,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 24,
  //     story_type: StoryType::Main,
  //     story_id: 100405,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 25,
  //     story_type: StoryType::Main,
  //     story_id: 100406,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 26,
  //     story_type: StoryType::Main,
  //     story_id: 100501,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 27,
  //     story_type: StoryType::Main,
  //     story_id: 100502,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 28,
  //     story_type: StoryType::Main,
  //     story_id: 100503,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 29,
  //     story_type: StoryType::Main,
  //     story_id: 100504,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 30,
  //     story_type: StoryType::Main,
  //     story_id: 100505,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 31,
  //     story_type: StoryType::Main,
  //     story_id: 100506,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Locked,
  //   },
  //   StoryData {
  //     user_story_id: 32,
  //     story_type: StoryType::Reminiscence,
  //     story_id: 300101,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Done,
  //   },
  //   StoryData {
  //     user_story_id: 33,
  //     story_type: StoryType::Gacha,
  //     story_id: 5230803,
  //     force_release: false,
  //     selections: vec![StorySelection { selection: vec![] }],
  //     status: StoryStatus::Done,
  //   },
  //   StoryData {
  //     user_story_id: 0,
  //     story_type: StoryType::Member,
  //     story_id: 910742280,
  //     force_release: true,
  //     selections: vec![],
  //     status: StoryStatus::Locked,
  //   },
  // ];

  Ok(Unsigned(StoryList {
    story_list: stories,
    gettable_sp_story_member_id_list: vec![],
  }))
}

// See [Wonder_Api_StoryrewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct StoryReward {
  pub reward: Vec<StoryRewardItem>,
}

impl CallCustom for StoryReward {}

// See [Wonder_Api_StoryrewardRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct StoryRewardItem {
  pub item_type: i32,
  pub item_id: i64,
  pub num: i32,
}

// type=3
// route=direct
// is_skip=0
// user_story_id=255
// selections=[]
// story_id=300102
pub async fn story_reward(_request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  Signed(StoryReward { reward: vec![] }, session)
}

// user_story_id=32
// selections=[]
// is_skip=1
pub async fn story_read(request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let user_story_id: i32 = request.body["user_story_id"].parse().unwrap();
  let selections = &request.body["selections"];
  // Probably always set to 1 when StoryStatus::Done
  let is_skip: i32 = request.body["is_skip"].parse().unwrap();

  Signed((), session)
}
