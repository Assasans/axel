//! Reference: https://youtu.be/b2S0_Q12axI and https://youtu.be/MQ9VOLhVRbE

use crate::api::master_all::get_master_manager;
use crate::api::{RemoteData, RemoteDataCommand, RemoteDataItemType};
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};
use serde::Serialize;

pub async fn assist_make_notice() -> impl IntoHandlerResponse {
  let assists = get_master_manager().get_master("assist_details");

  let mut response = CallResponse::new_success_empty();
  // Same as with equipment, item-id is assist_details, i.e. assist + level combined.
  // TODO: This sends same assists every time
  response.add_remote_data(
    assists
      .iter()
      .map(|assist| {
        let id = assist["id"].as_str().unwrap().parse::<i64>().unwrap();
        RemoteData {
          cmd: RemoteDataCommand::UserParamAdd as i32,
          uid: None,
          item_type: RemoteDataItemType::Assist.into(),
          item_id: id,
          item_num: 1,
          uniqid: id as i32,
          lv: 0,
          tag: String::from(""),
          member_parameter: None,
          character_parameter: None,
          is_trial: None,
        }
      })
      .collect(),
  );

  Ok(Unsigned(response))
}

// See [Wonder_Api_AssistMakeNoticeResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct AssistMakeList {
  pub assist_detail_id_list: Vec<i64>,
}

impl CallCustom for AssistMakeList {}

pub async fn assist_make_list() -> impl IntoHandlerResponse {
  Ok(Unsigned(AssistMakeList {
    assist_detail_id_list: vec![],
  }))
}
