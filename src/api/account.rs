use std::sync::Arc;

use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use tracing::info;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::session::Session;
use crate::AppState;

pub async fn set_name(
  state: Arc<AppState>,
  request: ApiRequest,
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let session = session.as_ref().ok_or_else(|| anyhow::anyhow!("session is not set"))?;

  let username = &request.body["name"];
  let username = BASE64_STANDARD_NO_PAD
    .decode(username)
    .context("failed to decode username from base64")?;
  let username = String::from_utf8(username).context("username is not valid UTF-8")?;

  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      update users
      set username = $2
      where users.id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  client
    .execute(&statement, &[&session.user_id, &username])
    .await
    .context("failed to execute query")?;
  info!(?session.user_id, ?username, "username updated");

  Ok((CallResponse::new_success(Box::new(())), true))
}
