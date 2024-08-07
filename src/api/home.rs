use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

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
