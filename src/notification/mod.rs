// Related client functions:
// [Wonder.UI.Mypage.MyPageScreen$$UpdateBadgeAll]
// [Wonder.UI.Menu.MenuTopPanel._ShowContent_d__35$$MoveNext]

mod friend_greeting;
mod mission_done;

pub use friend_greeting::*;
pub use mission_done::*;

use crate::api::NotificationData;

pub trait IntoNotificationData {
  fn into_notification_data(self) -> NotificationData;
}
