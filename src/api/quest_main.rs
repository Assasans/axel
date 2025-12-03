//! Hierarchy is Part (1) -> Area (Chapter 1) -> Stage (Chapter 1-1)

use serde_json::{json, Value};

use crate::api::master_all::get_masters;
use crate::api::{ApiRequest, NotificationData};
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};

pub async fn quest_main_part_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let parts: Vec<Value> = serde_json::from_str(&masters["main_quest_part"].master_decompressed).unwrap();
  let parts = parts
    .iter()
    .map(|part| {
      json!({
        "quest_part_id": part.get("part").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        "status": 0
      })
    })
    .collect::<Vec<Value>>();

  Ok(Unsigned(CallResponse::new_success(Box::new(json!({
    "quests": parts
  })))))
}

pub async fn quest_main_area_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["mainquest_area"].master_decompressed).unwrap();
  let areas = areas
    .iter()
    .map(|stage| {
      json!({
        "quest_area_master_id": stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        "status": 0
      })
    })
    .collect::<Vec<Value>>();

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(json!({
    "normal_area_list": areas,
    "hard_area_list": [],
    "expert_area_list": [],
  })));
  response.add_notifications(vec![NotificationData::new(1, 7, 20, 1, "".to_owned(), "".to_owned())]);

  Ok(Unsigned(response))
}

pub async fn quest_main_stage_list(request: ApiRequest) -> impl IntoHandlerResponse {
  let area_id: i32 = request.body["area_id"].parse().unwrap();

  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["mainquest_stage"].master_decompressed).unwrap();
  let stages = stages
    .iter()
    .filter(|stage| {
      stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == area_id
        && stage.get("mode").unwrap().as_str().unwrap().parse::<i32>().unwrap() == 1
    })
    .map(|stage| {
      json!({
        "quest_stage_id": stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        "status": 0,
        "task1": 0,
        "task2": 0,
        "task3": 0,
        "challenge_count": 0,
        "difficulty": 1,
      })
    })
    .collect::<Vec<Value>>();

  Ok(Unsigned(CallResponse::new_success(Box::new(json!({
    "quests": stages,
  })))))
}
