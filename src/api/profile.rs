use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct Profile {
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  #[serde(with = "crate::string_as_base64")]
  pub profile: String,
  pub icon: u32,
  pub honor_id: u32,
  pub display_play_data: Vec<DisplayPlayData>,
}

impl CallCustom for Profile {}

#[derive(Debug, Serialize)]
pub struct DisplayPlayData {
  #[serde(rename = "type")]
  pub kind: u32,
  pub value: i32,
  pub display_status: u32,
}

impl DisplayPlayData {
  pub fn new(kind: u32, value: i32, display_status: u32) -> Self {
    Self {
      kind,
      value,
      display_status,
    }
  }
}

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(Profile {
      name: "Aqua".to_string(),
      profile: "Wahhh! Kazuma, he! Kazuma, he wahhh!".to_string(),
      icon: 0,
      honor_id: 62010250,
      display_play_data: vec![
        DisplayPlayData::new(1, 2, 1),
        DisplayPlayData::new(4, 14, 1),
        DisplayPlayData::new(2, -1, 1),
        DisplayPlayData::new(3, 3, 1),
        DisplayPlayData::new(5, 1722883930, 1),
        DisplayPlayData::new(6, -2, 1),
        DisplayPlayData::new(7, 1, 1),
      ],
    })),
    true,
  ))
}
