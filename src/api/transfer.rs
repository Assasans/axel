use anyhow::Context;
use jwt_simple::prelude::Serialize;
use tracing::info;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::user::uuid::UserUuid;

#[derive(Debug, Serialize)]
pub struct IdConfirm {
  pub name: String,
  pub lv: i32,
  pub user_no: String,
}

impl CallCustom for IdConfirm {}

pub async fn id_confirm(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let take_over_id = &request.body["take_over_id"];
  let password = &request.body["password"];

  Ok((
    CallResponse::new_success(Box::new(IdConfirm {
      name: "Mock User".to_string(),
      lv: 333,
      user_no: "-1".to_string(),
    })),
    false,
  ))
}

#[derive(Debug, Serialize)]
pub struct PrepareSetMigration {
  pub user_key: String,
}

impl CallCustom for PrepareSetMigration {}

pub async fn prepare_set_migration(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(PrepareSetMigration {
      user_key: "ffffffffffffffffffffffffffffffee".to_string(),
    })),
    false,
  ))
}

#[derive(Debug, Serialize)]
pub struct NewIdCheck {
  pub check: i32,
}

impl CallCustom for NewIdCheck {}

pub async fn new_id_check(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((CallResponse::new_success(Box::new(NewIdCheck { check: 0 })), false))
}

#[derive(Debug, Serialize)]
pub struct NewId {
  pub take_over_id: String,
}

impl CallCustom for NewId {}

pub async fn new_id(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let newpassword = &request.body["newpassword"];

  Ok((
    CallResponse::new_success(Box::new(NewId {
      take_over_id: "MTF00LTR".to_owned(),
    })),
    false,
  ))
}

#[derive(Debug, Serialize)]
pub struct IdLogin {
  pub user_key: String,
  pub rule_ver: String,
  pub capture: String,
  pub user_no: String,
}

impl CallCustom for IdLogin {}

pub async fn id_login(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let take_over_id = &request.body["take_over_id"];
  let password = &request.body["password"];

  let uuid = request.body.get("uuid").context("no 'uuid' passed")?;
  let uuid = uuid.parse::<UserUuid>().unwrap();
  info!("{:?}", uuid);

  // TODO: This should reassociate UUID with new account from take_over_id

  Ok((
    CallResponse::new_success(Box::new(IdLogin {
      user_key: "ffffffffffffffffffffffffffffffee".to_string(),
      // From "system" master
      rule_ver: "3".to_string(),
      capture: "".to_string(),
      user_no: "-1".to_string(),
    })),
    false,
  ))
}
