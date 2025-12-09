use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::api::master_all::get_master_manager;

pub struct IntimacyLevelCalculator {
  absolute_xp_to_level: BTreeMap<i32, i32>,
  level_to_absolute_xp: BTreeMap<i32, i32>,
}

impl IntimacyLevelCalculator {
  fn new() -> Self {
    let mut levels = Vec::new();
    for entry in get_master_manager().get_master("intimacy_exp") {
      let level: i32 = entry["intimacy_lv"].as_str().unwrap().parse().unwrap();
      let required_exp: i32 = entry["exp"].as_str().unwrap().parse().unwrap();
      levels.push((level, required_exp));
    }
    Self::from_levels(&levels)
  }

  pub fn from_levels(levels: &[(i32, i32)]) -> Self {
    let mut cumulative_xp = 0;
    let mut absolute_xp_to_level = BTreeMap::new();
    let mut level_to_absolute_xp = BTreeMap::new();
    for &(level, required_exp) in levels {
      cumulative_xp += required_exp;
      absolute_xp_to_level.insert(cumulative_xp, level);
      level_to_absolute_xp.insert(level, cumulative_xp);
    }

    Self {
      absolute_xp_to_level,
      level_to_absolute_xp,
    }
  }

  pub fn get_level(&self, exp: i32) -> i32 {
    match self.absolute_xp_to_level.range(..=exp).next_back() {
      Some((_, &level)) => level,
      None => 0,
    }
  }

  pub fn get_xp_for_level(&self, level: i32) -> Option<i32> {
    self.level_to_absolute_xp.get(&level).copied()
  }
}

static INTIMACY_LEVEL_CALCULATOR: OnceLock<IntimacyLevelCalculator> = OnceLock::new();

pub fn get_intimacy_level_calculator() -> &'static IntimacyLevelCalculator {
  INTIMACY_LEVEL_CALCULATOR.get_or_init(IntimacyLevelCalculator::new)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_calculator() -> IntimacyLevelCalculator {
    // Cumulative XP: 0 -> 20 (level 2), 20 -> 80 (level 3), 80 -> 230 (level 4), etc.
    #[cfg_attr(rustfmt, rustfmt::skip)]
    IntimacyLevelCalculator::from_levels(&[
      (2, 20),
      (3, 60),
      (4, 150),
      (5, 250),
      (10, 1500),
      (20, 7000),
    ])
  }

  #[test]
  fn test_get_level_zero_xp() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_level(0), 0);
  }

  #[test]
  fn test_get_level_exact_threshold() {
    let calculator = create_test_calculator();
    // 20 XP is exactly level 2
    assert_eq!(calculator.get_level(20), 2);
    // 80 XP is exactly level 3 (20 + 60)
    assert_eq!(calculator.get_level(80), 3);
  }

  #[test]
  fn test_get_level_between_thresholds() {
    let calculator = create_test_calculator();
    // 50 XP is between level 2 (20) and level 3 (80), so should be level 2
    assert_eq!(calculator.get_level(50), 2);
    // 100 XP is between level 3 (80) and level 4 (230), so should be level 3
    assert_eq!(calculator.get_level(100), 3);
  }

  #[test]
  fn test_get_level_one_below_threshold() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_level(19), 0);
    assert_eq!(calculator.get_level(79), 2);
  }

  #[test]
  fn test_get_level_one_above_threshold() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_level(21), 2);
    assert_eq!(calculator.get_level(81), 3);
  }

  #[test]
  fn test_get_level_max_xp() {
    let calculator = create_test_calculator();
    // Very high XP should return the highest level
    assert_eq!(calculator.get_level(999999), 20);
  }

  #[test]
  fn test_get_xp_for_level_valid() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_xp_for_level(2), Some(20));
    assert_eq!(calculator.get_xp_for_level(3), Some(80));
    assert_eq!(calculator.get_xp_for_level(4), Some(230));
  }

  #[test]
  fn test_get_xp_for_level_invalid() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_xp_for_level(1), None);
    assert_eq!(calculator.get_xp_for_level(100), None);
    assert_eq!(calculator.get_xp_for_level(-1), None);
  }

  #[test]
  fn test_get_xp_for_level_boundaries() {
    let calculator = create_test_calculator();
    // First level
    assert_eq!(calculator.get_xp_for_level(2), Some(20));
    // Last level in our test data
    assert_eq!(calculator.get_xp_for_level(20), Some(8980));
  }

  #[test]
  fn test_cumulative_xp_calculation() {
    let calculator = create_test_calculator();
    // Verify cumulative XP: 20 + 60 = 80
    assert_eq!(calculator.get_level(80), 3);
    // 20 + 60 + 150 = 230
    assert_eq!(calculator.get_level(230), 4);
    // 20 + 60 + 150 + 250 = 480
    assert_eq!(calculator.get_level(480), 5);
  }

  #[test]
  fn test_negative_xp() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_level(-100), 0);
  }
}
