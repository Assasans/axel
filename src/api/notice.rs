use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct Notice {
  #[serde(rename = "answerAlarm")]
  pub answer_alarm: String,
}

impl CallCustom for Notice {}

pub async fn route(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_custom(
      1,
      Box::new(Notice {
        answer_alarm: "fail".to_owned(),
      }),
    ),
    false,
  ))
}
