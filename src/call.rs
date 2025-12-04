use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use erased_serde::serialize_trait_object;
use jwt_simple::prelude::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use crate::api::{NotificationData, RemoteData};
use crate::user::id::UserId;

#[derive(Debug, Serialize, Deserialize)]
pub struct CallMeta {
  pub cs: String,
  #[serde(default)]
  pub uk: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiCallParams {
  #[serde(rename = "u")]
  pub user_id: Option<UserId>,
}

pub trait CallCustom: erased_serde::Serialize + Send {}

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

// See [Wonder.UI.Title.TitleScene._OnTitleRequestError_d__97$$MoveNext]
pub const STATUS_ERROR: i32 = -100;
pub const STATUS_MAINTENANCE: i32 = -102;

pub const STATUS_QUARTZ_NOT_ENOUGH: i32 = -110;

pub const STATUS_LOGIN_TRANSFER_WRONG_KEY: i32 = -104;
/// Logs the user out
pub const STATUS_LOGIN_TRANSFER_SAME: i32 = -169;
/// Account was transferred, no longer available on this device
pub const STATUS_LOGIN_TRANSFER_DONE: i32 = -1013;
/// Cannot transfer account,
pub const STATUS_LOGIN_TRANSFER_LOCAL_ACCOUNT_PRESENT: i32 = -178;

pub const STATUS_ACCOUNT_RESTRICTED: i32 = -903;

pub const STATUS_NEW_DATA_AVAILABLE: i32 = -901;
pub const STATUS_APP_UPDATE_REQUIRED: i32 = -900;

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

  pub fn add_notifications(&mut self, mut notifications: Vec<NotificationData>) {
    self.notifications.append(&mut notifications);
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
