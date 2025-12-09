use serde_json::Value;

use crate::api::MemberParameterWire;
use crate::api::dungeon::PartyMember;
use crate::api::master_all::get_masters_definitely_initialized;

/// Member information from master data. Can be used to create new [Member] instances.
#[derive(Debug, Clone)]
pub struct MemberPrototype {
  pub id: i64,
  pub character_id: i64,
  pub rarity: i32,
  /// "Skill"
  pub active_skills: [Option<ActiveSkillPrototype>; 3],
  /// "Trait"
  pub passive_skill: Option<PassiveSkillPrototype>,
  pub special_attack: Option<SpecialAttackPrototype>,
  pub resistance_group: ResistanceGroupPrototype,
  pub stats: MemberStatsPrototype,
}

impl MemberPrototype {
  pub fn load_from_id(id: i64) -> Self {
    let masters = get_masters_definitely_initialized();
    let members: Vec<Value> = serde_json::from_str(&masters["member"].master_decompressed).unwrap();
    let active_skills: Vec<Value> = serde_json::from_str(&masters["skill_ac_details"].master_decompressed).unwrap();

    let member = members
      .iter()
      .find(|m| m["id"].as_str().unwrap().parse::<i64>().unwrap() == id)
      .unwrap();
    let character_id = member["character_id"].as_str().unwrap().parse::<i64>().unwrap();
    let rarity = member["rare"].as_str().unwrap().parse::<i32>().unwrap();

    let create_skill = |key: &str| -> ActiveSkillPrototype {
      let skill_id = key.parse::<i64>().unwrap();
      let skill_detail = active_skills
        .iter()
        .find(|sk| sk["skill_id"].as_str().unwrap().parse::<i64>().unwrap() == skill_id)
        .unwrap();
      ActiveSkillPrototype {
        id: skill_id,
        value: MinMaxRange {
          min: skill_detail["value_min"].as_str().unwrap().parse::<i32>().unwrap(),
          max: skill_detail["value_max"].as_str().unwrap().parse::<i32>().unwrap(),
        },
      }
    };

    let active_skills = [
      member["activeskill1"]
        .as_str()
        .take_if(|s| *s != "0")
        .map(|s| create_skill(s)),
      member["activeskill2"]
        .as_str()
        .take_if(|s| *s != "0")
        .map(|s| create_skill(s)),
      member["activeskill3"]
        .as_str()
        .take_if(|s| *s != "0")
        .map(|s| create_skill(s)),
    ];
    let passive_skill = member["passiveskill"]
      .as_str()
      .take_if(|s| *s != "0")
      .map(|s| PassiveSkillPrototype {
        id: s.parse::<i64>().unwrap(),
      });
    let special_attack = None;
    let resistance_group = ResistanceGroupPrototype {
      id: member["resist_attr"].as_str().unwrap().parse::<i32>().unwrap(),
    };

    let stats = MemberStatsPrototype {
      hp: MinMaxRange::new(
        member["min_hp"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_hp"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      attack: MinMaxRange::new(
        member["min_attack"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_attack"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      magic_attack: MinMaxRange::new(
        member["min_magicattak"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_magicattak"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      defense: MinMaxRange::new(
        member["min_defense"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_defense"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      magic_defense: MinMaxRange::new(
        member["min_magicdefence"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_magicdefence"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      agility: MinMaxRange::new(
        member["min_agility"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_agility"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      dexterity: MinMaxRange::new(
        member["min_dexterity"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_dexterity"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      luck: MinMaxRange::new(
        member["min_luck"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_luck"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
    };

    Self {
      id,
      character_id,
      rarity,
      active_skills,
      passive_skill,
      special_attack,
      resistance_group,
      stats,
    }
  }

  /// Creates a new [Member] instance from this prototype.
  /// TODO: TEMPORARY
  pub fn create_member_wire(&self) -> MemberParameterWire {
    MemberParameterWire {
      id: 0,
      lv: 1,
      exp: 0,
      member_id: self.id,
      ac_skill_id_a: 0,
      ac_skill_lv_a: 1,
      ac_skill_val_a: 0,
      ac_skill_id_b: 0,
      ac_skill_lv_b: 1,
      ac_skill_val_b: 0,
      ac_skill_id_c: 0,
      ac_skill_lv_c: 1,
      ac_skill_val_c: 0,
      hp: self.stats.hp.max,
      magicattack: self.stats.magic_attack.max,
      defense: self.stats.defense.max,
      magicdefence: self.stats.magic_defense.max,
      agility: self.stats.agility.max,
      dexterity: self.stats.dexterity.max,
      luck: self.stats.luck.max,
      limit_break: 0,
      character_id: self.character_id,
      passiveskill: 0,
      specialattack: 0,
      resist_state: self.resistance_group.id,
      resist_attr: 0,
      attack: self.stats.attack.min,
      waiting_room: 0,
      main_strength: 0,
      main_strength_for_fame_quest: 0,
      sub_strength: 0,
      sub_strength_for_fame_quest: 0,
      sub_strength_bonus: 0,
      sub_strength_bonus_for_fame_quest: 0,
      fame_hp_rank: 0,
      fame_attack_rank: 0,
      fame_defense_rank: 0,
      fame_magicattack_rank: 0,
      fame_magicdefence_rank: 0,
      skill_pa_fame_list: vec![],
    }
  }

  pub fn create_party_member_wire(&self, id: i32) -> PartyMember {
    PartyMember {
      id,
      lv: 1,
      exp: 0,
      member_id: self.id,
      ac_skill_lv_a: self.active_skills[0].as_ref().map_or(0, |skill| 1),
      ac_skill_val_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.value.max as i64),
      ac_skill_lv_b: self.active_skills[1].as_ref().map_or(0, |skill| 1),
      ac_skill_val_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.value.max as i64),
      ac_skill_lv_c: self.active_skills[2].as_ref().map_or(0, |skill| 1),
      ac_skill_val_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.value.max as i64),
      hp: self.stats.hp.max,
      attack: self.stats.attack.max,
      magicattack: self.stats.magic_attack.max,
      defense: self.stats.defense.max,
      magicdefence: self.stats.magic_defense.max,
      agility: self.stats.agility.max,
      dexterity: self.stats.dexterity.max,
      luck: self.stats.luck.max,
      limit_break: 0,
      character_id: self.character_id,
      waiting_room: 0,
      ex_flg: 0,
      is_undead: 0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MemberStatsPrototype {
  pub hp: MinMaxRange,
  pub attack: MinMaxRange,
  pub magic_attack: MinMaxRange,
  pub defense: MinMaxRange,
  pub magic_defense: MinMaxRange,
  pub agility: MinMaxRange,
  pub dexterity: MinMaxRange,
  pub luck: MinMaxRange,
}

#[derive(Debug, Clone)]
pub struct ActiveSkillPrototype {
  pub id: i64,
  pub value: MinMaxRange,
}

#[derive(Debug, Clone)]
pub struct PassiveSkillPrototype {
  pub id: i64,
}

#[derive(Debug, Clone)]
pub struct SpecialAttackPrototype {
  pub id: i64,
}

#[derive(Debug, Clone)]
pub struct ResistanceGroupPrototype {
  pub id: i32,
}

#[derive(Debug, Clone)]
pub struct MinMaxRange {
  pub min: i32,
  pub max: i32,
}

impl MinMaxRange {
  pub fn new(min: i32, max: i32) -> Self {
    Self { min, max }
  }
}
