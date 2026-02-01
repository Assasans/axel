//! Reference: https://youtu.be/A47Qcj323C0

use crate::api::master_all::get_master_manager;
use crate::api::RemoteDataItemType;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use serde::{Deserialize, Serialize};
use tracing::warn;

// See [Wonder_Api_PanelMissionListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionListResponse {
  pub panel_missions: Vec<PanelMissionSummary>,
}

impl CallCustom for PanelMissionListResponse {}

// See [Wonder_Api_PanelMissionSummaryResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionSummary {
  pub panel_group_id: i32,
  pub achieved_mission_count: i32,
  pub receivable_mission_count: i32,
  pub is_challengeable: bool,
  pub is_new: bool,
}

pub async fn panel_mission_list() -> impl IntoHandlerResponse {
  let mission_groups = get_master_manager().get_master("mission_panel_group");

  Ok(Unsigned(PanelMissionListResponse {
    panel_missions: mission_groups
      .iter()
      .map(|mission| PanelMissionSummary {
        panel_group_id: mission["panel_group_id"].as_str().unwrap().parse::<i32>().unwrap(),
        achieved_mission_count: 0,
        receivable_mission_count: 0,
        is_challengeable: true,
        is_new: true,
      })
      .collect(),
  }))
}

// See [Wonder_Api_PanelMissionRequest_Fields]
#[derive(Debug, Deserialize)]
pub struct PanelMissionRequest {
  pub panel_group_id: i32,
}

// See [Wonder_Api_PanelMissionResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionResponse {
  pub missions: Vec<PanelMissionDetail>,
  pub rewards: Vec<PanelMissionReward>,
  // "Bonus Acquired"
  pub complete_reward: PanelMissionCompleteReward,
}

impl CallCustom for PanelMissionResponse {}

// See [Wonder_Api_PanelMissionDetailResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionDetail {
  pub mission_id: i32,
  pub progress: i32,
  pub max_progress: i32,
  pub is_challengeable: bool,
}

// See [Wonder_Api_PanelMissionRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionReward {
  pub mission_id: i32,
  #[serde(rename = "goodsid")]
  pub goods_id: Vec<MissionGood>,
  /// "You were unable to receive the following items, so they have been sent to your Gift Box."
  #[serde(rename = "unreceived_goodsid")]
  pub unreceived_goods_id: Vec<MissionGood>,
}

// See [Wonder_Api_PanelMissionCompleteRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PanelMissionCompleteReward {
  #[serde(rename = "goodsid")]
  pub goods_id: Vec<MissionGood>,
  #[serde(rename = "unreceived_goodsid")]
  pub unreceived_goods_id: Vec<MissionGood>,
}

// See [Wonder_Api_MissionGoodsidResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MissionGood {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
}

pub async fn panel_mission(Params(params): Params<PanelMissionRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: panel_mission");

  let missions = get_master_manager().get_master("mission_panel");
  let missions = missions
    .iter()
    .filter(|mission| mission["panel_group_id"].as_str().unwrap().parse::<i32>().unwrap() == params.panel_group_id)
    .collect::<Vec<_>>();

  Ok(Unsigned(PanelMissionResponse {
    missions: missions
      .iter()
      .map(|mission| PanelMissionDetail {
        mission_id: mission["mission_id"].as_str().unwrap().parse::<i32>().unwrap(),
        progress: 0,
        max_progress: 100,
        is_challengeable: true,
      })
      .collect(),
    rewards: vec![/*PanelMissionReward {
      mission_id: missions.first().unwrap()["mission_id"]
        .as_str()
        .unwrap()
        .parse::<i32>()
        .unwrap(),
      goods_id: vec![MissionGood {
        item_type: RemoteDataItemType::RealMoney.into(),
        item_id: 1,
        item_num: 4242,
      }],
      unreceived_goods_id: vec![MissionGood {
        item_type: RemoteDataItemType::Money.into(),
        item_id: 1,
        item_num: 50000,
      }],
    }*/],
    complete_reward: PanelMissionCompleteReward {
      goods_id: vec![
        MissionGood {
          item_type: RemoteDataItemType::RealMoney.into(),
          item_id: 1,
          item_num: 100,
        }
      ],
      unreceived_goods_id: vec![
        /*MissionGood {
          item_type: RemoteDataItemType::Money.into(),
          item_id: 1,
          item_num: 100000,
        }*/
      ],
    },
  }))
}
