use std::sync::Arc;

use jwt_simple::prelude::Serialize;
use serde::Deserialize;
use tracing::warn;

use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed, Unsigned};
use crate::user::session::Session;
use crate::user::uuid::UserUuid;

#[derive(Debug, Serialize)]
pub struct IdConfirm {
  pub name: String,
  pub lv: i32,
  pub user_no: String,
}

impl CallCustom for IdConfirm {}

#[derive(Debug, Deserialize)]
pub struct IdConfirmRequest {
  pub take_over_id: String,
  pub password: String,
}

pub async fn id_confirm(Params(params): Params<IdConfirmRequest>) -> impl IntoHandlerResponse {
  warn!(?params.take_over_id, ?params.password, "encountered stub: id_confirm");

  Unsigned(IdConfirm {
    name: "Mock User".to_string(),
    lv: 333,
    user_no: "-1".to_string(),
  })
}

#[derive(Debug, Deserialize)]
pub struct PrepareSetMigrationRequest {
  pub uuid: String,
}

#[derive(Debug, Serialize)]
pub struct PrepareSetMigration {
  pub user_key: String,
}

impl CallCustom for PrepareSetMigration {}

pub async fn prepare_set_migration(Params(params): Params<PrepareSetMigrationRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: prepare_set_migration");

  Unsigned(PrepareSetMigration {
    user_key: "ffffffffffffffffffffffffffffffee".to_string(),
  })
}

// See [Wonder_Api_NewidcheckResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct NewIdCheck {
  pub check: i32,
}

impl CallCustom for NewIdCheck {}

pub async fn new_id_check() -> impl IntoHandlerResponse {
  Unsigned(NewIdCheck { check: 0 })
}

// See [Wonder_Api_NewidResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct NewId {
  pub take_over_id: String,
}

impl CallCustom for NewId {}

#[derive(Debug, Deserialize)]
pub struct NewIdRequest {
  pub newpassword: String,
}

pub async fn new_id(Params(request): Params<NewIdRequest>) -> impl IntoHandlerResponse {
  warn!(?request.newpassword, "encountered stub: new_id");

  Unsigned(NewId {
    take_over_id: "MTF00LTL".to_owned(),
  })
}

// See [Wonder_Api_IdloginResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct IdLogin {
  pub user_key: String,
  pub rule_ver: String,
  pub capture: String,
  pub user_no: String,
}

impl CallCustom for IdLogin {}

#[derive(Debug, Deserialize)]
pub struct IdLoginRequest {
  pub take_over_id: String,
  pub password: String,
  pub uuid: String,
}

pub async fn id_login(Params(params): Params<IdLoginRequest>) -> impl IntoHandlerResponse {
  let uuid = params.uuid.parse::<UserUuid>().unwrap();
  warn!(?params.take_over_id, ?params.password, ?uuid, "encountered stub: id_login");

  // TODO: This should reassociate UUID with new account from take_over_id

  Ok(Unsigned(IdLogin {
    user_key: "ffffffffffffffffffffffffffffffee".to_string(),
    // From "system" master
    rule_ver: "3".to_string(),
    capture: "".to_string(),
    user_no: "-1".to_string(),
  }))
}
