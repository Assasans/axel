use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct StoryReward {
  pub reward: Vec<()>,
}

impl CallCustom for StoryReward {}
