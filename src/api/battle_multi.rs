use crate::api::home::{MultiBattleInvitation, MultiBattleRoom};
use crate::call::CallCustom;
use crate::handler::{IntoHandlerResponse, Unsigned};
use serde::{Deserialize, Serialize};
use tracing::warn;
use crate::extractor::Params;

// See [Wonder_Api_MultiBattleInvitationListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitationListResponse {
  pub multi_battle_invitation: MultiBattleInvitation,
}

impl CallCustom for MultiBattleInvitationListResponse {}

// TODO: BROKEN - Client starts lagging, displays stub (?) text, and does not respond to selection
pub async fn multi_battle_invitation_list() -> impl IntoHandlerResponse {
  warn!("encountered stub: multi_battle_invitation_list");

  Ok(Unsigned(MultiBattleInvitationListResponse {
    multi_battle_invitation: MultiBattleInvitation {
      event_id: 24011,
      rooms: vec![
        MultiBattleRoom {
          room_no: 1,
          quest_id: 500101,
          user_icon: 1083110,
          user_name: "Megumin".to_string(),
        },
      ],
    },
  }))
}

// See [Wonder_Api_MultiBattleRoomInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleRoomInfoResponse {
  pub quest_id: i32,
  #[serde(with = "crate::string_as_base64")]
  pub name: String,
  pub icon: i64,
  #[serde(with = "crate::bool_as_int")]
  pub is_lock: bool,
}

impl CallCustom for MultiBattleRoomInfoResponse {}

#[derive(Debug, Deserialize)]
pub struct MultiBattleRoomInfoRequest {
  pub room_no: i32,
}

// TODO: BROKEN - Client does not show anything
pub async fn multi_battle_room_info(
  Params(params): Params<MultiBattleRoomInfoRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: multi_battle_room_info");

  Ok(Unsigned(MultiBattleRoomInfoResponse {
    quest_id: 0,
    name: "Megumin".to_string(),
    icon: 1083110,
    is_lock: false,
  }))
}
