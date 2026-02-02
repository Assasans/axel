use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod quest_fame;
pub mod quest_hunting;
pub mod quest_main;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestRewardItem {
  pub item_type: i32,
  pub item_id: i64,
  pub item_num: i32,
  pub item_rare: bool,
}

/// Parses quest reward items from:
/// - fame_quest_stage_itemreward
/// - huntingquest_stage_itemreward
/// - mainquest_stage_itemreward
/// - dungeon_stage_item_reward
/// - event_quest_stage_itemreward
// TODO: event_quest_boss_stage_itemreward support?
pub fn parse_reward_items(value: &Value) -> Vec<QuestRewardItem> {
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
      // If item_num is missing, default to 1
      let item_num = object
        .get(format!("item_num{i}").as_str())
        .map(|v| v.as_str().unwrap().parse::<i32>().unwrap())
        .unwrap_or(1);
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

      Some(QuestRewardItem {
        item_type,
        item_id,
        item_num,
        item_rare,
      })
    })
    .collect()
}
