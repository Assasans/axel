use std::sync::Arc;

use jwt_simple::prelude::Serialize;
use rand::random;
use tracing::info;

use crate::api::{ApiRequest, RemoteData};
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
  response.add_remote_data(vec![
    RemoteData::new(3, 0, 0, 0, 0, 0, "-".to_owned()),
    RemoteData::new(4, 1, 0, 80000, 0, 0, "-".to_owned()),
    RemoteData::new(4, 2, 0, 6000, 0, 0, "-".to_owned()),
    RemoteData::new(4, 3, 0, 3000, 0, 0, "-".to_owned()),
    RemoteData::new(4, 9, 0, 10, 0, 0, "-".to_owned()),
    RemoteData::new(4, 10, 0, 0, 0, 0, "-".to_owned()),
    RemoteData::new(4, 23, 0, 1, 0, 0, "-".to_owned()),
    RemoteData::new(4, 28, 230731, 0, 0, 0, "-".to_owned()),
    RemoteData::new(4, 34, 2, 3, 0, 0, "-".to_owned()),
    RemoteData::new(4, 34, 1, 0, 0, 0, "-".to_owned()),
    RemoteData::new(4, 40, 0, 1, 0, 0, "-".to_owned()),
  ]);

  Ok((response, true))
}
