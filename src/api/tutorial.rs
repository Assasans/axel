use std::sync::Arc;

use anyhow::Context;
use tracing::info;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;
use crate::AppState;

pub async fn tutorial(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
  let kind = &request.body["type"];
  let progress: i32 = request.body["progress"].parse().unwrap();

  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set tutorial_progress = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &progress])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?kind, ?progress, "tutorial progress updated");

  Ok(Unsigned(CallResponse::new_success(Box::new(()))))
}
