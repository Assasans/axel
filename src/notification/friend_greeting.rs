use crate::api::NotificationData;
use crate::notification::IntoNotificationData;

// See [Wonder.UI.Mypage.MyPageScreen$$UpdateBadgeAll]
/// Displays a popup notification near the Friend button when home screen is opened.
/// If user will click on the Friend button, it will open the Greeting Log instead of the Friend List.
#[derive(Debug, Clone)]
pub struct FriendGreetingNotify {
  pub message: String,
}

impl FriendGreetingNotify {
  // See [Wonder.UI.Mypage.MyPageScreen$$UpdateBadgeAll], [_friendButton]
  const KIND: i32 = 27;

  pub fn new(message: String) -> Self {
    FriendGreetingNotify { message }
  }
}

impl IntoNotificationData for FriendGreetingNotify {
  fn into_notification_data(self) -> NotificationData {
    NotificationData::new(1, 7, FriendGreetingNotify::KIND, 1, self.message, "".to_string())
  }
}
