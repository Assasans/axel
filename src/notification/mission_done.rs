use crate::api::NotificationData;
use crate::notification::IntoNotificationData;

/// Shows a notification like "Clear Main Quest Part 1, Chapter 1-1 on NORMAL."
pub struct MissionDone {
  pub mission_id: i32,
}

impl MissionDone {
  pub fn new(mission_id: i32) -> Self {
    MissionDone { mission_id }
  }
}

impl IntoNotificationData for MissionDone {
  fn into_notification_data(self) -> NotificationData {
    NotificationData::new(1, 1, 0, self.mission_id, "".to_string(), "".to_string())
  }
}
