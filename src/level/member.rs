use crate::api::master_all::get_master_manager;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub struct MemberLevelCalculator {
  // (rarity, (xp, level))
  absolute_xp_to_level: BTreeMap<i32, BTreeMap<i32, i32>>,
  // (rarity, (level, xp))
  level_to_absolute_xp: BTreeMap<i32, BTreeMap<i32, i32>>,
}

impl MemberLevelCalculator {
  fn new() -> Self {
    let rarities = [1, 2, 3, 4];
    let mut levels = Vec::new();
    for rarity in rarities {
      let mut levels_for_rarity = Vec::new();
      for entry in get_master_manager().get_master("member_lv_exp") {
        let level: i32 = entry["lv"].as_str().unwrap().parse().unwrap();
        let xp: i32 = entry[&format!("exp_rare_{}", rarity)]
          .as_str()
          .unwrap()
          .parse()
          .unwrap();
        levels_for_rarity.push((level, xp));
      }
      levels.push((rarity, levels_for_rarity));
    }
    Self::from_levels(&levels)
  }

  pub fn from_levels(levels: &[(i32, Vec<(i32, i32)>)]) -> Self {
    let mut absolute_xp_to_level = BTreeMap::new();
    let mut level_to_absolute_xp = BTreeMap::new();
    for (rarity, levels_for_rarity) in levels {
      let mut absolute_xp_to_level_local = BTreeMap::new();
      let mut level_to_absolute_xp_local = BTreeMap::new();
      for &(level, required_exp) in levels_for_rarity {
        absolute_xp_to_level_local.insert(required_exp, level);
        level_to_absolute_xp_local.insert(level, required_exp);
      }

      absolute_xp_to_level.insert(*rarity, absolute_xp_to_level_local);
      level_to_absolute_xp.insert(*rarity, level_to_absolute_xp_local);
    }

    Self {
      absolute_xp_to_level,
      level_to_absolute_xp,
    }
  }

  pub fn get_level(&self, rarity: i32, exp: i32) -> i32 {
    let absolute_xp_to_level_local = match self.absolute_xp_to_level.get(&rarity) {
      Some(map) => map,
      None => todo!("handle unknown rarity {}", rarity),
    };
    match absolute_xp_to_level_local.range(..=exp).next_back() {
      Some((_, &level)) => level,
      None => 0,
    }
  }

  pub fn get_xp_for_level(&self, level: i32) -> Option<i32> {
    let level_to_absolute_xp_local = match self.level_to_absolute_xp.get(&level) {
      Some(map) => map,
      None => return None,
    };
    level_to_absolute_xp_local.get(&level).copied()
  }
}

static MEMBER_LEVEL_CALCULATOR: OnceLock<MemberLevelCalculator> = OnceLock::new();

pub fn get_member_level_calculator() -> &'static MemberLevelCalculator {
  MEMBER_LEVEL_CALCULATOR.get_or_init(MemberLevelCalculator::new)
}
