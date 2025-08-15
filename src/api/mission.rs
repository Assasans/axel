use serde::Serialize;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::api::master_all::get_masters;
use crate::api::{battle, ApiRequest};
use crate::call::{CallCustom, CallResponse};

#[derive(Debug, Serialize)]
pub struct MissionList {
  #[serde(rename = "list")]
  pub missions: Vec<Mission>,
}

impl CallCustom for MissionList {}

// See [Wonder_Api_MissionListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct Mission {
  pub mission_id: i32,
  #[serde(rename = "type")]
  pub kind: i32,
  pub progress: i32,
  pub received: i32,
  pub newmisson: i32,
  /// Enables the Play button
  pub is_challenge: i32,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MissionKind {
  Beginner = 1,
  Daily = 2,
  Event = 3,
  Normal = 4,
}

pub async fn mission_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let kind = &request.body["type"];

  let masters = get_masters().await;
  let missions: Vec<Value> = serde_json::from_str(&masters["mission"].master_decompressed).unwrap();
  let event_missions: Vec<Value> = serde_json::from_str(&masters["event_mission"].master_decompressed).unwrap();

  let missions = missions
    .into_iter()
    .chain(event_missions.into_iter().take(100))
    .map(|mission| {
      let kind = mission.get("mission_type").unwrap().as_str().unwrap();
      Mission {
        mission_id: mission
          .get("mission_id")
          .unwrap()
          .as_str()
          .unwrap()
          .parse::<i32>()
          .unwrap(),
        kind: match kind {
          "NORMAL" => MissionKind::Normal as i32,
          "DAILY" => MissionKind::Daily as i32,
          "BEGINNER" => MissionKind::Beginner as i32,
          "EVENT" => MissionKind::Event as i32,
          _ => todo!("unknown mission type: {}", kind),
        },
        progress: 0,
        received: 0,
        newmisson: 0,
        is_challenge: 1,
      }
    })
    .collect();

  Ok((CallResponse::new_success(Box::new(MissionList { missions })), true))
}

#[derive(Debug, Serialize)]
pub struct BattleQuestInfo {
  pub ticket: i32,
  pub opflag: i32,
  pub symbol: i32,
  pub boss: Boss,
}

#[derive(Debug, Serialize)]
pub struct Boss {
  pub quest_id: i32,
  pub status: i32,
  pub kill: i32,
}

impl CallCustom for BattleQuestInfo {}

pub async fn battle_quest_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let event_id = request.body["event_id"].parse::<i32>().unwrap();

  Ok((
    CallResponse::new_success(Box::new(BattleQuestInfo {
      ticket: 0,
      opflag: 0,
      symbol: 0,
      boss: Boss {
        quest_id: 0,
        status: 0,
        kill: 0,
      },
    })),
    true,
  ))
}

#[derive(Debug, Serialize)]
pub struct BattleMarathonInfo {
  pub opflag: i32,
  pub boss: Boss,
  pub total_boss_info: TotalBossInfo,
}

// See [Wonder_Api_TotalBossInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct TotalBossInfo {
  pub total_defeat_count: i32,
  pub my_defeat_count: i32,
  pub boss_count_rewards: Vec<BossCountRewards>,
  pub ranking: i32,
  pub in_ranking_period: bool,
}

// See [Wonder_Api_BattlemarathoninfoBossCountRewardsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BossCountRewards {
  pub count_id: i32,
  pub is_received: bool,
}

impl CallCustom for BattleMarathonInfo {}

pub async fn battle_marathon_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let event_id = request.body["event_id"].parse::<i32>().unwrap();

  Ok((
    CallResponse::new_success(Box::new(BattleMarathonInfo {
      opflag: 0,
      boss: Boss {
        quest_id: 0,
        status: 0,
        kill: 0,
      },
      total_boss_info: TotalBossInfo {
        total_defeat_count: 0,
        my_defeat_count: 0,
        boss_count_rewards: vec![],
        ranking: 0,
        in_ranking_period: false,
      },
    })),
    true,
  ))
}

// See [Wonder_Api_MarathonInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonInfo {
  pub opflag: i32,
  /// 0 - not available, 1 - available
  pub boss: i32,
  pub open_scorechallenge: bool,
  pub multi_battle_invitation: MultiBattleInvitationRoom,
  pub total_boss_info: TotalBossInfo,
  pub emergency_boss_info: EmergencyBossInfo,
}

// See [Wonder_Api_MultiBattleInvitationResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitationRoom {
  pub room_id: i32,
  pub room_name: String,
  pub room_type: i32,
  pub room_status: i32,
  pub room_member_count: i32,
  pub room_member_max: i32,
  pub room_member_list: Vec<MultiBattleInvitationMember>,
}

// See [Wonder_Api_MultiBattleInvitationRoomResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MultiBattleInvitationMember {
  pub user_id: i64,
  pub user_icon: i64,
  pub user_name: String,
}

// See [Wonder_Api_EmergencyBossInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct EmergencyBossInfo {
  pub emergency_boss_id: i32,
  pub status: i32,
  pub total_defeat_count: i32,
  pub my_defeat_count: i32,
  pub ranking: i32,
}

impl CallCustom for MarathonInfo {}

pub async fn marathon_info(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let event_id = request.body["event_id"].parse::<i32>().unwrap();
  let display_multi_battle_invitation = request.body["display_multi_battle_invitation"].parse::<i32>().unwrap();

  Ok((
    CallResponse::new_success(Box::new(MarathonInfo {
      opflag: 0,
      boss: 1,
      open_scorechallenge: true,
      multi_battle_invitation: MultiBattleInvitationRoom {
        room_id: 0,
        room_name: "Sosal".to_string(),
        room_type: 0,
        room_status: 0,
        room_member_count: 0,
        room_member_max: 0,
        room_member_list: vec![],
      },
      total_boss_info: TotalBossInfo {
        total_defeat_count: 30,
        my_defeat_count: 4,
        boss_count_rewards: vec![],
        ranking: 3,
        in_ranking_period: false,
      },
      emergency_boss_info: EmergencyBossInfo {
        emergency_boss_id: 0,
        status: 1,
        total_defeat_count: 0,
        my_defeat_count: 0,
        ranking: 0,
      },
    })),
    true,
  ))
}

// See [Wonder_Api_MarathonStageListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonStageList {
  pub quests: Vec<MarathonStageQuest>,
}

// See [Wonder_Api_BattlemarathonstagelistQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonStageQuest {
  pub quest_id: i32,
  /// 0 - locked, 1 - unlocked, 2 - completed, 3 - 100% completed
  pub status: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub hardnum: i32,
}

impl CallCustom for MarathonStageList {}

pub async fn marathon_stage_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let event_id = request.body["event_id"].parse::<i32>().unwrap();

  let masters = get_masters().await;
  let quests: Vec<Value> = serde_json::from_str(&masters["event_marathon_quest_stage"].master_decompressed).unwrap();
  let quests = quests
    .into_iter()
    .filter(|quest| quest.get("event_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == event_id)
    .map(|quest| MarathonStageQuest {
      quest_id: quest.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
      status: 3,
      task1: 0,
      task2: 0,
      task3: 0,
      hardnum: 0,
    })
    .collect::<Vec<_>>();

  Ok((CallResponse::new_success(Box::new(MarathonStageList { quests })), true))
}

// quest_id=514012
// party_no=1
// auto_progression_info={"is_start":false,"stop_setting":0,"incomplete_setting":0}
// event_id=24013
pub async fn marathon_quest_start(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let quest_id = request.body["quest_id"].parse::<i32>().unwrap();
  let party_no = request.body["party_no"].parse::<i32>().unwrap();
  let auto_progression_info: Value = serde_json::from_str(&request.body["auto_progression_info"])?;
  let event_id: Value = serde_json::from_str(&request.body["event_id"])?;

  battle::battle_start(request).await
}

// quest_id=514012
// party_no=1
// auto_progression_info={"is_start":false,"stop_setting":0,"incomplete_setting":0}
// event_id=24013
pub async fn marathon_quest_result(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let quest_id = request.body["quest_id"].parse::<i32>().unwrap();
  let party_no = request.body["party_no"].parse::<i32>().unwrap();
  let event_id: Value = serde_json::from_str(&request.body["event_id"])?;

  battle::battle_result(request).await
}

// See [Wonder_Api_MarathonBossListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonBossList {
  #[serde(rename = "boss")]
  pub bosses: Vec<MarathonBoss>,
}

// See [Wonder_Api_BattlemarathonbosslistBossResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct MarathonBoss {
  pub quest_id: i32,
  pub hp1: i32,
  pub hp2: i32,
  pub hp3: i32,
  /// 0 - locked, 1 - unlocked, 2 - defeated
  pub status: i32,
  /// Wins
  pub kill: i32,
  pub limit_num: i32,
  pub display: i32,
  pub ticket_ratio: i32,
}

impl CallCustom for MarathonBossList {}

pub async fn marathon_boss_list(request: ApiRequest) -> anyhow::Result<(CallResponse<dyn CallCustom>, bool)> {
  let event_id: i32 = request.body["event_id"].parse::<i32>().unwrap();
  let is_multi: i32 = request.body["is_multi"].parse::<i32>().unwrap();

  let masters = get_masters().await;
  let bosses: Vec<Value> =
    serde_json::from_str(&masters["event_marathon_quest_stage_boss_single"].master_decompressed).unwrap();
  let bosses = bosses
    .into_iter()
    .filter(|boss| boss.get("event_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == event_id)
    .map(|boss| MarathonBoss {
      quest_id: boss.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
      hp1: 10,
      hp2: 20,
      hp3: 30,
      status: 2,
      kill: 0,
      limit_num: 0,
      display: 0,
      ticket_ratio: 0,
    })
    .collect::<Vec<_>>();

  Ok((CallResponse::new_success(Box::new(MarathonBossList { bosses })), true))
}
