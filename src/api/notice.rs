use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

#[derive(Debug, Serialize)]
pub struct Notice {
  #[serde(rename = "answerAlarm")]
  pub answer_alarm: String,
}

impl CallCustom for Notice {}

pub async fn notice(_request: ApiRequest) -> impl IntoHandlerResponse {
  Unsigned(CallResponse::new_custom(
    1,
    Box::new(Notice {
      answer_alarm: "fail".to_owned(),
    }),
  ))
}
