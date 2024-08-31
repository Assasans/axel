use jwt_simple::prelude::Serialize;

pub mod gacha;
pub mod home;
pub mod honor_list;
pub mod idlink_confirm_google;
pub mod interaction;
pub mod login;
pub mod login_bonus;
pub mod maintenance_check;
pub mod master_all;
pub mod master_list;
pub mod notice;
pub mod party_info;
pub mod profile;
pub mod story_reward;

#[derive(Debug, Serialize)]
pub struct RemoteData {
  pub cmd: u8,
  pub item_type: u8,
  pub item_id: u32,
  pub item_num: u32,
  pub uniqid: u32,
  pub lv: u32,
  pub tag: String,
}

impl RemoteData {
  pub fn new(cmd: u8, item_type: u8, item_id: u32, item_num: u32, uniqid: u32, lv: u32, tag: String) -> Self {
    Self {
      cmd,
      item_type,
      item_id,
      item_num,
      uniqid,
      lv,
      tag,
    }
  }
}

#[derive(Debug, Serialize)]
pub struct NotificationData {
  pub cmd: u8,
  #[serde(rename = "type")]
  pub kind: u32,
  pub key: u32,
  pub value: u32,
  pub msgkey: String,
  pub tag: String,
}

impl NotificationData {
  pub fn new(cmd: u8, kind: u32, key: u32, value: u32, msgkey: String, tag: String) -> Self {
    Self {
      cmd,
      kind,
      key,
      value,
      msgkey,
      tag,
    }
  }
}
