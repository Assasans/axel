use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct MaintenanceCheck {
  pub typestatus: u32,
  #[serde(rename = "systemID")]
  pub system_id: Option<()>,
}

impl CallCustom for MaintenanceCheck {}

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(MaintenanceCheck {
      typestatus: 0,
      system_id: None,
    })),
    false,
  ))
}
