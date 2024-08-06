use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct Notice {
  #[serde(rename = "answerAlarm")]
  pub answer_alarm: String,
}

impl CallCustom for Notice {}
