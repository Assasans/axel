use serde_json::json;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

pub async fn weapon_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "items": [],
  })));
  Ok((response, false))
}

pub async fn accessory_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "items": [],
  })));
  Ok((response, false))
}
