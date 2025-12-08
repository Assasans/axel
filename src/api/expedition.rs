use serde::Serialize;
use serde_json::Value;

use crate::api::master_all::get_masters;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_ExpeditiontopResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionTop {
  #[serde(rename = "expeditioninfo")]
  pub expeditions: Vec<ExpeditionInfo>,
  #[serde(rename = "bonuspack")]
  pub bonus_pack: i32,
}

impl CallCustom for ExpeditionTop {}

// See [Wonder_Api_ExpeditiontopExpeditioninfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExpeditionInfo {
  pub expedition_id: i32,
  pub character_id: i64,
  #[serde(rename = "starttime")]
  pub start_time: String,
}

// TODO: Can't click to add character for some reason
pub async fn expedition_top() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let expeditions: Vec<Value> = serde_json::from_str(&masters["expedition"].master_decompressed).unwrap();

  // See [Wonder_Api_ExpeditionTopResponseDto_Fields]
  Ok(Unsigned(ExpeditionTop {
    expeditions: expeditions
      .iter()
      .map(|expedition| ExpeditionInfo {
        expedition_id: expedition["expedition_id"].as_str().unwrap().parse().unwrap(),
        character_id: 0,
        start_time: "".to_string(),
      })
      .collect(),
    bonus_pack: 0,
  }))
}
