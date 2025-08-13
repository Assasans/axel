use serde::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

// See [Wonder_Api_AssistMakeNoticeResponseDto_Fields] - no fields
pub async fn assist_make_notice(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  // TODO: This should probably return remote data or notification data, but I have no dumps for it
  Ok((CallResponse::new_success(Box::new(())), false))
}

// See [Wonder_Api_AssistMakeNoticeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AssistMakeList {
  pub assist_detail_id_list: Vec<i64>,
}

impl CallCustom for AssistMakeList {}

pub async fn assist_make_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(AssistMakeList {
      assist_detail_id_list: vec![],
    })),
    false,
  ))
}
