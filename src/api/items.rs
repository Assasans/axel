use serde_json::json;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

pub async fn weapon_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "items": [],
  })));
  Ok(Unsigned(response))
}

pub async fn accessory_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "items": [],
  })));
  Ok(Unsigned(response))
}
