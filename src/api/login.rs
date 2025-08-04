use std::sync::Arc;

use anyhow::Context;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Utc};
use jwt_simple::prelude::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::info;

use crate::api::{ApiRequest, NotificationData, RemoteData};
use crate::build_info::BUILD_INFO;
use crate::call::{CallCustom, CallResponse};
use crate::notification::FriendGreetingNotify;
use crate::session::{Session, UserId};
use crate::AppState;

// See [Wonder_Api_LoginInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Login {
  pub user_no: String,
  pub user_key: String,
  pub user_name: String,
  pub tutorial: i32,
  pub created_at: String,
}

impl CallCustom for Login {}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TutorialState {
  Afterlife = 0,
  Battle = 1,
  Gacha = 2,
  /// Client sets it only after username was already set, so it is never actually used.
  SetUsername = 4,
  Completed = 99,
}

pub async fn login(
  state: Arc<AppState>,
  request: ApiRequest,
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let uuid = request.body.get("uuid").context("no device uuid passed")?;

  let client = state.pool.get().await.context("failed to get database connection")?;
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select users.id, users.username, users.created_at, users.tutorial_progress
      from users
        inner join user_devices device on device.user_id = users.id
      where token = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let rows = client
    .query(&statement, &[&uuid])
    .await
    .context("failed to execute query")?;
  info!(?rows, "login query executed");

  let (id, username, created_at, tutorial_progress) = if rows.is_empty() {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        insert into users default values
        returning id, username, created_at, tutorial_progress
      "#)
      .await
      .context("failed to prepare insert statement")?;
    let rows = client
      .query(&statement, &[])
      .await
      .context("failed to execute insert query")?;
    let row = rows.first().context("no rows returned from insert query")?;

    let id: UserId = row.get(0);
    let username: Option<String> = row.get(1);
    let created_at: DateTime<Utc> = row.get(2);
    let tutorial_progress: i32 = row.get(3);

    #[rustfmt::skip]
    let statement = client
      .prepare(
        /* language=postgresql */ r#"
        insert into user_devices (user_id, token, last_used)
        values ($1, $2, now())
      "#)
      .await
      .context("failed to prepare device insert statement")?;
    client
      .execute(&statement, &[&id, &uuid])
      .await
      .context("failed to execute device insert query")?;

    info!("created new user {}", id);
    (id, username, created_at, tutorial_progress)
  } else {
    if rows.len() > 1 {
      todo!("multiple rows returned from login query (unique token constraint violated?)");
    }
    let row = rows.first().context("no rows returned from login query")?;

    let id: UserId = row.get(0);
    let username: Option<String> = row.get(1);
    let created_at: DateTime<Utc> = row.get(2);
    let tutorial_progress: i32 = row.get(3);

    info!(?username, "user {} logged in", id);
    (id, username, created_at, tutorial_progress)
  };

  *session = Some(Arc::new(Session::new(id, Some(uuid.to_owned()))));
  let session = session.as_ref().unwrap();

  session.rotate_user_key();
  request
    .state
    .sessions
    .lock()
    .unwrap()
    .insert(session.user_id, session.clone());

  let mut response: CallResponse<dyn CallCustom> = CallResponse::new_success(Box::new(Login {
    user_no: session.user_id.to_string(),
    user_key: const_hex::encode(session.user_key.lock().unwrap().expect("no user key")),
    // "---" matches the behavior of the original server
    user_name: BASE64_STANDARD_NO_PAD.encode(username.as_deref().unwrap_or("---").as_bytes()),
    tutorial: tutorial_progress,
    created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
  }));

  // Keep it as a blob for now, it is very large...
  let remote_data = include_str!("../login-remotedata.json");
  let remote_data: Vec<RemoteData> = serde_json::from_str(remote_data).unwrap();
  response.add_remote_data(remote_data);

  response.add_notifications(vec![
    // Jobs
    NotificationData::new(1, 7, 6, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 8, 0, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 6, 1, 30030001, "".to_string(), "".to_string()),
    NotificationData::new(1, 10, 230731, 52307325, "".to_string(), "".to_string()),
    NotificationData::new(1, 10, 230831, 52308305, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 200012, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410535, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410536, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410553, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410123, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410436, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410565, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410433, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410564, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410554, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410554, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410554, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410554, "".to_string(), "".to_string()),
    NotificationData::new(1, 12, 19, 410554, "".to_string(), "".to_string()),
    NotificationData::new(1, 14, 21, 11003, "".to_string(), "".to_string()),
    NotificationData::new(1, 14, 21, 31015, "".to_string(), "".to_string()),
    NotificationData::new(1, 14, 21, 31016, "".to_string(), "".to_string()),
    NotificationData::new(1, 14, 21, 31017, "".to_string(), "".to_string()),
    NotificationData::new(1, 16, 1, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 15, 1, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 23, 3, 1, "".to_string(), "".to_string()),
    NotificationData::new(1, 25, 3, 0, "0".to_string(), "".to_string()),
    NotificationData::new(1, 14, 4, 1722609570, "".to_string(), "".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "100".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "101".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "102".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "103".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "104".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "105".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "106".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "107".to_string()),
    NotificationData::new(1, 19, 5, 0, "".to_string(), "23083".to_string()),
    NotificationData::new(1, 19, 5, 1722620388, "".to_string(), "50081".to_string()),
    NotificationData::new(1, 7, 23, 1, "[\"12209\",\"12206\",\"12204\",\"12203\",\"12205\",\"12207\",\"12208\",\"12210\",\"12211\",\"12212\",\"12320\",\"12900\",\"12100\",\"12200\",\"12300\",\"12310\"]".to_string(), "".to_string()),
    NotificationData::new(1, 7, 401, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 4011, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 4012, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 15, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 16, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 18, 1722864558, "".to_string(), "".to_string()),
    // Missions
    NotificationData::new(1, 7, 3, 2, "".to_string(), "".to_string()),
    // Something related to Missions
    NotificationData::new(1, 7, 13, 7, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 11, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 12, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 24, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
    FriendGreetingNotify::new({
      let hash = BUILD_INFO.git_hash.chars().take(8).collect();
      let revision = if BUILD_INFO.git_dirty {
        format!("{hash}-dirty")
      } else {
        hash
      };

      if BUILD_INFO.profile == "debug" {
        format!("axel/{revision} (development build)")
      } else {
        format!("axel/{revision}")
      }
    }).into(),
  ]);

  Ok((response, true))
}

pub const OP_BADGE_COUNT: i32 = 7;
