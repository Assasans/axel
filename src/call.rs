use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use erased_serde::serialize_trait_object;
use jwt_simple::prelude::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{NotificationData, RemoteData};
use crate::session::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct CallMeta {
  pub cs: String,
  #[serde(default)]
  pub uk: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiCallParams {
  #[serde(rename = "u")]
  pub user_id: Option<UserId>,
}

pub trait CallCustom: erased_serde::Serialize {}

impl CallCustom for () {}

impl CallCustom for Value {}

serialize_trait_object!(CallCustom);

#[derive(Debug, Serialize)]
pub struct CallResponse<T: CallCustom + ?Sized> {
  pub status: i32,
  pub time: Option<u32>,
  #[serde(rename = "remotedata")]
  pub remote: Vec<RemoteData>,
  #[serde(rename = "notificationdata")]
  pub notifications: Vec<NotificationData>,
  #[serde(flatten)]
  pub custom: Box<T>,
}

impl<T: CallCustom + ?Sized> CallResponse<T> {
  pub fn new_success(custom: Box<T>) -> Self {
    Self {
      status: 0,
      time: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32),
      remote: Vec::new(),
      notifications: Vec::new(),
      custom,
    }
  }

  pub fn new_custom(status: i32, custom: Box<T>) -> Self {
    Self {
      status,
      time: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32),
      remote: Vec::new(),
      notifications: Vec::new(),
      custom,
    }
  }

  pub fn add_remote_data(&mut self, mut remote_data: Vec<RemoteData>) {
    self.remote.append(&mut remote_data);
  }
}

impl CallResponse<()> {
  pub fn new_success_empty() -> Self {
    Self {
      status: 0,
      time: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32),
      remote: Vec::new(),
      notifications: Vec::new(),
      custom: Box::new(()),
    }
  }
}
