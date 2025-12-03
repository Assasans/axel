use std::sync::Arc;

use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

#[derive(Debug, Serialize)]
pub struct HonorList {
  pub honor_list: Vec<HonorItem>,
}

impl CallCustom for HonorList {}

#[derive(Debug, Serialize)]
pub struct HonorItem {
  pub honor_id: u32,
  pub is_selected: bool,
  pub is_new: bool,
}

impl HonorItem {
  pub fn new(honor_id: u32, is_selected: bool, is_new: bool) -> Self {
    Self {
      honor_id,
      is_selected,
      is_new,
    }
  }
}

pub async fn honor_list(_request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  Ok(Signed(
    CallResponse::new_success(Box::new(HonorList {
      honor_list: vec![
        HonorItem::new(60000000, false, false),
        HonorItem::new(62010250, true, false),
      ],
    })),
    session,
  ))
}
