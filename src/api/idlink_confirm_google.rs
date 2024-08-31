use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct IdLinkConfirmGoogle {
  #[serde(with = "crate::bool_as_int")]
  pub islink: bool,
}

impl CallCustom for IdLinkConfirmGoogle {}

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(IdLinkConfirmGoogle { islink: false })),
    false,
  ))
}
