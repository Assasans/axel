use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct StoryReward {
  pub reward: Vec<()>,
}

impl CallCustom for StoryReward {}

pub async fn route(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(StoryReward { reward: vec![] })),
    true,
  ))
}
