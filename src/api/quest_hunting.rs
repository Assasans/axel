//! Hierarchy is Area (Relic Quest) -> Stage (Eris - Beginner)
//! Reference: https://youtu.be/S9fX6sbXRHw (also shows character upgrade and promotion)

use crate::AppState;
use crate::api::battle_multi::{BattleCharacterLove, BattleClearReward, BattleMemberExp};
use crate::api::master_all::{get_master_manager, get_masters};
use crate::api::smith_upgrade::{DungeonAreaMaterialInfoResponseDto, FameQuestMaterialInfoResponseDto};
use crate::api::{RemoteDataItemType, battle, MemberFameStats};
use crate::blob::IntoRemoteData;
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::item::UpdateItemCountBy;
use crate::user::session::Session;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};
use crate::api::party_info::{Party, PartyForm, SpecialSkillInfo};
use crate::member::{Member, MemberActiveSkill, MemberPrototype, MemberStrength};

// See [Wonder_Api_QuesthuntinglistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingListResponse {
  pub limitquests: Vec<HuntingLimitQuest>,
  pub freequests: Vec<HuntingFreeQuest>,
  /// Whether "purchase more attempts" button is enabled
  pub enablepackage: bool,
}

impl CallCustom for QuestHuntingListResponse {}

// See [Wonder_Api_QuesthuntinglistLimitquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingLimitQuest {
  pub area_id: i32,
  pub status: i32,
  /// Attempts left. "Challenges {master.limit_count - limit} / {master.limit_count}"
  pub limit: i32,
}

// See [Wonder_Api_QuesthuntinglistFreequestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingFreeQuest {
  pub area_id: i32,
  pub status: i32,
}

pub async fn quest_hunting_list() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let areas: Vec<Value> = serde_json::from_str(&masters["huntingquest_area"].master_decompressed).unwrap();

  Ok(Unsigned(QuestHuntingListResponse {
    limitquests: areas
      .iter()
      .filter(|area| area.get("type").unwrap().as_str().unwrap() == "LIMITED")
      .map(|area| {
        let area_id = area.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();

        HuntingLimitQuest {
          area_id,
          status: 0,
          limit: 1,
        }
      })
      .collect::<Vec<_>>(),
    freequests: areas
      .iter()
      .filter(|area| area.get("type").unwrap().as_str().unwrap() == "FREE")
      .map(|area| {
        let area_id = area.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap();

        HuntingFreeQuest { area_id, status: 0 }
      })
      .collect::<Vec<_>>(),
    enablepackage: true,
  }))
}

// See [Wonder_Api_QuesthuntingstagelistResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingStageListResponse {
  pub quests: Vec<HuntingStageQuest>,
}

impl CallCustom for QuestHuntingStageListResponse {}

// Wonder.Util.ParameterUtil$$GetAttributeNum
// See [Wonder.Util.ParameterUtil$$GetAttributeNum]
// "all" = 8
// "wind" = 3
// "earth" = 2
// "water" = 1
// "0" = 7
// "thunder" = 4
// "light" = 5
// "cursed" = 6
// "fire" = 0
// "unattributed" = 7

// See [Wonder_Api_QuesthuntingstagelistQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingStageQuest {
  pub stage_id: i32,
  /// 0 - unlocked (new), 1 - unlocked, 2 - completed, 3 - 100% completed
  pub status: i32,
  /// Unknown
  pub newstage: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
}

#[derive(Debug, Deserialize)]
pub struct QuestHuntingStageListRequest {
  pub area_id: i32,
}

pub async fn quest_hunting_stage_list(
  Params(params): Params<QuestHuntingStageListRequest>,
) -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["huntingquest_stage"].master_decompressed).unwrap();

  // TODO: This should probably return remote data or notification data, but I have no dumps for it.
  //  All stages are locked...
  Ok(Unsigned(QuestHuntingStageListResponse {
    quests: stages
      .iter()
      .filter(|stage| stage.get("area_id").unwrap().as_str().unwrap().parse::<i32>().unwrap() == params.area_id)
      .map(|stage| HuntingStageQuest {
        stage_id: stage.get("id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        status: 2,
        newstage: 0,
        task1: 1,
        task2: 1,
        task3: 0,
      })
      .collect::<Vec<_>>(),
  }))
}

// See [Wonder_Api_QuestHuntingLimitStageListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct QuestHuntingLimitStageListResponse {
  pub quests: Vec<HuntingLimitStageQuest>,
  pub bonuspack: i32,
}

impl CallCustom for QuestHuntingLimitStageListResponse {}

// See [Wonder_Api_QuestHuntingLimitStageListQuestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingLimitStageQuest {
  pub stage_id: i32,
  pub challenge_count: i32,
}

pub async fn quest_hunting_limit_stage_list() -> impl IntoHandlerResponse {
  let masters = get_masters().await;
  let stages: Vec<Value> = serde_json::from_str(&masters["huntingquest_stage"].master_decompressed).unwrap();

  Ok(Unsigned(QuestHuntingLimitStageListResponse {
    quests: stages
      .iter()
      .map(|stage| HuntingLimitStageQuest {
        stage_id: stage.get("stage_id").unwrap().as_str().unwrap().parse::<i32>().unwrap(),
        challenge_count: 42,
      })
      .collect::<Vec<_>>(),
    bonuspack: 0,
  }))
}

// body={"quest_id": "416055", "party_no": "1"}
#[derive(Debug, Deserialize)]
pub struct BattleHuntingStartRequest {
  pub quest_id: i32,
  #[serde(rename = "party_no")]
  pub party_id: i32,
}

pub async fn battle_hunting_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleHuntingStartRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_hunting_start");

  battle::make_battle_start(&state, &session, params.party_id).await
}

// See [Wonder_Api_BattlehuntingresultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleHuntingResultResponse {
  pub limit: i32,
  pub exp: i32,
  pub lvup: i32,
  pub money: i32,
  pub storyunlock: Vec<i32>,
  /// Must contain all characters used in the battle
  pub love: Vec<BattleCharacterLove>,
  /// Must contain all members used in the battle
  pub member_exp: Vec<BattleMemberExp>,
  pub mission: Vec<i32>,
  pub reward: Vec<BattleReward>,
  pub clearreward: Vec<BattleClearReward>,
}

// See [Wonder_Api_BattlehuntingresultRewardResponseDto_Fields]
// See [Wonder_Api_ResultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BattleReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  #[serde(with = "crate::bool_as_int")]
  pub is_rare: bool,
}

impl CallCustom for BattleHuntingResultResponse {}

// body={"party_no": "1", "win": "1", "quest_id": "416011", "memcheckcount": "0", "clearquestmission": "[12,13,15]", "wave": "3"}
#[derive(Debug, Deserialize)]
pub struct BattleHuntingResultRequest {
  #[serde(rename = "party_no")]
  pub party_id: i32,
  pub win: i32,
  pub quest_id: i32,
  pub memcheckcount: i32,
  pub clearquestmission: Vec<i32>,
  pub wave: i32,
}

pub async fn battle_hunting_result(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<BattleHuntingResultRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: battle_hunting_result");

  let rewards = get_master_manager()
    .get_master("huntingquest_stage_itemreward")
    .into_iter()
    .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
    .collect::<HashMap<_, _>>();
  let mut rewards = extract_items(&rewards[&params.quest_id]);
  for item in &mut rewards {
    item.item_num *= 20;
  }

  let mut client = state.get_database_client().await?;
  let transaction = client.transaction().await.context("failed to start transaction")?;
  let update = UpdateItemCountBy::new(&transaction).await?;
  let mut update_items = Vec::new();
  for item in &rewards {
    let item = update
      .run(
        session.user_id,
        (RemoteDataItemType::from(item.item_type), item.item_id),
        item.item_num,
      )
      .await
      .context("failed to execute query")?;
    debug!(?item, "granted hunting quest reward");

    update_items.push(item.into_remote_data());
  }
  transaction.commit().await.context("failed to commit transaction")?;

  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      select
        member_id,
        xp,
        promotion_level
      from user_members
      where user_id = $1
    "#)
    .await
    .context("failed to prepare statement")?;
  let members = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")?;

  let statement = client
    .prepare(
      /* language=postgresql */
      r#"
      select
        up.party_id,
        -- Incidentally, client expects party name to be inside each form,
        -- which is exactly how JOIN returns it.
        up.name,
        upf.form_id,
        upf.main_member_id,
        upf.sub1_member_id,
        upf.sub2_member_id,
        upf.weapon_id,
        upf.accessory_id,
        upf.special_skill_id
      from user_parties up
        join user_party_forms upf
          on up.user_id = upf.user_id and up.party_id = upf.party_id
      where up.user_id = $1 and up.party_id = $2
      order by upf.form_id
    "#,
    )
    .await
    .context("failed to prepare statement")?;
  let forms = client
    .query(&statement, &[&session.user_id, &(params.party_id as i64)])
    .await
    .context("failed to execute query")?;
  let forms = forms
    .into_iter()
    .map(|row| {
      let party_name: String = row.get(1);
      let form_id: i64 = row.get(2);
      let main_member_id: i64 = row.get(3);
      let sub1_member_id: i64 = row.get(4);
      let sub2_member_id: i64 = row.get(5);
      let weapon_id: i64 = row.get(6);
      let accessory_id: i64 = row.get(7);
      let special_skill_id: i64 = row.get(8);

      PartyForm {
        id: form_id as i32,
        form_no: form_id as i32,
        party_no: params.party_id,
        main: main_member_id as i32,
        sub1: sub1_member_id as i32,
        sub2: sub2_member_id as i32,
        weapon: weapon_id,
        acc: accessory_id,
        name: party_name,
        strength: 123,
        specialskill: SpecialSkillInfo {
          special_skill_id: special_skill_id as i32,
          trial: false,
        },
        skill_pa_fame: 0,
      }
    })
    .collect::<Vec<_>>()
    .try_into()
    .unwrap();

  let party = Party::new(forms, params.party_id);

  let members = members
    .iter()
    .enumerate()
    .map(|(index, row)| {
      let member_id: i64 = row.get(0);
      let xp: i32 = row.get(1);
      let promotion_level: i32 = row.get(2);
      // let active_skills: Value = row.get(3);
      let prototype = MemberPrototype::load_from_id(member_id);

      let form = party
        .party_forms
        .iter()
        .find(|form| form.main as i64 == member_id)
        .unwrap();
      Member {
        id: prototype.id as i32,
        prototype: &prototype,
        xp,
        promotion_level,
        active_skills: prototype
          .active_skills
          .iter()
          .map(|skill_opt| {
            skill_opt.as_ref().map(|skill| MemberActiveSkill {
              prototype: skill,
              level: 1,
              value: skill.value.max,
            })
          })
          .collect::<Vec<_>>()
          .try_into()
          .unwrap(),
        // active_skills: prototype
        //   .active_skills
        //   .iter()
        //   .enumerate()
        //   .map(|(index, prototype)| {
        //     // TODO: Wrong
        //     let active_skill = active_skills.get(index).unwrap();
        //     // let skill_id = active_skill["id"].as_i64().unwrap();
        //     let level = active_skill["level"].as_i64().unwrap() as i32;
        //     let value = active_skill["value"].as_i64().unwrap() as i32;
        //     Some(MemberActiveSkill {
        //       prototype: &prototype,
        //       level,
        //       value,
        //     })
        //   })
        //   .try_into()
        //   .unwrap(),
        stats: prototype.stats.clone(),
        main_strength: MemberStrength::default(),
        sub_strength: MemberStrength::default(),
        sub_strength_bonus: MemberStrength::default(),
        fame_stats: MemberFameStats::default(),
        skill_pa_fame_list: vec![],
      }
        .to_battle_member(form)
    })
    .collect::<Vec<_>>();

  let characters = party.party_forms.iter().map(|form| {
    vec![
      members.iter().find(|member| member.member_id == form.main as i64).map(|m| m.character_id),
      members.iter().find(|member| member.member_id == form.sub1 as i64).map(|m| m.character_id),
      members.iter().find(|member| member.member_id == form.sub2 as i64).map(|m| m.character_id),
    ]
  }).flatten().flatten().collect::<Vec<_>>();
  let characters = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select
          character_id,
          intimacy
        from user_characters
        where user_id = $1
      "#)
      .await
      .context("failed to prepare statement")?;
    let rows = client
      .query(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")?;
    rows
      .iter()
      .map(|row| {
        let character_id: i64 = row.get(0);
        let intimacy: i32 = row.get(1);
        (character_id, intimacy)
      })
      .filter(|(id, _)| characters.contains(id))
      .collect::<HashMap<_, _>>()
  };

  let mut response = CallResponse::new_success(Box::new(BattleHuntingResultResponse {
    limit: 1,
    exp: 230,
    lvup: 0,
    money: 42000,
    storyunlock: vec![],
    love: characters.iter().map(|(id, intimacy)| BattleCharacterLove {
      character_id: *id,
      love: *intimacy + 10,
    }).collect(),
    member_exp: members
      .iter()
      .map(|member| BattleMemberExp {
        member_id: member.member_id,
        exp: 230,
      })
      .collect(),
    mission: params.clearquestmission,
    reward: rewards
      .iter()
      .map(|item| BattleReward {
        itemtype: item.item_type,
        itemid: item.item_id,
        itemnum: item.item_num,
        is_rare: item.item_rare,
      })
      .collect(),
    clearreward: vec![],
  }));
  response.remote.extend(update_items.into_iter().flatten());
  Ok(Unsigned(response))
}

// See [Wonder_Api_HuntingquestListByItemResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingQuestListByItemResponse {
  pub huntingquests: Vec<HuntingQuest>,
  pub eventboxgachas: Vec<i32>,
  pub exchanges: Vec<i32>,
  pub expedition: i32,
  pub scorechallenge: i32,
  pub scorechallenge_ex: i32,
  pub dungeon: DungeonAreaMaterialInfoResponseDto,
  pub fame_quest: Vec<FameQuestMaterialInfoResponseDto>,
}

// See [Wonder_Api_HuntingquestListByItemHuntingquestsResponseDto_Fields]
// See [Wonder_Api_BlacksmithquestlistHuntingquestsResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct HuntingQuest {
  pub quest_id: i32,
  pub task1: i32,
  pub task2: i32,
  pub task3: i32,
  pub limit: i32,
  pub status: i32,
}

impl CallCustom for HuntingQuestListByItemResponse {}

// body={"item_type": "16", "item_id": "161"}
#[derive(Debug, Deserialize)]
pub struct HuntingQuestListByItemRequest {
  pub item_type: i32,
  pub item_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntingRewardItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
  pub item_rare: bool,
}

pub fn extract_items(value: &Value) -> Vec<HuntingRewardItem> {
  let object = value.as_object().unwrap();

  // Find all indices that appear in keys like "item_type{n}"
  let mut indices: Vec<u32> = object
    .keys()
    .filter_map(|k| k.strip_prefix("item_type"))
    .filter_map(|suffix| suffix.parse::<u32>().ok())
    .collect();

  indices.sort_unstable();
  indices.dedup();

  indices
    .into_iter()
    .filter_map(|i| {
      let item_type = object
        .get(format!("item_type{i}").as_str())?
        .as_str()
        .unwrap()
        .parse::<i32>()
        .ok()?;
      let item_id = object
        .get(format!("item_id{i}").as_str())?
        .as_str()
        .unwrap()
        .parse::<i64>()
        .ok()?;
      let item_num = object
        .get(format!("item_num{i}").as_str())?
        .as_str()
        .unwrap()
        .parse::<i32>()
        .ok()?;
      let item_rare = object
        .get(format!("item_rare{i}").as_str())?
        .as_str()
        .unwrap()
        .parse::<i32>()
        .ok()?
        != 0;

      // Skip empty item slots
      if item_type == 0 || item_id == 0 || item_num == 0 {
        return None;
      }

      Some(HuntingRewardItem {
        item_type,
        item_id,
        item_num,
        item_rare,
      })
    })
    .collect()
}

pub async fn hunting_quest_list_by_item(
  Params(params): Params<HuntingQuestListByItemRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: hunting_quest_list_by_item");

  let stages = get_master_manager().get_master("huntingquest_stage");
  let rewards = get_master_manager()
    .get_master("huntingquest_stage_itemreward")
    .into_iter()
    .map(|reward| (reward["id"].as_str().unwrap().parse::<i32>().unwrap(), reward))
    .collect::<HashMap<_, _>>();

  let stages = stages.iter().filter(|stage| {
    let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
    let rewards = extract_items(&rewards[&id]);

    rewards
      .iter()
      .any(|item| item.item_type == params.item_type && item.item_id == params.item_id)
  });

  Ok(Unsigned(HuntingQuestListByItemResponse {
    huntingquests: stages
      .map(|stage| {
        let id = stage["id"].as_str().unwrap().parse::<i32>().unwrap();
        HuntingQuest {
          quest_id: id,
          task1: 1,
          task2: 1,
          task3: 0,
          limit: 42,
          status: 2,
        }
      })
      .collect(),
    eventboxgachas: vec![20043],
    exchanges: vec![100],
    expedition: 0,
    scorechallenge: 0,
    scorechallenge_ex: 0,
    dungeon: DungeonAreaMaterialInfoResponseDto {
      area_ids: vec![],
      unlocked_area_ids: vec![],
      challenging_area_id: 0,
    },
    fame_quest: vec![],
  }))
}
