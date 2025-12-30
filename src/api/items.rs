use serde::Serialize;
use serde_json::json;

use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};

// See [Wonder_Api_WeaponlistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct WeaponList {
  pub items: Vec<WeaponListItem>,
}

impl CallCustom for WeaponList {}

// See [Wonder_Api_WeaponlistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct WeaponListItem {
  pub id: i64,
  pub weapon_id: i64,
  pub islock: i32,
  pub trial: bool,
}

pub async fn weapon_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(WeaponList {
    items: vec![
      WeaponListItem {
        id: 12211,
        weapon_id: 12211,
        islock: 0,
        trial: false,
      },
    ],
  }))
}

pub async fn accessory_list(_request: ApiRequest) -> impl IntoHandlerResponse {
  Ok(Unsigned(json!({
    "items": [],
  })))
}
