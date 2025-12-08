use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::api::master_all::get_masters_definitely_initialized;

pub struct IntimacyRankCalculator {
  absolute_xp_to_rank: BTreeMap<i32, i32>,
  rank_to_absolute_xp: BTreeMap<i32, i32>,
}

impl IntimacyRankCalculator {
  fn new() -> Self {
    let masters = get_masters_definitely_initialized();
    let levels: Vec<Value> = serde_json::from_str(&masters["intimacy_exp"].master_decompressed).unwrap();

    let mut level_data = Vec::new();
    for level in levels {
      let rank: i32 = level["intimacy_lv"].as_str().unwrap().parse().unwrap();
      let required_exp: i32 = level["exp"].as_str().unwrap().parse().unwrap();
      level_data.push((rank, required_exp));
    }
    Self::from_levels(&level_data)
  }

  pub fn from_levels(levels: &[(i32, i32)]) -> Self {
    let mut cumulative_xp = 0;
    let mut absolute_xp_to_rank = BTreeMap::new();
    let mut rank_to_absolute_xp = BTreeMap::new();
    for &(rank, required_exp) in levels {
      cumulative_xp += required_exp;
      absolute_xp_to_rank.insert(cumulative_xp, rank);
      rank_to_absolute_xp.insert(rank, cumulative_xp);
    }

    Self {
      absolute_xp_to_rank,
      rank_to_absolute_xp,
    }
  }

  pub fn get_rank(&self, exp: i32) -> i32 {
    match self.absolute_xp_to_rank.range(..=exp).next_back() {
      Some((_, &rank)) => rank,
      None => 0,
    }
  }

  pub fn get_xp_for_rank(&self, rank: i32) -> Option<i32> {
    self.rank_to_absolute_xp.get(&rank).copied()
  }
}

static INTIMACY_RANK_CALCULATOR: OnceLock<IntimacyRankCalculator> = OnceLock::new();

pub fn get_intimacy_rank_calculator() -> &'static IntimacyRankCalculator {
  INTIMACY_RANK_CALCULATOR.get_or_init(IntimacyRankCalculator::new)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_calculator() -> IntimacyRankCalculator {
    // Cumulative XP: 0 -> 20 (rank 2), 20 -> 80 (rank 3), 80 -> 230 (rank 4), etc.
    #[cfg_attr(rustfmt, rustfmt::skip)]
    IntimacyRankCalculator::from_levels(&[
      (2, 20),
      (3, 60),
      (4, 150),
      (5, 250),
      (10, 1500),
      (20, 7000),
    ])
  }

  #[test]
  fn test_get_rank_zero_xp() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_rank(0), 0);
  }

  #[test]
  fn test_get_rank_exact_threshold() {
    let calculator = create_test_calculator();
    // 20 XP is exactly rank 2
    assert_eq!(calculator.get_rank(20), 2);
    // 80 XP is exactly rank 3 (20 + 60)
    assert_eq!(calculator.get_rank(80), 3);
  }

  #[test]
  fn test_get_rank_between_thresholds() {
    let calculator = create_test_calculator();
    // 50 XP is between rank 2 (20) and rank 3 (80), so should be rank 2
    assert_eq!(calculator.get_rank(50), 2);
    // 100 XP is between rank 3 (80) and rank 4 (230), so should be rank 3
    assert_eq!(calculator.get_rank(100), 3);
  }

  #[test]
  fn test_get_rank_one_below_threshold() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_rank(19), 0);
    assert_eq!(calculator.get_rank(79), 2);
  }

  #[test]
  fn test_get_rank_one_above_threshold() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_rank(21), 2);
    assert_eq!(calculator.get_rank(81), 3);
  }

  #[test]
  fn test_get_rank_max_xp() {
    let calculator = create_test_calculator();
    // Very high XP should return the highest rank
    assert_eq!(calculator.get_rank(999999), 20);
  }

  #[test]
  fn test_get_xp_for_rank_valid() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_xp_for_rank(2), Some(20));
    assert_eq!(calculator.get_xp_for_rank(3), Some(80));
    assert_eq!(calculator.get_xp_for_rank(4), Some(230));
  }

  #[test]
  fn test_get_xp_for_rank_invalid() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_xp_for_rank(1), None);
    assert_eq!(calculator.get_xp_for_rank(100), None);
    assert_eq!(calculator.get_xp_for_rank(-1), None);
  }

  #[test]
  fn test_get_xp_for_rank_boundaries() {
    let calculator = create_test_calculator();
    // First rank
    assert_eq!(calculator.get_xp_for_rank(2), Some(20));
    // Last rank in our test data
    assert_eq!(calculator.get_xp_for_rank(20), Some(8980));
  }

  #[test]
  fn test_cumulative_xp_calculation() {
    let calculator = create_test_calculator();
    // Verify cumulative XP: 20 + 60 = 80
    assert_eq!(calculator.get_rank(80), 3);
    // 20 + 60 + 150 = 230
    assert_eq!(calculator.get_rank(230), 4);
    // 20 + 60 + 150 + 250 = 480
    assert_eq!(calculator.get_rank(480), 5);
  }

  #[test]
  fn test_negative_xp() {
    let calculator = create_test_calculator();
    assert_eq!(calculator.get_rank(-100), 0);
  }
}
