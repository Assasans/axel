use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

#[derive(Debug, Serialize)]
pub struct MaintenanceCheck {
  pub typestatus: u32,
  #[serde(rename = "systemID")]
  pub system_id: Option<()>,
}

impl CallCustom for MaintenanceCheck {}

pub async fn maintenance_check(_request: ApiRequest) -> impl IntoHandlerResponse {
  Unsigned(MaintenanceCheck {
    typestatus: 0,
    system_id: None,
  })
}
