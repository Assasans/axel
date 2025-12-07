use std::sync::Arc;

use anyhow::Context;
use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use crate::api::master_all::get_masters;
use crate::api::ApiRequest;
use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::AppState;

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
  let masters = get_masters().await;
  let honors: Vec<Value> = serde_json::from_str(&masters["honor"].master_decompressed).unwrap();

  let selected_honor_id = 62010250;

  Ok(Signed(
    HonorList {
      honor_list: honors
        .iter()
        .map(|honor| {
          let honor_id: u32 = honor["id"].as_str().unwrap().parse().unwrap();
          HonorItem::new(honor_id, honor_id == selected_honor_id, false)
        })
        .collect(),
    },
    session,
  ))
}

#[derive(Debug, Deserialize)]
pub struct HonorSetRequest {
  pub honor_id: u32,
}

// honor_id=60000000
pub async fn honor_set(
  state: Arc<AppState>,
  Params(params): Params<HonorSetRequest>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set honor = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &(params.honor_id as i64)])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.honor_id, "honor updated");

  // See [Wonder_Api_HonorSetResponseDto_Fields]
  Ok(Unsigned(()))
}

#[derive(Debug, Deserialize)]
pub struct SetIconRequest {
  pub illustration_id: u32,
}

// illustration_id=1143127
pub async fn set_icon(
  state: Arc<AppState>,
  Params(params): Params<SetIconRequest>,
  request: ApiRequest,
  session: Arc<Session>,
) -> impl IntoHandlerResponse {
  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set favorite_member = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &(params.illustration_id as i64)])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.illustration_id, "icon updated");

  // See [Wonder_Api_HonorSetResponseDto_Fields]
  Ok(Unsigned(()))
}
