use std::sync::Arc;

use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

// See [Wonder_Api_HomeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Home {
  pub multi_battle_invitation: Option<MultiBattleInvitation>,
  pub member_info: MemberInfo,
  pub advertisement_data: AdvertisementData,
  pub display_plan_map: bool,
}

impl CallCustom for Home {}

// See [Wonder_Api_MultiBattleInvitationResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitation {
  pub event_id: i32,
  pub rooms: Vec<MultiBattleRoom>,
}

// See [Wonder_Api_MultiBattleInvitationRoomResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoom {
  pub room_no: i32,
  pub quest_id: i32,
  pub user_icon: i64,
  #[serde(with = "crate::string_as_base64")]
  pub user_name: String,
}

// See [Wonder_Api_HomeMemberInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MemberInfo {
  pub current_member_id: i64,
  pub member_ids: Vec<i64>,
}

// See [Wonder_Api_AdvertisementDataResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AdvertisementData {
  pub id: i32,
  pub reward_type: i32,
  pub status: i32,
}

pub async fn home(session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Signed(
    Home {
      multi_battle_invitation: None,
      member_info: MemberInfo {
        current_member_id: 1064200,
        member_ids: vec![1011100, 1024213, 1064200, 0, 0],
      },
      advertisement_data: AdvertisementData {
        id: 10006,
        reward_type: 1,
        status: 0,
      },
      display_plan_map: false,
    },
    session,
  ))
}
