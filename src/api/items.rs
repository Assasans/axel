use serde_json::json;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

pub async fn weapon_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(json!({
    "items": [],
  })))
}

pub async fn accessory_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(json!({
    "items": [],
  })))
}
