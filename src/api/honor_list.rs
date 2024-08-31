use jwt_simple::prelude::Serialize;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

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

pub async fn route(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(HonorList {
      honor_list: vec![
        HonorItem::new(60000000, false, false),
        HonorItem::new(62010250, true, false),
      ],
    })),
    true,
  ))
}
