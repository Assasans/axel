use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_IdlinkConfirmGoogleResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct IdLinkConfirmGoogle {
  #[serde(with = "crate::bool_as_int")]
  pub islink: bool,
}

impl CallCustom for IdLinkConfirmGoogle {}

pub async fn idlink_confirm_google(_request: ApiRequest) -> impl IntoHandlerResponse {
  Unsigned(IdLinkConfirmGoogle { islink: false })
}
