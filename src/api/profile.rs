use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

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
