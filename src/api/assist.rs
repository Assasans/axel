use serde::Serialize;

use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_AssistMakeNoticeResponseDto_Fields] - no fields
pub async fn assist_make_notice() -> impl IntoHandlerResponse {
  // TODO: This should probably return remote data or notification data, but I have no dumps for it
  Ok(Unsigned(()))
}

// See [Wonder_Api_AssistMakeNoticeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AssistMakeList {
  pub assist_detail_id_list: Vec<i64>,
}

impl CallCustom for AssistMakeList {}

pub async fn assist_make_list() -> impl IntoHandlerResponse {
  Ok(Unsigned(AssistMakeList {
    assist_detail_id_list: vec![],
  }))
}
