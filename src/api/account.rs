use std::sync::Arc;

use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use tracing::{info, warn};

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;
use crate::AppState;

pub async fn set_name(state: Arc<AppState>, request: ApiRequest, session: Arc<Session>) -> impl IntoHandlerResponse {
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

  Ok(Signed((), session))
}

pub async fn delete_account(session: Arc<Session>) -> impl IntoHandlerResponse {
  warn!("encountered stub: delete_account");

  // See [Wonder_Api_DeleteAccountResponseDto_Fields]
  Ok(Signed((), session))
}
