use serde::Serialize;

use crate::api::party_info::Member;
use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};

// See [Wonder_Api_DungeonStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonStatusResponse {
  pub area_id: i32,
  pub status: i32,
}

impl CallCustom for DungeonStatusResponse {}

pub async fn dungeon_status(_request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  Ok((
    CallResponse::new_success(Box::new(DungeonStatusResponse { area_id: 11, status: 0 })),
    false,
  ))
}

// Not sure if this is correct type, game gets soft lock up
// See [Wonder_Api_DungeonAreaTopResponseDto_uWuMsMa_Fields]
#[derive(Debug, Serialize)]
pub struct DungeonAreaTopResponse {
  pub member: Member,
}

impl CallCustom for DungeonAreaTopResponse {}

pub async fn dungeon_area_top(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let area_id: i32 = request.body["area_id"].parse().unwrap();

  Ok((
    CallResponse::new_success(Box::new(DungeonAreaTopResponse {
      member: Member {
        id: 1011100,
        lv: 1,
        exp: 0,
        member_id: 1011100,
        ac_skill_lv_a: 1,
        ac_skill_val_a: 93,
        ac_skill_lv_b: 1,
        ac_skill_val_b: 128,
        ac_skill_lv_c: 1,
        ac_skill_val_c: 122,
        hp: 239,
        attack: 25,
        magicattack: 32,
        defense: 24,
        magicdefence: 24,
        agility: 71,
        dexterity: 74,
        luck: 72,
        limit_break: 0,
        character_id: 101,
        waiting_room: 0,
        ex_flg: 0,
        is_undead: 0,
      },
    })),
    false,
  ))
}
