use crate::api::battle::BattleMember;
use crate::api::dungeon::{DungeonBattleMember, PartyMember};
use crate::api::master_all::get_master_manager;
use crate::api::{MemberFameStats, MemberParameterWire, MemberStats, SkillPaFame};
use crate::level::get_member_level_calculator;

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
    let masters = get_master_manager();
    let members = masters.get_master("member");
    let active_skills = masters.get_master("skill_ac_details");

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
      attack_magic: MinMaxRange::new(
        member["min_magicattak"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_magicattak"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      defense: MinMaxRange::new(
        member["min_defense"].as_str().unwrap().parse::<i32>().unwrap(),
        member["max_defense"].as_str().unwrap().parse::<i32>().unwrap(),
      ),
      defense_magic: MinMaxRange::new(
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
  pub fn create_member(&self, id: i32) -> Member<'_> {
    Member {
      id,
      prototype: self,
      xp: if self.character_id == 102 { 150_000 } else { 35_000 } * 20,
      promotion_level: 2,
      active_skills: self
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
      stats: self.stats.clone(),
      main_strength: MemberStrength::default(),
      sub_strength: MemberStrength::default(),
      sub_strength_bonus: MemberStrength::default(),
      fame_stats: MemberFameStats::default(),
      skill_pa_fame_list: vec![],
    }
  }
}

/// ## Members vs Characters
/// *Members* are playable character units you build your team with (Front, Back, Sub),
/// while *Characters* are the actual people from the anime (Kazuma, Aqua, Megumin, etc.)
/// that these units represent. A single Character can have multiple Member versions
/// (e.g., "Yunyun (Beginnger)" and "Yunyun (Wakey Wakey)"), each with unique stats
/// that determine how effective they are in battle as a Front, Back, or Sub member.
// See [Wonder_Data_MemberParameter_Fields]
#[derive(Debug)]
pub struct Member<'a> {
  pub id: i32,
  pub prototype: &'a MemberPrototype,
  pub xp: i32,
  /// "Promotions"
  pub promotion_level: i32,
  pub active_skills: [Option<MemberActiveSkill<'a>>; 3],
  pub stats: MemberStatsPrototype,
  pub main_strength: MemberStrength,
  pub sub_strength: MemberStrength,
  pub sub_strength_bonus: MemberStrength,
  pub fame_stats: MemberFameStats,
  pub skill_pa_fame_list: Vec<SkillPaFame>,
}

impl Member<'_> {
  /// ## Level
  /// Members have level cap ('member_lv_limit' master data, currently level 30 for all)
  /// based on their rarity.
  ///
  /// ## Promotion
  /// Called 'limit_break' internally, promotions increase the level cap of a member
  /// ('member_lv_limitbreak' master data, currently 5 promotions resulting in 30 bonus levels).
  pub fn level(&self) -> i32 {
    get_member_level_calculator().get_level(self.xp, self.prototype.rarity, self.promotion_level)
  }

  pub fn to_member_parameter_wire(&self) -> MemberParameterWire {
    MemberParameterWire {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.value),
      hp: self.stats.hp.interpolate(self.level()),
      magicattack: self.stats.attack_magic.interpolate(self.level()),
      defense: self.stats.defense.interpolate(self.level()),
      magicdefence: self.stats.defense_magic.interpolate(self.level()),
      agility: self.stats.agility.interpolate(self.level()),
      dexterity: self.stats.dexterity.interpolate(self.level()),
      luck: self.stats.luck.interpolate(self.level()),
      limit_break: self.promotion_level,
      character_id: self.prototype.character_id,
      passiveskill: self.prototype.passive_skill.as_ref().map_or(0, |skill| skill.id),
      specialattack: self.prototype.special_attack.as_ref().map_or(0, |skill| skill.id),
      resist_state: self.prototype.resistance_group.id,
      resist_attr: 0,
      attack: self.stats.attack.interpolate(self.level()),
      waiting_room: 0,
      main_strength: self.main_strength.strength,
      main_strength_for_fame_quest: self.main_strength.for_fame_quest,
      sub_strength: self.sub_strength.strength,
      sub_strength_for_fame_quest: self.sub_strength.for_fame_quest,
      sub_strength_bonus: self.sub_strength_bonus.strength,
      sub_strength_bonus_for_fame_quest: self.sub_strength_bonus.for_fame_quest,
      fame_hp_rank: self.fame_stats.fame_hp,
      fame_attack_rank: self.fame_stats.fame_attack,
      fame_defense_rank: self.fame_stats.fame_defense,
      fame_magicattack_rank: self.fame_stats.fame_magicattack,
      fame_magicdefence_rank: self.fame_stats.fame_magicdefence,
      skill_pa_fame_list: self.skill_pa_fame_list.clone(),
    }
  }

  pub fn to_party_member(&self) -> PartyMember {
    PartyMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_lv_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.value as i64),
      ac_skill_lv_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.value as i64),
      ac_skill_lv_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.value as i64),
      hp: self.stats.hp.interpolate(self.level()),
      attack: self.stats.attack.interpolate(self.level()),
      magicattack: self.stats.attack_magic.interpolate(self.level()),
      defense: self.stats.defense.interpolate(self.level()),
      magicdefence: self.stats.defense_magic.interpolate(self.level()),
      agility: self.stats.agility.interpolate(self.level()),
      dexterity: self.stats.dexterity.interpolate(self.level()),
      luck: self.stats.luck.interpolate(self.level()),
      limit_break: self.promotion_level,
      character_id: self.prototype.character_id,
      waiting_room: 0,
      ex_flg: 0,
      is_undead: 0,
    }
  }

  pub fn to_battle_member(&self) -> BattleMember {
    BattleMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.value),
      hp: self.stats.hp.interpolate(self.level()),
      magicattack: self.stats.attack_magic.interpolate(self.level()),
      defense: self.stats.defense.interpolate(self.level()),
      magicdefence: self.stats.defense_magic.interpolate(self.level()),
      agility: self.stats.agility.interpolate(self.level()),
      dexterity: self.stats.dexterity.interpolate(self.level()),
      luck: self.stats.luck.interpolate(self.level()),
      limit_break: self.promotion_level,
      character_id: self.prototype.character_id,
      passiveskill: 210201,  // self.prototype.passive_skill.as_ref().map_or(0, |skill| skill.id),
      specialattack: 100001, // self.prototype.special_attack.as_ref().map_or(0, |skill| skill.id),
      resist_state: 210201,  // self.prototype.resistance_group.id,
      resist_attr: 150000000,
      attack: self.stats.attack.interpolate(self.level()),
      ex_flg: 0,
      is_undead: 0,
      special_skill_lv: 1,
    }
  }

  pub fn to_dungeon_battle_member(&self) -> DungeonBattleMember {
    DungeonBattleMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: self.active_skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: self.active_skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: self.active_skills[2].as_ref().map_or(0, |skill| skill.value),
      hp: self.stats.hp.interpolate(self.level()),
      magicattack: self.stats.attack_magic.interpolate(self.level()),
      defense: self.stats.defense.interpolate(self.level()),
      magicdefence: self.stats.defense_magic.interpolate(self.level()),
      agility: self.stats.agility.interpolate(self.level()),
      dexterity: self.stats.dexterity.interpolate(self.level()),
      luck: self.stats.luck.interpolate(self.level()),
      limit_break: self.promotion_level,
      character_id: self.prototype.character_id,
      passiveskill: 210201,  // self.prototype.passive_skill.as_ref().map_or(0, |skill| skill.id),
      specialattack: 100001, // self.prototype.special_attack.as_ref().map_or(0, |skill| skill.id),
      resist_state: 210201,  // self.prototype.resistance_group.id,
      resist_attr: 150000000,
      attack: self.stats.attack.interpolate(self.level()),
      ex_flg: 0,
      is_undead: 0,
      special_skill_lv: 1,
      character_piece_board_stage_id_list: vec![100001001, 100002002, 100003003, 100004004],
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct MemberStrength {
  pub strength: i32,
  pub for_fame_quest: i32,
}

#[derive(Debug)]
pub struct MemberActiveSkill<'a> {
  pub prototype: &'a ActiveSkillPrototype,
  pub level: i32,
  pub value: i32,
}

#[derive(Debug, Clone)]
pub struct MemberStatsPrototype {
  pub hp: MinMaxRange,
  pub attack: MinMaxRange,
  pub attack_magic: MinMaxRange,
  pub defense: MinMaxRange,
  pub defense_magic: MinMaxRange,
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

  pub fn interpolate(&self, level: i32) -> i32 {
    const MAX_LEVEL: i32 = 60;
    let ratio = (level - 1) as f32 / (MAX_LEVEL - 1) as f32;
    (self.min as f32 + (self.max - self.min) as f32 * ratio).round() as i32
  }
}
