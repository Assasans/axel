use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct Home {
  pub multi_battle_invitation: Option<()>,
  pub member_info: MemberInfo,
  pub advertisement_data: AdvertisementData,
  pub display_plan_map: bool,
}

impl CallCustom for Home {}

#[derive(Debug, Serialize)]
pub struct MemberInfo {
  pub current_member_id: u32,
  pub member_ids: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct AdvertisementData {
  pub id: u32,
  pub reward_type: u32,
  pub status: u32,
}

pub async fn route(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(Home {
      multi_battle_invitation: None,
      member_info: MemberInfo {
        current_member_id: 1011100,
        member_ids: vec![1011100, 0, 0, 0, 0],
      },
      advertisement_data: AdvertisementData {
        id: 10006,
        reward_type: 1,
        status: 0,
      },
      display_plan_map: false,
    })),
    true,
  ))
}
