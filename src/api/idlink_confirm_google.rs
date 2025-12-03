use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

#[derive(Debug, Serialize)]
pub struct IdLinkConfirmGoogle {
  #[serde(with = "crate::bool_as_int")]
  pub islink: bool,
}

impl CallCustom for IdLinkConfirmGoogle {}

pub async fn idlink_confirm_google(_request: ApiRequest) -> impl IntoHandlerResponse {
  Unsigned(CallResponse::new_success(Box::new(IdLinkConfirmGoogle {
    islink: false,
  })))
}
