use crate::api::master_all::get_master_manager;
use crate::api::{
  CharacterParameter, MemberFameStats, MemberParameterWire, MemberStats, RemoteData, RemoteDataCommand,
  RemoteDataItemType, SpSkill,
};
use crate::level::get_member_level_calculator;
use crate::member::{Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use std::sync::Arc;
use tracing::info;

pub async fn get_login_remote_data(state: &AppState, session: &Session) -> Vec<RemoteData> {
  let masters = get_master_manager();
  let characters = masters.get_master("character");
  // let members = masters.get_master("member");
  let costumes = masters.get_master("costume");
  let backgrounds = masters.get_master("background");

  let client = state
    .pool
    .get()
    .await
    .context("failed to get database connection")
    .unwrap();
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
    .context("failed to prepare statement").unwrap();
  let rows = client
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")
    .unwrap();
  info!(?rows, "get friend info query executed");

  let members = rows
    .iter()
    .enumerate()
    .map(|(index, row)| {
      let member_id: i64 = row.get(0);
      let xp: i32 = row.get(1);
      let promotion_level: i32 = row.get(2);
      // let active_skills: Value = row.get(3);
      let prototype = MemberPrototype::load_from_id(member_id);

      Member {
        id: index as i32 + 1,
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
      .to_member_parameter_wire()
    })
    .collect::<Vec<_>>();

  let characters = characters
    .iter()
    .enumerate()
    .map(|(index, character)| {
      AddCharacter::new(
        index as i32,
        CharacterParameter {
          id: index as i64,
          character_id: character.get("id").unwrap().as_str().unwrap().parse().unwrap(),
          rank: 1,
          rank_progress: 5,
          sp_skill: vec![SpSkill {
            group_id: 10000,
            id: 100001,
            lv: 2,
            is_trial: false,
          }],
          character_enhance_stage_id_list: vec![0, 0, 0, 0],
          character_piece_board_stage_id_list: vec![100001001, 100002002, 100003003, 100004004],
          is_trial: false,
        },
      )
      .into_remote_data()
    })
    .collect::<Vec<_>>();

  let costumes = costumes
    .iter()
    .enumerate()
    .map(|(index, costume)| {
      AddMemberCostume::new(
        index as i32,
        costume.get("id").unwrap().as_str().unwrap().parse().unwrap(),
      )
      .into_remote_data()
    })
    .collect::<Vec<_>>();

  let backgrounds = backgrounds
    .iter()
    .enumerate()
    .map(|(index, background)| {
      AddMemberBackground::new(
        index as i32,
        background.get("id").unwrap().as_str().unwrap().parse().unwrap(),
      )
      .into_remote_data()
    })
    .collect::<Vec<_>>();

  let members = members
    .into_iter()
    .enumerate()
    .map(|(index, member)| AddMember::new(member, "front").into_remote_data())
    .collect::<Vec<_>>();

  #[cfg_attr(rustfmt, rustfmt::skip)]
  vec![
    ClearUserParams.into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::Money, 85720).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::RealMoney, 100000).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::RealMoneyFree, 1000000).into_remote_data(),
    // AddMember::new(MemberPrototype::load_from_id(1001100).create_member_wire(), "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 11, lv: 4, exp: 150, member_id: 1001100, ac_skill_id_a: 21503639, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 0, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 130, hp: 277, magicattack: 31, defense: 24, magicdefence: 22, agility: 72, dexterity: 78, luck: 88, limit_break: 0, character_id: 100, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 32, waiting_room: 0, main_strength: 444, main_strength_for_fame_quest: 444, sub_strength: 106, sub_strength_for_fame_quest: 106, sub_strength_bonus: 141, sub_strength_bonus_for_fame_quest: 141, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 8, lv: 1, exp: 0, member_id: 1002102, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 20, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 130, hp: 257, magicattack: 28, defense: 21, magicdefence: 20, agility: 73, dexterity: 79, luck: 87, limit_break: 0, character_id: 100, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 28, waiting_room: 0, main_strength: 409, main_strength_for_fame_quest: 409, sub_strength: 95, sub_strength_for_fame_quest: 95, sub_strength_bonus: 127, sub_strength_bonus_for_fame_quest: 127, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 12, lv: 4, exp: 150, member_id: 1011100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 170, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 152, hp: 285, magicattack: 37, defense: 25, magicdefence: 27, agility: 66, dexterity: 76, luck: 10, limit_break: 0, character_id: 101, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 33, waiting_room: 0, main_strength: 477, main_strength_for_fame_quest: 477, sub_strength: 116, sub_strength_for_fame_quest: 116, sub_strength_bonus: 154, sub_strength_bonus_for_fame_quest: 154, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 13, lv: 1, exp: 0, member_id: 1021100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 20, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 152, hp: 202, magicattack: 30, defense: 18, magicdefence: 21, agility: 68, dexterity: 71, luck: 72, limit_break: 0, character_id: 102, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 26, waiting_room: 0, main_strength: 387, main_strength_for_fame_quest: 387, sub_strength: 89, sub_strength_for_fame_quest: 89, sub_strength_bonus: 118, sub_strength_bonus_for_fame_quest: 118, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 14, lv: 1, exp: 0, member_id: 1031100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 127, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 150, hp: 281, magicattack: 24, defense: 24, magicdefence: 24, agility: 68, dexterity: 10, luck: 64, limit_break: 0, character_id: 103, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 29, waiting_room: 0, main_strength: 419, main_strength_for_fame_quest: 419, sub_strength: 98, sub_strength_for_fame_quest: 98, sub_strength_bonus: 131, sub_strength_bonus_for_fame_quest: 131, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 2, lv: 1, exp: 0, member_id: 1034100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 102, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 130, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 150, hp: 330, magicattack: 29, defense: 28, magicdefence: 28, agility: 68, dexterity: 10, luck: 64, limit_break: 0, character_id: 103, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 35, waiting_room: 0, main_strength: 481, main_strength_for_fame_quest: 481, sub_strength: 117, sub_strength_for_fame_quest: 117, sub_strength_bonus: 156, sub_strength_bonus_for_fame_quest: 156, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 15, lv: 1, exp: 0, member_id: 1061100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 154, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 122, hp: 214, magicattack: 30, defense: 19, magicdefence: 22, agility: 69, dexterity: 68, luck: 67, limit_break: 0, character_id: 106, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 25, waiting_room: 0, main_strength: 391, main_strength_for_fame_quest: 391, sub_strength: 90, sub_strength_for_fame_quest: 90, sub_strength_bonus: 120, sub_strength_bonus_for_fame_quest: 120, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 1, lv: 1, exp: 0, member_id: 1063113, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 122, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 138, hp: 237, magicattack: 34, defense: 21, magicdefence: 25, agility: 70, dexterity: 69, luck: 66, limit_break: 0, character_id: 106, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 28, waiting_room: 0, main_strength: 427, main_strength_for_fame_quest: 427, sub_strength: 101, sub_strength_for_fame_quest: 101, sub_strength_bonus: 134, sub_strength_bonus_for_fame_quest: 134, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 10, lv: 3, exp: 150, member_id: 1064217, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 128, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 173, hp: 270, magicattack: 41, defense: 25, magicdefence: 29, agility: 69, dexterity: 67, luck: 68, limit_break: 0, character_id: 106, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 33, waiting_room: 0, main_strength: 487, main_strength_for_fame_quest: 487, sub_strength: 119, sub_strength_for_fame_quest: 119, sub_strength_bonus: 158, sub_strength_bonus_for_fame_quest: 158, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 4, lv: 1, exp: 0, member_id: 1083110, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 165, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 165, hp: 292, magicattack: 34, defense: 25, magicdefence: 25, agility: 61, dexterity: 66, luck: 63, limit_break: 0, character_id: 108, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 29, waiting_room: 0, main_strength: 456, main_strength_for_fame_quest: 456, sub_strength: 109, sub_strength_for_fame_quest: 109, sub_strength_bonus: 146, sub_strength_bonus_for_fame_quest: 146, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 6, lv: 1, exp: 0, member_id: 1093100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 170, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 105, hp: 266, magicattack: 32, defense: 22, magicdefence: 24, agility: 68, dexterity: 67, luck: 65, limit_break: 0, character_id: 109, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 30, waiting_room: 0, main_strength: 438, main_strength_for_fame_quest: 438, sub_strength: 104, sub_strength_for_fame_quest: 104, sub_strength_bonus: 139, sub_strength_bonus_for_fame_quest: 139, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 17, lv: 1, exp: 0, member_id: 1102102, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 186, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 144, hp: 207, magicattack: 32, defense: 18, magicdefence: 23, agility: 68, dexterity: 70, luck: 70, limit_break: 0, character_id: 110, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 26, waiting_room: 0, main_strength: 397, main_strength_for_fame_quest: 397, sub_strength: 92, sub_strength_for_fame_quest: 92, sub_strength_bonus: 122, sub_strength_bonus_for_fame_quest: 122, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 5, lv: 1, exp: 0, member_id: 1122100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 139, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 128, hp: 282, magicattack: 24, defense: 23, magicdefence: 19, agility: 71, dexterity: 70, luck: 62, limit_break: 0, character_id: 112, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 32, waiting_room: 0, main_strength: 419, main_strength_for_fame_quest: 419, sub_strength: 98, sub_strength_for_fame_quest: 98, sub_strength_bonus: 131, sub_strength_bonus_for_fame_quest: 131, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 7, lv: 1, exp: 0, member_id: 1132100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 154, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 122, hp: 247, magicattack: 31, defense: 19, magicdefence: 22, agility: 69, dexterity: 73, luck: 73, limit_break: 0, character_id: 113, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 25, waiting_room: 0, main_strength: 405, main_strength_for_fame_quest: 405, sub_strength: 94, sub_strength_for_fame_quest: 94, sub_strength_bonus: 126, sub_strength_bonus_for_fame_quest: 126, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 18, lv: 1, exp: 0, member_id: 1143127, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 138, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 138, hp: 301, magicattack: 29, defense: 26, magicdefence: 24, agility: 71, dexterity: 72, luck: 71, limit_break: 0, character_id: 114, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 31, waiting_room: 0, main_strength: 450, main_strength_for_fame_quest: 450, sub_strength: 108, sub_strength_for_fame_quest: 108, sub_strength_bonus: 144, sub_strength_bonus_for_fame_quest: 144, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 3, lv: 1, exp: 0, member_id: 1152102, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 170, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 152, hp: 247, magicattack: 31, defense: 21, magicdefence: 23, agility: 71, dexterity: 74, luck: 70, limit_break: 0, character_id: 115, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 27, waiting_room: 0, main_strength: 416, main_strength_for_fame_quest: 416, sub_strength: 97, sub_strength_for_fame_quest: 97, sub_strength_bonus: 130, sub_strength_bonus_for_fame_quest: 130, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 19, lv: 1, exp: 0, member_id: 1162100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 110, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 12, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 145, hp: 257, magicattack: 27, defense: 22, magicdefence: 20, agility: 73, dexterity: 74, luck: 77, limit_break: 0, character_id: 116, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 27, waiting_room: 0, main_strength: 404, main_strength_for_fame_quest: 404, sub_strength: 94, sub_strength_for_fame_quest: 94, sub_strength_bonus: 125, sub_strength_bonus_for_fame_quest: 125, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 16, lv: 1, exp: 0, member_id: 1192102, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 100, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 170, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 152, hp: 242, magicattack: 32, defense: 20, magicdefence: 23, agility: 66, dexterity: 72, luck: 72, limit_break: 0, character_id: 119, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 25, waiting_room: 0, main_strength: 410, main_strength_for_fame_quest: 410, sub_strength: 96, sub_strength_for_fame_quest: 96, sub_strength_bonus: 128, sub_strength_bonus_for_fame_quest: 128, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 9, lv: 1, exp: 0, member_id: 1282100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 93, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 128, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 122, hp: 239, magicattack: 32, defense: 24, magicdefence: 24, agility: 71, dexterity: 74, luck: 72, limit_break: 0, character_id: 128, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 25, waiting_room: 0, main_strength: 416, main_strength_for_fame_quest: 416, sub_strength: 97, sub_strength_for_fame_quest: 97, sub_strength_bonus: 130, sub_strength_bonus_for_fame_quest: 130, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 1, lv: 0, exp: 0, member_id: 1192102, ac_skill_id_a: 0, ac_skill_lv_a: 0, ac_skill_val_a: 0, ac_skill_id_b: 0, ac_skill_lv_b: 0, ac_skill_val_b: 0, ac_skill_id_c: 0, ac_skill_lv_c: 0, ac_skill_val_c: 0, hp: 0, magicattack: 0, defense: 0, magicdefence: 0, agility: 0, dexterity: 0, luck: 0, limit_break: 0, character_id: 0, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 0, waiting_room: 0, main_strength: 0, main_strength_for_fame_quest: 0, sub_strength: 0, sub_strength_for_fame_quest: 0, sub_strength_bonus: 0, sub_strength_bonus_for_fame_quest: 0, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "back").into_remote_data(),
    // AddMember::new(MemberParameterWire { id: 111, lv: 1, exp: 0, member_id: 1282100, ac_skill_id_a: 0, ac_skill_lv_a: 1, ac_skill_val_a: 93, ac_skill_id_b: 0, ac_skill_lv_b: 1, ac_skill_val_b: 128, ac_skill_id_c: 0, ac_skill_lv_c: 1, ac_skill_val_c: 122, hp: 239, magicattack: 32, defense: 24, magicdefence: 24, agility: 71, dexterity: 74, luck: 72, limit_break: 0, character_id: 128, passiveskill: 0, specialattack: 0, resist_state: 0, resist_attr: 0, attack: 25, waiting_room: 0, main_strength: 416, main_strength_for_fame_quest: 416, sub_strength: 97, sub_strength_for_fame_quest: 97, sub_strength_bonus: 130, sub_strength_bonus_for_fame_quest: 130, fame_hp_rank: 0, fame_attack_rank: 0, fame_defense_rank: 0, fame_magicattack_rank: 0, fame_magicdefence_rank: 0, skill_pa_fame_list: vec![] }, "front").into_remote_data(),
    AddItem::new(RemoteDataItemType::SkipTicket, 0, 1, 800).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::Stamina, 419).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::Exp, 10).into_remote_data(),
    // AddCharacter::new(8, CharacterParameter { id: 5335218194, character_id: 100, rank: 1, rank_progress: 4, sp_skill: vec![SpSkill { group_id: 10000, id: 100001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(10, CharacterParameter { id: 5335220194, character_id: 101, rank: 1, rank_progress: 4, sp_skill: vec![SpSkill { group_id: 10100, id: 101001, lv: 1, is_trial: false }, SpSkill { group_id: 10102, id: 101021, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(11, CharacterParameter { id: 5335221194, character_id: 102, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 10200, id: 102001, lv: 1, is_trial: false }, SpSkill { group_id: 10202, id: 102021, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(2, CharacterParameter { id: 5335212194, character_id: 103, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 10300, id: 103001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(1, CharacterParameter { id: 5335211194, character_id: 106, rank: 1, rank_progress: 4, sp_skill: vec![SpSkill { group_id: 10600, id: 106001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(4, CharacterParameter { id: 5335214194, character_id: 108, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 10800, id: 108001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(6, CharacterParameter { id: 5335216194, character_id: 109, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 10900, id: 109001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(13, CharacterParameter { id: 5335293194, character_id: 110, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11000, id: 110001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(5, CharacterParameter { id: 5335215194, character_id: 112, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11200, id: 112001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(7, CharacterParameter { id: 5335217194, character_id: 113, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11300, id: 113001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(14, CharacterParameter { id: 5335294194, character_id: 114, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11400, id: 114001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(3, CharacterParameter { id: 5335213194, character_id: 115, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11500, id: 115001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(15, CharacterParameter { id: 5335295194, character_id: 116, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11600, id: 116001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(12, CharacterParameter { id: 5335292194, character_id: 119, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 11900, id: 119001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddCharacter::new(9, CharacterParameter { id: 5335219194, character_id: 128, rank: 1, rank_progress: 0, sp_skill: vec![SpSkill { group_id: 12800, id: 128001, lv: 1, is_trial: false }], character_enhance_stage_id_list: vec![0, 0, 0, 0], character_piece_board_stage_id_list: vec![], is_trial: false }).into_remote_data(),
    // AddMemberCostume::new(10, 1004100).into_remote_data(),
    // AddMemberCostume::new(93, 1004100).into_remote_data(),
    // AddMemberCostume::new(13, 1014100).into_remote_data(),
    // AddMemberCostume::new(92, 1014100).into_remote_data(),
    // AddMemberCostume::new(14, 1024100).into_remote_data(),
    // AddMemberCostume::new(91, 1024100).into_remote_data(),
    // AddMemberCostume::new(3, 1034100).into_remote_data(),
    // AddMemberCostume::new(90, 1034100).into_remote_data(),
    // AddMemberCostume::new(89, 1044100).into_remote_data(),
    // AddMemberCostume::new(88, 1054100).into_remote_data(),
    // AddMemberCostume::new(2, 1063113).into_remote_data(),
    // AddMemberCostume::new(1, 1064100).into_remote_data(),
    // AddMemberCostume::new(87, 1064100).into_remote_data(),
    // AddMemberCostume::new(12, 1064217).into_remote_data(),
    // AddMemberCostume::new(86, 1074100).into_remote_data(),
    // AddMemberCostume::new(6, 1083110).into_remote_data(),
    // AddMemberCostume::new(5, 1084100).into_remote_data(),
    // AddMemberCostume::new(15, 1084100).into_remote_data(),
    // AddMemberCostume::new(8, 1094100).into_remote_data(),
    // AddMemberCostume::new(85, 1094100).into_remote_data(),
    // AddMemberCostume::new(84, 1104100).into_remote_data(),
    // AddMemberCostume::new(83, 1114100).into_remote_data(),
    // AddMemberCostume::new(7, 1124100).into_remote_data(),
    // AddMemberCostume::new(82, 1124100).into_remote_data(),
    // AddMemberCostume::new(9, 1134100).into_remote_data(),
    // AddMemberCostume::new(81, 1134100).into_remote_data(),
    // AddMemberCostume::new(94, 1143127).into_remote_data(),
    // AddMemberCostume::new(80, 1144100).into_remote_data(),
    // AddMemberCostume::new(4, 1154100).into_remote_data(),
    // AddMemberCostume::new(79, 1154100).into_remote_data(),
    // AddMemberCostume::new(78, 1164100).into_remote_data(),
    // AddMemberCostume::new(77, 1174100).into_remote_data(),
    // AddMemberCostume::new(76, 1184100).into_remote_data(),
    // AddMemberCostume::new(75, 1194100).into_remote_data(),
    // AddMemberCostume::new(74, 1209100).into_remote_data(),
    // AddMemberCostume::new(73, 1219100).into_remote_data(),
    // AddMemberCostume::new(72, 1229100).into_remote_data(),
    // AddMemberCostume::new(71, 1239100).into_remote_data(),
    // AddMemberCostume::new(70, 1249100).into_remote_data(),
    // AddMemberCostume::new(69, 1259100).into_remote_data(),
    // AddMemberCostume::new(68, 1264100).into_remote_data(),
    // AddMemberCostume::new(67, 1279100).into_remote_data(),
    // AddMemberCostume::new(11, 1284100).into_remote_data(),
    // AddMemberCostume::new(66, 1284100).into_remote_data(),
    // AddMemberCostume::new(65, 1299100).into_remote_data(),
    // AddMemberCostume::new(64, 1309100).into_remote_data(),
    // AddMemberCostume::new(63, 1319100).into_remote_data(),
    // AddMemberCostume::new(62, 1329100).into_remote_data(),
    // AddMemberCostume::new(61, 1339100).into_remote_data(),
    // AddMemberCostume::new(60, 1349100).into_remote_data(),
    // AddMemberCostume::new(59, 1369100).into_remote_data(),
    // AddMemberCostume::new(58, 1429100).into_remote_data(),
    // AddMemberCostume::new(57, 1474132).into_remote_data(),
    // AddMemberCostume::new(56, 1504132).into_remote_data(),
    // AddMemberCostume::new(55, 1514100).into_remote_data(),
    // AddMemberCostume::new(54, 1539132).into_remote_data(),
    // AddMemberCostume::new(53, 1584147).into_remote_data(),
    // AddMemberCostume::new(52, 1599147).into_remote_data(),
    // AddMemberCostume::new(51, 1604147).into_remote_data(),
    // AddMemberCostume::new(50, 1619147).into_remote_data(),
    // AddMemberCostume::new(49, 1634161).into_remote_data(),
    // AddMemberCostume::new(48, 1644161).into_remote_data(),
    // AddMemberCostume::new(47, 1654161).into_remote_data(),
    // AddMemberCostume::new(46, 1694100).into_remote_data(),
    // AddMemberCostume::new(45, 1814100).into_remote_data(),
    // AddMemberCostume::new(44, 1834189).into_remote_data(),
    // AddMemberCostume::new(43, 1844189).into_remote_data(),
    // AddMemberCostume::new(42, 1854189).into_remote_data(),
    // AddMemberCostume::new(41, 1864189).into_remote_data(),
    // AddMemberCostume::new(40, 1924195).into_remote_data(),
    // AddMemberCostume::new(39, 1934195).into_remote_data(),
    // AddMemberCostume::new(38, 1944195).into_remote_data(),
    // AddMemberCostume::new(37, 1954203).into_remote_data(),
    // AddMemberCostume::new(36, 1964203).into_remote_data(),
    // AddMemberCostume::new(35, 1974203).into_remote_data(),
    // AddMemberCostume::new(34, 1984210).into_remote_data(),
    // AddMemberCostume::new(33, 1994210).into_remote_data(),
    // AddMemberCostume::new(32, 2004210).into_remote_data(),
    // AddMemberCostume::new(31, 2054220).into_remote_data(),
    // AddMemberCostume::new(30, 2064220).into_remote_data(),
    // AddMemberCostume::new(29, 2074220).into_remote_data(),
    // AddMemberCostume::new(28, 2084220).into_remote_data(),
    // AddMemberCostume::new(27, 2094100).into_remote_data(),
    // AddMemberCostume::new(26, 2114225).into_remote_data(),
    // AddMemberCostume::new(25, 2124225).into_remote_data(),
    // AddMemberCostume::new(24, 2134225).into_remote_data(),
    // AddMemberCostume::new(23, 2144225).into_remote_data(),
    // AddMemberCostume::new(22, 2174236).into_remote_data(),
    // AddMemberCostume::new(21, 2184236).into_remote_data(),
    // AddMemberCostume::new(20, 2194236).into_remote_data(),
    // AddMemberCostume::new(19, 2204236).into_remote_data(),
    // AddMemberCostume::new(18, 2234241).into_remote_data(),
    // AddMemberCostume::new(17, 2244241).into_remote_data(),
    // AddMemberCostume::new(16, 2254241).into_remote_data(),
    AddItem::new(RemoteDataItemType::MaterialWA, 2, 1100, 3).into_remote_data(),
    AddItem::new(RemoteDataItemType::MaterialWA, 1, 5001, 4).into_remote_data(),
    AddItem::new(RemoteDataItemType::MaterialLimit, 1, 151, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::PowerPotion, 2, 1, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::PowerPotion, 3, 2, 2).into_remote_data(),
    AddItem::new(RemoteDataItemType::PowerPotion, 1, 3, 2).into_remote_data(),
    AddItem::new(RemoteDataItemType::Ticket, 0, 17, 2).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::Level, 35).into_remote_data(),
    // AddMemberBackground::new(5, 1010).into_remote_data(),
    // AddMemberBackground::new(4, 1011).into_remote_data(),
    // AddMemberBackground::new(3, 1012).into_remote_data(),
    // AddMemberBackground::new(2, 1013).into_remote_data(),
    // AddMemberBackground::new(1, 1090).into_remote_data(),
    // AddMemberBackground::new(6, 1180).into_remote_data(),
    AddItem::new(RemoteDataItemType::BossTicket, 0, 230831, 3).into_remote_data(),
    AddItem::new(RemoteDataItemType::SlayerMedal, 0, 230731, 0).into_remote_data(),
    // TODO: { "cmd": 4, "item_type": 34, "item_id": 2, "item_num": 3, "uniqid": 0, "lv": 0, "tag": "-" }
    // TODO: { "cmd": 4, "item_type": 34, "item_id": 1, "item_num": 0, "uniqid": 0, "lv": 0, "tag": "-" }
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 100, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 103, 20).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 106, 25).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 108, 5).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 109, 5).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 110, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 112, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 113, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 114, 5).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 115, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 116, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 119, 2).into_remote_data(),
    AddItem::new(RemoteDataItemType::CharacterPiece, 0, 128, 1).into_remote_data(),
    AddSingletonItem::new(RemoteDataItemType::FameRank, 1).into_remote_data(),
    AddItem::new(RemoteDataItemType::ExchangeMedal, 0, 1001, 100100).into_remote_data(),
    AddItem::new(RemoteDataItemType::ExchangeMedal, 0, 1011, 101100).into_remote_data(),
    AddItem::new(RemoteDataItemType::ExchangeMedal, 0, 1012, 101200).into_remote_data(),
  ]
    .into_iter()
    .chain(members)
    .chain(characters)
    .chain(costumes)
    .chain(backgrounds)
    .collect::<Vec<_>>()
}

pub trait IntoRemoteData {
  fn into_remote_data(self) -> RemoteData;
}

pub struct AddMemberBackground {
  pub unique_id: i32,
  pub background_id: i64,
}

impl AddMemberBackground {
  pub fn new(unique_id: i32, background_id: i64) -> Self {
    Self {
      unique_id,
      background_id,
    }
  }
}

impl IntoRemoteData for AddMemberBackground {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: RemoteDataItemType::MemberBackground.into(),
      item_id: self.background_id,
      item_num: 1,
      uniqid: self.unique_id,
      lv: 0,
      tag: String::from("-"),
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}

pub struct AddMemberCostume {
  pub unique_id: i32,
  pub costume_id: i64,
}

impl AddMemberCostume {
  pub fn new(unique_id: i32, costume_id: i64) -> Self {
    Self { unique_id, costume_id }
  }
}

impl IntoRemoteData for AddMemberCostume {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: RemoteDataItemType::MemberCostume.into(),
      item_id: self.costume_id,
      item_num: 1,
      uniqid: self.unique_id,
      lv: 0,
      tag: String::from("useflag:0"),
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}

pub struct AddItem {
  pub item_type: RemoteDataItemType,
  pub unique_id: i32,
  pub item_id: i64,
  pub item_num: i32,
}

impl AddItem {
  pub fn new(item_type: RemoteDataItemType, unique_id: i32, item_id: i64, item_num: i32) -> Self {
    Self {
      item_type,
      unique_id,
      item_id,
      item_num,
    }
  }
}

impl IntoRemoteData for AddItem {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: self.item_type.into(),
      item_id: self.item_id,
      item_num: self.item_num,
      uniqid: self.unique_id,
      lv: 0,
      tag: String::from("-"),
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}

/// Singleton items have no [item_id] and [uniqid] set.
pub struct AddSingletonItem {
  pub item_type: RemoteDataItemType,
  pub item_num: i32,
}

impl AddSingletonItem {
  pub fn new(item_type: RemoteDataItemType, item_num: i32) -> Self {
    Self { item_type, item_num }
  }
}

impl IntoRemoteData for AddSingletonItem {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: self.item_type.into(),
      item_id: 0,
      item_num: self.item_num,
      uniqid: 0,
      lv: 0,
      tag: String::from("-"),
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}

pub struct AddMember {
  pub member_parameter: MemberParameterWire,
  pub tag: String,
}

impl AddMember {
  pub fn new(member_parameter: MemberParameterWire, tag: &str) -> Self {
    Self {
      member_parameter,
      tag: tag.to_owned(),
    }
  }
}

impl IntoRemoteData for AddMember {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: RemoteDataItemType::Member.into(),
      item_id: self.member_parameter.member_id,
      item_num: 1,
      uniqid: self.member_parameter.id,
      lv: self.member_parameter.lv,
      tag: self.tag,
      member_parameter: Some(self.member_parameter),
      character_parameter: None,
      is_trial: None,
    }
  }
}

/// Unlocks character stories, also needed for [AddMember].
pub struct AddCharacter {
  pub unique_id: i32,
  pub character_parameter: CharacterParameter,
}

impl AddCharacter {
  pub fn new(unique_id: i32, character_parameter: CharacterParameter) -> Self {
    Self {
      unique_id,
      character_parameter,
    }
  }
}

impl IntoRemoteData for AddCharacter {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamAdd as i32,
      uid: None,
      item_type: RemoteDataItemType::Character.into(),
      item_id: self.character_parameter.character_id,
      item_num: 1,
      uniqid: self.unique_id,
      lv: 1,
      tag: String::from(""),
      member_parameter: None,
      character_parameter: Some(self.character_parameter),
      is_trial: None,
    }
  }
}

pub struct ClearUserParams;

impl IntoRemoteData for ClearUserParams {
  fn into_remote_data(self) -> RemoteData {
    RemoteData {
      cmd: RemoteDataCommand::UserParamClear as i32,
      uid: None,
      item_type: 0,
      item_id: 0,
      item_num: 0,
      uniqid: 0,
      lv: 0,
      tag: String::from("-"),
      member_parameter: None,
      character_parameter: None,
      is_trial: None,
    }
  }
}
