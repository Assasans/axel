use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct MaintenanceCheck {
  pub typestatus: u32,
  #[serde(rename = "systemID")]
  pub system_id: Option<()>,
}

impl CallCustom for MaintenanceCheck {}
