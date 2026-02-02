use crate::api::battle::BattleMember;
use crate::api::dungeon::{DungeonBattleMember, PartyMember};
use crate::api::master_all::get_master_manager;
use crate::api::party_info::{Party, PartyForm, PartyPassiveSkillInfo, SpecialSkillInfo};
use crate::api::{MemberFameStats, MemberParameterWire, SkillPaFame};
use crate::database::QueryExecutor;
use crate::level::get_member_level_calculator;
use crate::user::id::UserId;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::{Row, Statement};
use tracing::warn;

/// Member information from master data. Can be used to create new [Member] instances.
#[derive(Debug, Clone)]
pub struct MemberPrototype {
  pub id: i64,
  pub character_id: i64,
  pub rarity: i32,
  /// "Skill"
  // TODO: Should it be 'struct SkillHandle(Arc<MemberPrototype>, usize)' instead of Arc?
  pub active_skills: [Option<Arc<ActiveSkillPrototype>>; 3],
  /// "Trait"
  pub passive_skill: Option<PassiveSkillPrototype>,
  pub special_attack: Option<SpecialAttackPrototype>,
  pub resistance_group: ResistanceGroupPrototype,
  pub stats: MemberStatsPrototype,
}

impl MemberPrototype {
  pub fn load_from_id(id: i64) -> Arc<Self> {
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
        .map(|s| Arc::new(create_skill(s))),
      member["activeskill2"]
        .as_str()
        .take_if(|s| *s != "0")
        .map(|s| Arc::new(create_skill(s))),
      member["activeskill3"]
        .as_str()
        .take_if(|s| *s != "0")
        .map(|s| Arc::new(create_skill(s))),
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

    // TODO: Caching?
    Arc::new(Self {
      id,
      character_id,
      rarity,
      active_skills,
      passive_skill,
      special_attack,
      resistance_group,
      stats,
    })
  }

  /// Creates a new [Member] instance from this prototype.
  pub fn create_member(self: &Arc<Self>, id: i32) -> Member {
    Member {
      id,
      prototype: self.clone(),
      xp: 0,
      promotion_level: 0,
      active_skills: OptionallyFetched::Fetched(
        self
          .active_skills
          .iter()
          .map(|skill_opt| {
            skill_opt.as_ref().map(|skill| MemberActiveSkill {
              prototype: skill.clone(),
              level: 1,
              value: skill.value.max,
            })
          })
          .collect::<Vec<_>>()
          .try_into()
          .unwrap(),
      ),
      stats: self.stats.clone(),
      main_strength: MemberStrength::default(),
      sub_strength: MemberStrength::default(),
      sub_strength_bonus: MemberStrength::default(),
      fame_stats: MemberFameStats::default(),
      skill_pa_fame_list: vec![],
    }
  }

  pub fn create_reserve_member(self: &Arc<Self>, id: i32) -> Member {
    Member {
      id,
      prototype: self.clone(),
      xp: 0,
      promotion_level: 0,
      active_skills: OptionallyFetched::Fetched([None, None, None]),
      stats: self.stats.clone(),
      main_strength: MemberStrength::default(),
      sub_strength: MemberStrength::default(),
      sub_strength_bonus: MemberStrength::default(),
      fame_stats: MemberFameStats::default(),
      skill_pa_fame_list: vec![],
    }
  }
}

#[derive(Debug)]
pub enum OptionallyFetched<T> {
  Fetched(T),
  Unfetched,
}

/// ## Members vs Characters
/// *Members* are playable character units you build your team with (Front, Back, Sub),
/// while *Characters* are the actual people from the anime (Kazuma, Aqua, Megumin, etc.)
/// that these units represent. A single Character can have multiple Member versions
/// (e.g., "Yunyun (Beginnger)" and "Yunyun (Wakey Wakey)"), each with unique stats
/// that determine how effective they are in battle as a Front, Back, or Sub member.
// See [Wonder_Data_MemberParameter_Fields]
#[derive(Debug)]
pub struct Member {
  // TODO: Should be i64, it is BIGINT in database and it being i32 just constantly
  //  causes type-cast errors in database queries.
  pub id: i32,
  pub prototype: Arc<MemberPrototype>,
  pub xp: i32,
  /// "Promotions"
  pub promotion_level: i32,
  pub active_skills: OptionallyFetched<[Option<MemberActiveSkill>; 3]>,
  pub stats: MemberStatsPrototype,
  pub main_strength: MemberStrength,
  pub sub_strength: MemberStrength,
  pub sub_strength_bonus: MemberStrength,
  pub fame_stats: MemberFameStats,
  pub skill_pa_fame_list: Vec<SkillPaFame>,
}

impl Member {
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
    let skills = match &self.active_skills {
      OptionallyFetched::Fetched(skills) => skills,
      OptionallyFetched::Unfetched => panic!("active skills not fetched for member {}", self.id),
    };

    MemberParameterWire {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: skills[2].as_ref().map_or(0, |skill| skill.value),
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
    let skills = match &self.active_skills {
      OptionallyFetched::Fetched(skills) => skills,
      OptionallyFetched::Unfetched => panic!("active skills not fetched for member {}", self.id),
    };

    PartyMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_lv_a: skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: skills[0].as_ref().map_or(0, |skill| skill.value as i64),
      ac_skill_lv_b: skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: skills[1].as_ref().map_or(0, |skill| skill.value as i64),
      ac_skill_lv_c: skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: skills[2].as_ref().map_or(0, |skill| skill.value as i64),
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

  pub fn to_battle_member(&self, form: &PartyForm) -> BattleMember {
    let skills = match &self.active_skills {
      OptionallyFetched::Fetched(skills) => skills,
      OptionallyFetched::Unfetched => panic!("active skills not fetched for member {}", self.id),
    };

    BattleMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: skills[2].as_ref().map_or(0, |skill| skill.value),
      hp: self.stats.hp.interpolate(self.level()),
      magicattack: self.stats.attack_magic.interpolate(self.level()),
      defense: self.stats.defense.interpolate(self.level()),
      magicdefence: self.stats.defense_magic.interpolate(self.level()),
      agility: self.stats.agility.interpolate(self.level()),
      dexterity: self.stats.dexterity.interpolate(self.level()),
      luck: self.stats.luck.interpolate(self.level()),
      limit_break: self.promotion_level,
      character_id: self.prototype.character_id,
      passiveskill: 210201, // self.prototype.passive_skill.as_ref().map_or(0, |skill| skill.id),
      specialattack: form.specialskill.special_skill_id as i64, // self.prototype.special_attack.as_ref().map_or(0, |skill| skill.id),
      resist_state: 210201,                                     // self.prototype.resistance_group.id,
      resist_attr: 150000000,
      attack: self.stats.attack.interpolate(self.level()),
      ex_flg: 0,
      is_undead: 0,
      special_skill_lv: 1,
    }
  }

  pub fn to_dungeon_battle_member(&self) -> DungeonBattleMember {
    let skills = match &self.active_skills {
      OptionallyFetched::Fetched(skills) => skills,
      OptionallyFetched::Unfetched => panic!("active skills not fetched for member {}", self.id),
    };

    DungeonBattleMember {
      id: self.id,
      lv: self.level(),
      exp: self.xp,
      member_id: self.prototype.id,
      ac_skill_id_a: skills[0].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_a: skills[0].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_a: skills[0].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_b: skills[1].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_b: skills[1].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_b: skills[1].as_ref().map_or(0, |skill| skill.value),
      ac_skill_id_c: skills[2].as_ref().map_or(0, |skill| skill.prototype.id),
      ac_skill_lv_c: skills[2].as_ref().map_or(0, |skill| skill.level),
      ac_skill_val_c: skills[2].as_ref().map_or(0, |skill| skill.value),
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
pub struct MemberActiveSkill {
  pub prototype: Arc<ActiveSkillPrototype>,
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

    // Used for debugging only, should always be 1.0 in production.
    const POST_MULTIPLIER: f32 = 20.0;
    if POST_MULTIPLIER != 1.0 {
      static WARNED: std::sync::Once = std::sync::Once::new();
      WARNED.call_once(|| {
        warn!(
          "MinMaxRange::interpolate: POST_MULTIPLIER is not 1.0, got {}",
          POST_MULTIPLIER
        );
      });
    }

    let ratio = (level - 1) as f32 / (MAX_LEVEL - 1) as f32;
    let interpolated = self.min as f32 + (self.max - self.min) as f32 * ratio;
    (interpolated * POST_MULTIPLIER).round() as i32
  }
}

pub struct FetchUserMembers<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserMembers<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select member_id, xp, promotion_level
        from user_members
        where user_id = $1
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId) -> anyhow::Result<Vec<Member>> {
    let rows = self.executor.client().query(&self.statement, &[&user_id]).await?;
    Ok(rows.into_iter().map(materialize_member_row).collect::<Vec<_>>())
  }
}

pub struct FetchUserMembersIn<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserMembersIn<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select member_id, xp, promotion_level
        from user_members
        where user_id = $1 and member_id = any($2)
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId, ids: &[i64]) -> anyhow::Result<Vec<Member>> {
    let rows = self.executor.client().query(&self.statement, &[&user_id, &ids]).await?;
    Ok(rows.into_iter().map(materialize_member_row).collect::<Vec<_>>())
  }
}

pub fn materialize_member_row(row: Row) -> Member {
  let member_id: i64 = row.get("member_id");
  let xp: i32 = row.get("xp");
  let promotion_level: i32 = row.get("promotion_level");

  let prototype = MemberPrototype::load_from_id(member_id);
  materialize_member_row_impl(member_id, xp, promotion_level, prototype)
}

pub fn materialize_member_row_impl(
  member_id: i64,
  xp: i32,
  promotion_level: i32,
  prototype: Arc<MemberPrototype>,
) -> Member {
  Member {
    id: prototype.id as i32,
    prototype: prototype.clone(),
    xp,
    promotion_level,
    active_skills: OptionallyFetched::Unfetched,
    stats: prototype.stats.clone(),
    main_strength: MemberStrength::default(),
    sub_strength: MemberStrength::default(),
    sub_strength_bonus: MemberStrength::default(),
    fame_stats: MemberFameStats::default(),
    skill_pa_fame_list: vec![],
  }
}

pub struct FetchUserParties<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserParties<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select
          up.party_id,
          up.name,
          up.assist_id,
          up.trait_id,
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
        where up.user_id = $1
        order by up.party_id, upf.form_id
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId) -> anyhow::Result<Vec<Party>> {
    let rows = self.executor.client().query(&self.statement, &[&user_id]).await?;
    Ok(materialize_party_rows(rows))
  }
}

pub struct FetchUserParty<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserParty<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select
          up.party_id,
          up.name,
          up.assist_id,
          up.trait_id,
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
        order by up.party_id, upf.form_id
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId, party_id: i64) -> anyhow::Result<Party> {
    let rows = self
      .executor
      .client()
      .query(&self.statement, &[&user_id, &party_id])
      .await?;
    Ok(materialize_party_rows(rows).into_iter().next().unwrap())
  }
}

pub fn materialize_party_rows(rows: Vec<Row>) -> Vec<Party> {
  rows
    .into_iter()
    .chunk_by(|row| {
      let party_id: i64 = row.get("party_id");
      party_id as i32
    })
    .into_iter()
    .map(|(party_id, forms)| {
      let forms = forms.collect::<Vec<_>>();
      // Hack because of JOIN, I guess
      let form_one = &forms[0];
      let assist_id: i64 = form_one.get("assist_id");
      let trait_id: i64 = form_one.get("trait_id");

      let forms = forms
        .into_iter()
        .map(|row| {
          let party_name: String = row.get("name");
          let form_id: i64 = row.get("form_id");
          let main_member_id: i64 = row.get("main_member_id");
          let sub1_member_id: i64 = row.get("sub1_member_id");
          let sub2_member_id: i64 = row.get("sub2_member_id");
          let weapon_id: i64 = row.get("weapon_id");
          let accessory_id: i64 = row.get("accessory_id");
          let special_skill_id: i64 = row.get("special_skill_id");

          PartyForm {
            id: form_id as i32,
            form_no: form_id as i32,
            party_no: party_id,
            main: main_member_id as i32,
            sub1: sub1_member_id as i32,
            sub2: sub2_member_id as i32,
            weapon: weapon_id,
            acc: accessory_id,
            name: party_name,
            strength: 12300,
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

      Party {
        party_forms: forms,
        party_no: party_id,
        assist: assist_id,
        sub_assists: vec![],
        party_passive_skill: PartyPassiveSkillInfo {
          skill_id: trait_id,
          user_member_id: 0,
        },
      }
    })
    .collect::<Vec<_>>()
}

pub struct FetchUserMemberSkillsIn<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserMemberSkillsIn<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select member_id, skill_id, level
        from user_member_skills
        where user_id = $1 and member_id = any($2)
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId, members: &mut [&mut Member]) -> anyhow::Result<()> {
    let member_ids: Vec<i64> = members.iter().map(|member| member.prototype.id).collect();
    let rows = self
      .executor
      .client()
      .query(&self.statement, &[&user_id, &member_ids])
      .await?;

    let mut skills = HashMap::new();
    for row in rows {
      let member_id: i64 = row.get("member_id");
      let skill_id: i64 = row.get("skill_id");
      let level: i32 = row.get("level");
      skills.entry(member_id).or_insert_with(Vec::new).push((skill_id, level));
    }

    for member in members.iter_mut() {
      if let Some(member_skills) = skills.get(&member.prototype.id) {
        let mut active_skills_array: [Option<MemberActiveSkill>; 3] = [None, None, None];
        for (skill_id, level) in member_skills {
          for (index, prototype) in member.prototype.active_skills.iter().flatten().enumerate() {
            if prototype.id == *skill_id {
              active_skills_array[index] = Some(MemberActiveSkill {
                prototype: prototype.clone(),
                level: *level,
                value: prototype.value.max, // TODO: Calculate actual value based on level
              });
            }
          }
        }
        member.active_skills = OptionallyFetched::Fetched(active_skills_array);
      }
    }

    Ok(())
  }
}
