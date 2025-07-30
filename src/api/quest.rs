use serde_json::json;

use crate::api::{ApiRequest, NotificationData};
use crate::call::{CallCustom, CallResponse};

pub async fn quest_main_part_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(json!({
      "quests": [
        {
          "quest_part_id": 1,
          "status": 0
        }
      ]
    }))),
    false,
  ))
}

pub async fn quest_main_stage_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(json!({
      "quests": [
        {
          "quest_stage_id": 1,
          "status": 0,
          "task1": 0,
          "task2": 0,
          "task3": 0,
          "challenge_count": 0,
          "difficulty": 1
        }
      ]
    }))),
    false,
  ))
}

pub async fn quest_main_area_list(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "normal_area_list": [
      {
        "quest_area_master_id": 1,
        "status": 0
      }
    ],
    "hard_area_list": [],
    "expert_area_list": [],
  })));
  response.add_notifications(vec![NotificationData::new(1, 7, 20, 1, "".to_owned(), "".to_owned())]);

  Ok((response, false))
}
