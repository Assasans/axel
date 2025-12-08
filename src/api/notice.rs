use jwt_simple::prelude::Serialize;

use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_NoticeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Notice {
  pub text_japanese: Option<String>,
  pub text_english: Option<String>,
  pub text_korean: Option<String>,
  #[serde(rename = "answerAlarm")]
  pub answer_alarm: String,
}

impl CallCustom for Notice {}

pub async fn notice() -> impl IntoHandlerResponse {
  Unsigned(CallResponse::new_custom(
    1,
    Box::new(Notice {
      text_japanese: None,
      text_english: None,
      text_korean: None,
      answer_alarm: "fail".to_owned(),
    }),
  ))

  // Unsigned(Notice {
  //   text_japanese: Some("メンテナンスのお知らせ".to_owned()),
  //   text_english: Some("Maintenance Notice".to_owned()),
  //   text_korean: Some("점검 안내".to_owned()),
  //   answer_alarm: "success".to_owned(),
  // })
}
