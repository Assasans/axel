use std::sync::Arc;

use anyhow::Context;
use serde::Deserialize;
use tracing::info;

use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::user::session::Session;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TutorialRequest {
  #[serde(rename = "type")]
  pub kind: String,
  pub progress: i32,
}

pub async fn tutorial(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<TutorialRequest>,
) -> impl IntoHandlerResponse {
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
    .execute(&statement, &[&session.user_id, &params.progress])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?params.kind, ?params.progress, "tutorial progress updated");

  Ok(Unsigned(()))
}
