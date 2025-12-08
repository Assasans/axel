use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use tracing::warn;

use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_IdlinkConfirmGoogleResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct IdLinkConfirmGoogle {
  #[serde(with = "crate::bool_as_int")]
  pub islink: bool,
}

impl CallCustom for IdLinkConfirmGoogle {}

#[derive(Debug, Deserialize)]
pub struct IdLinkConfirmGoogleRequest {
  pub uuid: String,
}

pub async fn idlink_confirm_google(Params(params): Params<IdLinkConfirmGoogleRequest>) -> impl IntoHandlerResponse {
  warn!(?params.uuid, "encountered stub: idlink_confirm_google");
  Unsigned(IdLinkConfirmGoogle { islink: false })
}
