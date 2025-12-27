use crate::api::master_all::get_master_manager;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub struct MemberLevelCalculator {
  // (rarity, (xp, level))
  absolute_xp_to_level: BTreeMap<i32, BTreeMap<i32, i32>>,
  // (rarity, (level, xp))
  level_to_absolute_xp: BTreeMap<i32, BTreeMap<i32, i32>>,
  // (rarity, max_level)
  level_limits: BTreeMap<i32, i32>,
  // (promotion_level, cumulative{bonus_levels))
  promotion_levels: BTreeMap<i32, i32>,
}

impl MemberLevelCalculator {
  fn new() -> Self {
    let rarities = [1, 2, 3, 4];
    let mut absolute_xp_to_level = BTreeMap::new();
    let mut level_to_absolute_xp = BTreeMap::new();
    for rarity in rarities {
      let mut xp_to_level = BTreeMap::new();
      let mut level_to_xp = BTreeMap::new();
      for entry in get_master_manager().get_master("member_lv_exp") {
        let level: i32 = entry["lv"].as_str().unwrap().parse().unwrap();
        let xp: i32 = entry[&format!("exp_rare_{}", rarity)]
          .as_str()
          .unwrap()
          .parse()
          .unwrap();
        xp_to_level.insert(xp, level);
        level_to_xp.insert(level, xp);
      }
      absolute_xp_to_level.insert(rarity, xp_to_level);
      level_to_absolute_xp.insert(rarity, level_to_xp);
    }

    let mut level_limits = BTreeMap::new();
    for entry in get_master_manager().get_master("member_lv_limit") {
      let rarity: i32 = entry["rare"].as_str().unwrap().parse().unwrap();
      let max_level: i32 = entry["lv_limit"].as_str().unwrap().parse().unwrap();
      level_limits.insert(rarity, max_level);
    }

    let mut promotion_levels = BTreeMap::new();
    let mut cumulative = 0;
    for promotion_entry in get_master_manager().get_master("member_lv_limitbreak") {
      let promotion_level: i32 = promotion_entry["lv"].as_str().unwrap().parse().unwrap();
      let bonus_levels: i32 = promotion_entry["lv_limit"].as_str().unwrap().parse().unwrap();
      cumulative += bonus_levels;
      promotion_levels.insert(promotion_level, cumulative);
    }

    Self {
      absolute_xp_to_level,
      level_to_absolute_xp,
      level_limits,
      promotion_levels,
    }
  }

  pub fn get_level(&self, xp: i32, rarity: i32, promotion_level: i32) -> i32 {
    let absolute_xp_to_level_local = match self.absolute_xp_to_level.get(&rarity) {
      Some(map) => map,
      None => todo!("handle unknown rarity {}", rarity),
    };

    let level = match absolute_xp_to_level_local.range(..=xp).next_back() {
      Some((_, &level)) => level,
      // 'member_lv_exp' starts from level 2, so this branch is actually reachable
      None => 1,
    };

    level.min(self.get_max_level(rarity, promotion_level))
  }

  /// Returns the total XP required to reach the given level for a member of the given rarity,
  /// ignoring level caps.
  pub fn get_xp_for_level(&self, level: i32, rarity: i32) -> i32 {
    let level_to_absolute_xp_local = match self.level_to_absolute_xp.get(&rarity) {
      Some(map) => map,
      None => todo!("handle unknown rarity {}", rarity),
    };

    match level_to_absolute_xp_local.range(..=level).next_back() {
      Some((_, &xp)) => xp,
      // 'member_lv_exp' starts from level 2, so this branch is actually reachable
      None => 0,
    }
  }

  pub fn get_max_level(&self, rarity: i32, promotion_level: i32) -> i32 {
    let base_limit = match self.level_limits.get(&rarity) {
      Some(&limit) => limit,
      None => todo!("handle unknown rarity {}", rarity),
    };

    let bonus_limits = match self.promotion_levels.get(&promotion_level) {
      Some(&bonus) => bonus,
      None if promotion_level == 0 => 0,
      None => todo!("handle unknown promotion level {}", promotion_level),
    };

    base_limit + bonus_limits
  }
}

static MEMBER_LEVEL_CALCULATOR: OnceLock<MemberLevelCalculator> = OnceLock::new();

pub fn get_member_level_calculator() -> &'static MemberLevelCalculator {
  MEMBER_LEVEL_CALCULATOR.get_or_init(MemberLevelCalculator::new)
}
