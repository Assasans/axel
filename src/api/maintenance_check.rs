use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_MaintenancecheckResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MaintenanceCheck {
  pub typestatus: i32,
  #[serde(rename = "systemID")]
  pub system_id: Option<Vec<i32>>,
}

impl CallCustom for MaintenanceCheck {}

pub async fn maintenance_check() -> impl IntoHandlerResponse {
  Unsigned(MaintenanceCheck {
    typestatus: 0,
    system_id: None,
  })
}
