use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, trace, warn};

use crate::AppState;
use crate::api::RemoteDataItemType;
use crate::api::master_all::{get_master_manager, get_masters};
use crate::blob::{DeleteMember, IntoRemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::item::UpdateItemCountBy;
use crate::member::MemberPrototype;
use crate::user::session::Session;

// See [Wonder_Api_ExchangelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExchangeList {
  pub exchange_master_id: i32,
  pub items: Vec<ExchangeItem>,
}

impl CallCustom for ExchangeList {}

// See [Wonder_Api_ExchangelistItemsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ExchangeItem {
  pub exchange_reward_master_id: i32,
  pub limit: i32,
  pub exchange_num: i32,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeListRequest {
  pub exchange_master_id: i32,
}

pub async fn exchange_list(
  session: Arc<Session>,
  Params(params): Params<ExchangeListRequest>,
) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let items: Vec<Value> = serde_json::from_str(&masters["exchange_item"].master_decompressed).unwrap();
  let items = items
    .iter()
    .filter_map(|item| {
      if item
        .get("exchange_id")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<i32>()
        .unwrap()
        == params.exchange_master_id
      {
        Some(ExchangeItem {
          exchange_reward_master_id: item.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
          limit: 1000,
          exchange_num: 500,
        })
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  Ok(Signed(
    ExchangeList {
      exchange_master_id: params.exchange_master_id,
      items,
    },
    session,
  ))
}

#[derive(Debug, Deserialize)]
pub struct LeaveMembersRequest {
  #[serde(deserialize_with = "crate::serde_compat::comma_separated_i64")]
  pub ids: Vec<i64>,
}

pub async fn leave_members(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<LeaveMembersRequest>,
) -> impl IntoHandlerResponse {
  let medal_rate = get_master_manager()
    .get_master("member_medal_rate")
    .into_iter()
    .map(|m| {
      let rarity = m["rare"].as_str().unwrap().parse::<i32>().unwrap();
      let medals = m["medal_rate"].as_str().unwrap().parse::<i32>().unwrap();
      (rarity, medals)
    })
    .collect::<HashMap<_, _>>();

  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;

  #[rustfmt::skip]
  let statement = transaction.prepare(/* language=postgresql */ r#"
    delete from user_members_reserve
    where user_id = $1
      and id = any($2)
    returning id, member_id
  "#).await.context("failed to prepare statement")?;
  let rows = transaction
    .query(&statement, &[&session.user_id, &params.ids])
    .await
    .context("failed to execute statement")?;

  // See [Wonder_Api_LeavemenbersResponseDto_Fields]
  let mut response = CallResponse::new_success_empty();

  let mut total_medal_count = 0;
  for row in rows {
    let id: i64 = row.get(0);
    let member_id: i64 = row.get(1);

    let prototype = MemberPrototype::load_from_id(member_id);
    let medal_count = *medal_rate
      .get(&prototype.rarity)
      .context(format!("no medal rate found for member rarity {}", prototype.rarity))?;
    trace!(
      ?id,
      ?member_id,
      ?medal_count,
      "calculated medals for releasing reserve member"
    );

    total_medal_count += medal_count;
    response.remote.extend(DeleteMember::new(id, member_id).into_remote_data());
  }

  const ADVENTURER_MEDAL_ID: i64 = 1001;
  let update = UpdateItemCountBy::new(&transaction).await?;
  let item = update
    .run(
      session.user_id,
      (RemoteDataItemType::ExchangeMedal, ADVENTURER_MEDAL_ID),
      total_medal_count,
    )
    .await
    .context("failed to execute query")?;
  info!(members = ?params.ids, ?item, "granted medals for releasing reserve members");
  response.remote.extend(item.into_remote_data());

  transaction.commit().await.context("failed to commit transaction")?;

  Ok(Signed(response, session))
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRequest {
  pub exchange_reward_master_id: i32,
  pub num: i32,
}

// exchange_reward_master_id=10001
// num=1
pub async fn exchange(session: Arc<Session>, Params(params): Params<ExchangeRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: exchange");

  // See [Wonder_Api_ExchangeResponseDto_Fields]
  Ok(Signed((), session))
}
