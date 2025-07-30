use std::sync::Arc;

use jwt_simple::prelude::Serialize;
use rand::random;
use tracing::info;

use crate::api::{ApiRequest, NotificationData, RemoteData};
use crate::call::{CallCustom, CallResponse};
use crate::session::{Session, UserId};

#[derive(Debug, Serialize)]
pub struct Login {
  pub user_no: String,
  pub user_key: String,
  pub user_name: String,
  pub tutorial: u8,
  pub created_at: String,
}

impl CallCustom for Login {}

pub async fn route(
  request: ApiRequest,
  session: &mut Option<Arc<Session>>,
) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  info!(user_id = ?request.params.user_id, "create session");
  *session = Some(if let Some(user_id) = request.params.user_id {
    // Existing user
    // TODO: Load from database...
    Arc::new(Session::new(user_id))
  } else {
    // New user
    let user_id = UserId::new(random::<u32>() as u64);
    Arc::new(Session::new(user_id))
  });
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
    user_key: hex::encode(session.user_key.lock().unwrap().expect("no user key")),
    user_name: "".to_string(),
    tutorial: 99,
    created_at: "".to_string(),
  }));

  // Keep it as a blob for now, it is very large...
  let remote_data = include_str!("../login-remotedata.json");
  let remote_data: Vec<RemoteData> = serde_json::from_str(remote_data).unwrap();
  info!(?remote_data, "adding remote data");
  response.add_remote_data(remote_data);

  response.add_notifications(vec![
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
    NotificationData::new(1, 7, 3, 2, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 13, 7, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 11, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 12, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 24, 0, "".to_string(), "".to_string()),
    NotificationData::new(1, 7, 14, 1, "".to_string(), "".to_string()),
  ]);

  Ok((response, true))
}
