use std::sync::LazyLock;

use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct MasterList {
  pub masterversion: String,
  pub masterarray: Vec<MasterListItem>,
}

impl CallCustom for MasterList {}

#[derive(Clone, Debug, Serialize)]
pub struct MasterListItem {
  pub master_key: String,
  pub size: u32,
  pub checkkey: String,
}

impl MasterListItem {
  pub fn new(master_key: String, size: u32, checkkey: String) -> Self {
    Self {
      master_key,
      size,
      checkkey,
    }
  }
}

#[rustfmt::skip]
pub static MASTER_LIST: LazyLock<Vec<MasterListItem>> = LazyLock::new(|| {
  vec![
    MasterListItem::new("get_member_list".to_owned(), 11956, "16c5aa6ac594b9c99eb56f4c85282d5c".to_owned()),
    MasterListItem::new("story_unique".to_owned(), 1648, "61df8578e16a9343ffcfb6ea0d550eaf".to_owned()),
    MasterListItem::new("mission".to_owned(), 39512, "240e2829bc6fb97ac480a3c237e7a709".to_owned()),
    MasterListItem::new("text".to_owned(), 878704, "76f2c41fcd835701a1985ecf367dfaac".to_owned()),
    MasterListItem::new("shop_continuation".to_owned(), 332, "80f679063f7065e510258d26b933658f".to_owned()),
    MasterListItem::new("roulette_loginbonus".to_owned(), 1040, "27eeee7f864c1898c325384c3c977154".to_owned()),
    MasterListItem::new("quest_mission".to_owned(), 324, "473a78aaa94cc6d391fe78ad08c67841".to_owned()),
    MasterListItem::new("event_emergency_boss".to_owned(), 3104, "0a98006a9f3c7a681b37b8c4ad911fac".to_owned()),
    MasterListItem::new("equip_weapon_details".to_owned(), 33220, "1d6f2cb0dff68214e2c86eacf5256f08".to_owned()),
    MasterListItem::new("event_quest_reward".to_owned(), 413076, "284f9a4f5a78fc9a70d113a0256a9e3c".to_owned()),
    MasterListItem::new("roulette_loginbonus_navi_set".to_owned(), 1024, "fb150dff9a20c1b06b01817a86ae9243".to_owned()),
    MasterListItem::new("navi_interaction_serif".to_owned(), 17276, "d91ffff97bafd0cca4648cba7beba701".to_owned()),
    MasterListItem::new("character_piece_board_stage".to_owned(), 24196, "26f03755163facbcb47c566859ddb18b".to_owned()),
    MasterListItem::new("campaign_box_gacha_infinity".to_owned(), 280, "c4ccf220276a5af9c7ad0fad839d9e1a".to_owned()),
    MasterListItem::new("campaign".to_owned(), 10788, "bd961ab9738b8782bc74c98c6e231401".to_owned()),
    MasterListItem::new("event_boss_count".to_owned(), 2036, "fab8d719c7d06ff6cc9e5e46afb73919".to_owned()),
    MasterListItem::new("skill_sp_details".to_owned(), 71288, "265372af95a44b6da34bdfa565ca7dab".to_owned()),
    MasterListItem::new("animation_loginbonus_navi_set".to_owned(), 464, "47de66dca0736abef9aef839e3411ae3".to_owned()),
    MasterListItem::new("event_mission".to_owned(), 182544, "ff3a29abd469e3f80e7362bda5925a9c".to_owned()),
    MasterListItem::new("scorechallenge_stage".to_owned(), 22528, "88ef75f550ee17bb1b7f3c64667ba6b6".to_owned()),
    MasterListItem::new("gacha_tab".to_owned(), 9284, "d00a8dc70287fc4071c3324ffd18ba28".to_owned()),
    MasterListItem::new("system".to_owned(), 1608, "e1e97780473ebe7a4c83b822bc1087c3".to_owned()),
    MasterListItem::new("equip_accessory".to_owned(), 3256, "1f8b43e46867d00907b1c36f10d44823".to_owned()),
    MasterListItem::new("plan_map".to_owned(), 632, "4c61c5efaf0a05fdf2a0c037eed0b944".to_owned()),
    MasterListItem::new("target_costume_member".to_owned(), 3416, "48fea33e77fb20cc83c0b2fd0ce7f8e7".to_owned()),
    MasterListItem::new("character_piece_board".to_owned(), 1508, "d44492907353bc09bcfb4865f5a04906".to_owned()),
    MasterListItem::new("gallery_group".to_owned(), 524, "80ce03cbb5c143a94cb4251206f7041f".to_owned()),
    MasterListItem::new("event_scorechallenge_stage_reward".to_owned(), 368, "7a5334ad45507c00596bbeaa738c981e".to_owned()),
    MasterListItem::new("bgm".to_owned(), 704, "40cef09d9641e8fa9cbe07bddb817474".to_owned()),
    MasterListItem::new("character_enhance".to_owned(), 43540, "582bbc99841f181a46b433e73bea2de2".to_owned()),
    MasterListItem::new("ad_reward".to_owned(), 328, "e14b7398a1e8de63a123c18b8b43f418".to_owned()),
    MasterListItem::new("character".to_owned(), 3692, "796423b56703b8a0b5112e8c0ea600b0".to_owned()),
    MasterListItem::new("skill_ac".to_owned(), 21588, "195616bad268c2cfa16f7227cf23c73b".to_owned()),
    MasterListItem::new("download_page_character".to_owned(), 300, "6f55339e670df47002ae318359d2c852".to_owned()),
    MasterListItem::new("surprise_quest_stage".to_owned(), 272, "5a8bd2f264b6c9f6c9391410d504c630".to_owned()),
    MasterListItem::new("random_loginbonus_interaction_group".to_owned(), 416, "d7f1b53784f98aa9ca6a797221f4e97f".to_owned()),
    MasterListItem::new("member".to_owned(), 53508, "0801283528c778f9150c8507fb61dc09".to_owned()),
    MasterListItem::new("surprise_quest_score_reward_item".to_owned(), 764, "0352123ff6d8f01877bf7933ad394d10".to_owned()),
    MasterListItem::new("love_item".to_owned(), 1056, "f4be7cfc3ca1e52907b4a0de859a1d05".to_owned()),
    MasterListItem::new("comeback_present".to_owned(), 152, "f9c2f19b23036a9d65c394d8771207f9".to_owned()),
    MasterListItem::new("plan_map_mission".to_owned(), 656, "ef9b7175c7393dd716fdf5bffa42488b".to_owned()),
    MasterListItem::new("skill_pa_field".to_owned(), 364, "04ae85ebbc127c58964f4acd01ace8f5".to_owned()),
    MasterListItem::new("event_emergency_sns_mission".to_owned(), 324, "5534784329683613d714d0539e04fa68".to_owned()),
    MasterListItem::new("surprise_quest_score_reward".to_owned(), 504, "bf2bb6143b6deb33e0c3ff8eca732cfd".to_owned()),
    MasterListItem::new("multi_battle_penalty".to_owned(), 136, "bf6196ec31a36e4fb8bf517296dc56aa".to_owned()),
    MasterListItem::new("gacha_raise".to_owned(), 248, "90512f2d6dc9fe351dd4053b10683e2b".to_owned()),
    MasterListItem::new("quest".to_owned(), 220, "fff95e7a855a949b79a67b8b9fc18530".to_owned()),
    MasterListItem::new("scorechallenge_mission".to_owned(), 2676, "d00001d2a62c7e16cfbd66329a4dd48e".to_owned()),
    MasterListItem::new("gacha_item".to_owned(), 42920, "79cc5bbf78ae52cb10e5d8d833becdca".to_owned()),
    MasterListItem::new("mission_panel_group".to_owned(), 4304, "b53e4fc8bf6e34ec1f5a855a0bd44e8e".to_owned()),
    MasterListItem::new("huntingquest_area".to_owned(), 1472, "b0067cf3ec77c0eb52a9401be1529ffd".to_owned()),
    MasterListItem::new("item".to_owned(), 18016, "92cd8c953db32c9cd1374c2bbbb649c0".to_owned()),
    MasterListItem::new("mainquest_area".to_owned(), 1132, "7d69373597ac84980e7eb9b720ba6835".to_owned()),
    MasterListItem::new("gacha_lot".to_owned(), 15796, "24a245419c0353066655ed6d5c5e9b3f".to_owned()),
    MasterListItem::new("mission_panel_voice".to_owned(), 3712, "0a0ff62a442969cdd58eb3712ca61408".to_owned()),
    MasterListItem::new("expedition_quote".to_owned(), 812, "8ea1b4b8995bd5cdeb403e9092c4ed4a".to_owned()),
    MasterListItem::new("surprise_short_selection".to_owned(), 908, "1fa4b8867b4b30a3bb5e17d356ba403c".to_owned()),
    MasterListItem::new("event_quest_stage".to_owned(), 27692, "b769b223d19a95d4fb91726d177f1f19".to_owned()),
    MasterListItem::new("surprise_short_lot".to_owned(), 260, "62c19de6a7bb5672b40b7f9fa0dc86fb".to_owned()),
    MasterListItem::new("white_list".to_owned(), 800, "1cf59cbf29c7e1565e30d714a1f00b3a".to_owned()),
    MasterListItem::new("character_intimacy_details".to_owned(), 41068, "5dc47ab858768a6fbd497a9b6c557645".to_owned()),
    MasterListItem::new("member_detail_cell_image_pos".to_owned(), 124, "b1084c788e12b77be6f3487738dd7410".to_owned()),
    MasterListItem::new("huntingquest_stage_itemreward".to_owned(), 5060, "fc2f08a94141ca62a3aa4bb7deca3180".to_owned()),
    MasterListItem::new("gallery_still".to_owned(), 868, "45c4571472e168edf5dd9517c49b49dd".to_owned()),
    MasterListItem::new("event_marathon_quest_stage_boss_single".to_owned(), 12852, "66e8840113fc1aaa815e9a34086efe60".to_owned()),
    MasterListItem::new("event".to_owned(), 168, "6b1201afd3e24c827df8cf2086f111ab".to_owned()),
    MasterListItem::new("character_piece_board_reward".to_owned(), 2296, "b5db2b5cad6f4037a780fef7988c5e90".to_owned()),
    MasterListItem::new("pack_mission_progress".to_owned(), 2524, "6688913fbcb515dc75084add264c7976".to_owned()),
    MasterListItem::new("sd".to_owned(), 2112, "b206fbe6d2db274cf3b3b41d84ec6007".to_owned()),
    MasterListItem::new("shop".to_owned(), 136, "ce9f626b2645a58ee7139eb2ca191ab0".to_owned()),
    MasterListItem::new("special_box_gacha_reward_sp".to_owned(), 300, "a8de740c45194bbbf2644cd3ee13d511".to_owned()),
    MasterListItem::new("event_marathon_quest_stage".to_owned(), 56100, "070bae1f6d6cc620aefe282fd5a6cb9a".to_owned()),
    MasterListItem::new("event_quest_boss_stage_itemreward".to_owned(), 78076, "b504f61751b3a12c909ee02d2bedd4b6".to_owned()),
    MasterListItem::new("plan_map_progress_control".to_owned(), 388, "1e3f555456cca3e8e9e5d4e4512d2e51".to_owned()),
    MasterListItem::new("event_box_gacha_infinity".to_owned(), 10652, "16ecda7278271dcb453d833c0179117d".to_owned()),
    MasterListItem::new("gacha_encore_interaction".to_owned(), 164, "081e8e05eac13343366c304fbc63f067".to_owned()),
    MasterListItem::new("loginbonus_item".to_owned(), 19028, "87e641da630763ff2e05fe4bfcc2e07d".to_owned()),
    MasterListItem::new("surprise_quest_enemy".to_owned(), 576, "25c92fb4ce6293591fb298c1ef77a5b1".to_owned()),
    MasterListItem::new("pack".to_owned(), 1066756, "1d8032477c02ad3f109dfe41f95fc575".to_owned()),
    MasterListItem::new("dungeon".to_owned(), 932, "8abcb08dfeb9133ab8467cd92b8b54de".to_owned()),
    MasterListItem::new("skill_pa_fame".to_owned(), 1588, "e9e20ab3f30dccbdc80cf687abfbf814".to_owned()),
    MasterListItem::new("dungeon_benefit".to_owned(), 800, "29ee58716bb12be6b3d81b33fcd237fa".to_owned()),
    MasterListItem::new("expedition_drop".to_owned(), 4972, "2a70706582f16020c7c16af7a43bbda2".to_owned()),
    MasterListItem::new("animation_loginbonus_days".to_owned(), 504, "5f0bbe0b0c097605fc480e13a4519d6a".to_owned()),
    MasterListItem::new("story_event".to_owned(), 22488, "ad08001db6e24ee38c05e8a6359312c4".to_owned()),
    MasterListItem::new("animation_loginbonus_result_pattern".to_owned(), 256, "7b4d1f816dd7d805e98b6f3f527f4044".to_owned()),
    MasterListItem::new("story_release".to_owned(), 2464, "cbf56cd72afc53ce2893726615c81bb5".to_owned()),
    MasterListItem::new("ngword_per".to_owned(), 960, "b508e5229823ba4ae53cab6fd174c6bb".to_owned()),
    MasterListItem::new("roulette_loginbonus_result_pattern".to_owned(), 608, "c024fd8d8874adc522ce19136e4956f5".to_owned()),
    MasterListItem::new("surprise_short_interaction".to_owned(), 892, "2d644b366d59e9d68d9f112dd31c513b".to_owned()),
    MasterListItem::new("equip_weapon".to_owned(), 7132, "5053370d05873783072490e295d6e8dd".to_owned()),
    MasterListItem::new("mainquest_stage_itemreward".to_owned(), 21544, "ac7d87f7f361797b9c72b374b499d11a".to_owned()),
    MasterListItem::new("story_main".to_owned(), 11316, "5e37748a73d99abd23b5138921b402db".to_owned()),
    MasterListItem::new("keyword_campaign".to_owned(), 392, "79a8ccc421521c7d5e8c587d01662010".to_owned()),
    MasterListItem::new("exchange_item".to_owned(), 178052, "65140febd26d85452b1d5687f6e1734c".to_owned()),
    MasterListItem::new("skill_sp".to_owned(), 3028, "5a61748fe05c454191683cd7b71e9bd9".to_owned()),
    MasterListItem::new("special_box_gacha".to_owned(), 160, "06a64b731a7d006555b107929d606591".to_owned()),
    MasterListItem::new("fame_quest_stage_drop_bonus".to_owned(), 496, "ca97a26f5ebf1d3518d4cfd6bd06678d".to_owned()),
    MasterListItem::new("quest_reward".to_owned(), 171352, "b11fecc46e7c3324a4743063cd4035c9".to_owned()),
    MasterListItem::new("story_reminiscence".to_owned(), 1952, "eca207dec22b334352ff1e8c82703b1c".to_owned()),
    MasterListItem::new("gacha_bonus".to_owned(), 2372, "075c872497f478abbd105193de4db688".to_owned()),
    MasterListItem::new("fame_quest_reward_item_list".to_owned(), 6140, "2396677a0647d28c0d6df80f2943cd69".to_owned()),
    MasterListItem::new("story_gacha".to_owned(), 5428, "446d0466d8d84dad591fbe2f5f818f93".to_owned()),
    MasterListItem::new("expedition_animation".to_owned(), 204, "eecb4e07c5222cd88e8507d6d014dddd".to_owned()),
    MasterListItem::new("event_box_gacha_limit".to_owned(), 29900, "42b3730c0f92d703034d38661145c2c4".to_owned()),
    MasterListItem::new("expedition_campaign".to_owned(), 212, "53f48f325fc8a821038fe832b46caa8a".to_owned()),
    MasterListItem::new("surprise_short".to_owned(), 380, "273bdbdf591541ae34b11b08b84810c2".to_owned()),
    MasterListItem::new("scorechallenge_ex_score_reward".to_owned(), 8832, "b5c7cd011edbb07bbdde86e95ffaa53f".to_owned()),
    MasterListItem::new("main_quest_part".to_owned(), 244, "5eba55d2e433fbb390a7288e50f357a3".to_owned()),
    MasterListItem::new("assist".to_owned(), 1548, "2386a3b7fdd2cfea450403be7b2b8c1b".to_owned()),
    MasterListItem::new("equip_accessory_details".to_owned(), 13536, "08bd43e2ba563943b53137034a79d454".to_owned()),
    MasterListItem::new("bonus_character".to_owned(), 1116, "a401f2823d3c16dd347265eefe51f23d".to_owned()),
    MasterListItem::new("battle_enemy_voice".to_owned(), 6940, "5cbdf16b02413bcb9e67cc5416e715bb".to_owned()),
    MasterListItem::new("scorechallenge_ex_battle_time_bonus".to_owned(), 348, "21055931144acb37062f4247ab39097d".to_owned()),
    MasterListItem::new("loginbonus_days".to_owned(), 7336, "f8b4333d778085988325667c273cc429".to_owned()),
    MasterListItem::new("skill_pa_fame_details".to_owned(), 1588, "9159c5a3abfb17bbabf551de1f6c05be".to_owned()),
    MasterListItem::new("expedition_count".to_owned(), 168, "62cacc909d9d62536fdbdb64a4f2793f".to_owned()),
    MasterListItem::new("event_main_quest_reward".to_owned(), 286260, "407c1d3bbc581347eb32f70455359d5f".to_owned()),
    MasterListItem::new("story_fes_button_pattern".to_owned(), 156, "baf876595118cabac1c6b52ceb4f42b3".to_owned()),
    MasterListItem::new("skill_pa_assist_details".to_owned(), 6968, "c20e6e3690f47d257a3bcdd0b90b684d".to_owned()),
    MasterListItem::new("gacha_reset_txt".to_owned(), 200, "7c5e6e8f3edc98c19dee2eef4f11fc64".to_owned()),
    MasterListItem::new("gacha".to_owned(), 49732, "a2a87b7b15f823b76ce4e28e74ec5648".to_owned()),
    MasterListItem::new("event_config".to_owned(), 8500, "abdba2ceccb30f8a1a49a3c2ed8d37a9".to_owned()),
    MasterListItem::new("surprise_quest_wave".to_owned(), 536, "b2d39c31c10a70163b1c29db31b2b567".to_owned()),
    MasterListItem::new("banner".to_owned(), 55384, "691825d96310bd17957fdfc16f84e0aa".to_owned()),
    MasterListItem::new("fame_quest_bonus".to_owned(), 256, "95c12f83e89fe522f0c96770b5c80bef".to_owned()),
    MasterListItem::new("stamina_item".to_owned(), 92, "fc597faea7a89ee86286a2145b9ab3f6".to_owned()),
    MasterListItem::new("scorechallenge_ex_score".to_owned(), 412, "17be4ef6a2193f54ebe4e4d529758155".to_owned()),
    MasterListItem::new("skill_pa".to_owned(), 6572, "83ef74441fd87bc70d3beaeca1ae8867".to_owned()),
    MasterListItem::new("member_illustration".to_owned(), 380, "781ee3014817396684b63ec7bd824a82".to_owned()),
    MasterListItem::new("surprise_event_point_weight".to_owned(), 2020, "da80ac726e6b7eff4ffd1b7ed948348c".to_owned()),
    MasterListItem::new("scorechallenge_mission_strategy_version".to_owned(), 140, "e6a96808f986743b251fbdef6ea5f6df".to_owned()),
    MasterListItem::new("dungeon_stage".to_owned(), 10684, "6ac13313a74d14c6f7f9f8ac9d92604b".to_owned()),
    MasterListItem::new("event_mission_recommend".to_owned(), 8160, "eb1692b220bf7d8ff89b33813a525d4b".to_owned()),
    MasterListItem::new("scorechallenge".to_owned(), 8092, "7541d4af52de458b8c4ce8f4af775d08".to_owned()),
    MasterListItem::new("clienttext".to_owned(), 226444, "7da3eb99f61443c0fa759863b3d3e065".to_owned()),
    MasterListItem::new("surprise_event_point".to_owned(), 1944, "13388e193cbadf4965f31bbebe06920c".to_owned()),
    MasterListItem::new("campaign_box_gacha_limit".to_owned(), 396, "e6f0fb9a919547b5eb87acc440baa75a".to_owned()),
    MasterListItem::new("app_version".to_owned(), 1108, "9746e6bb76701740612e034035d63c6f".to_owned()),
    MasterListItem::new("surprise_quest_score".to_owned(), 288, "b2885159a93751b75ec1f74063059ffa".to_owned()),
    MasterListItem::new("fame_quest_stage_itemreward".to_owned(), 1556, "9788808181fd300d410da7f3eafe7766".to_owned()),
    MasterListItem::new("random_loginbonus_result".to_owned(), 1568, "a770554767a91e8981925d590411bf0b".to_owned()),
    MasterListItem::new("member_lv_exp".to_owned(), 1800, "b9ffea4de416e4d006eed2b18c815b0f".to_owned()),
    MasterListItem::new("title_theme".to_owned(), 1124, "e5eaea82405e89f23d7db4792b1a2282".to_owned()),
    MasterListItem::new("gacha_performance".to_owned(), 6188, "4d2e37000d8772c81f73e6c48e8de774".to_owned()),
    MasterListItem::new("surprise_event".to_owned(), 680, "854768c6e2aee6d8a7ae417c7d21fc63".to_owned()),
    MasterListItem::new("scorechallenge_ex_score_reward_item".to_owned(), 3288, "7d936ea3447270c325f308e9babe9bcd".to_owned()),
    MasterListItem::new("gacha_priority".to_owned(), 1332, "46f010441b6daec3a6796d9a7fa7536b".to_owned()),
    MasterListItem::new("target_type_character".to_owned(), 140, "df04582d07b220b83c3af89f2ad7dab1".to_owned()),
    MasterListItem::new("scorechallenge_ex_reward_achievement".to_owned(), 58324, "1647fb5bfd255f3c0752a22d4eb4757b".to_owned()),
    MasterListItem::new("skill_assist_details".to_owned(), 8108, "10ba7285a95ca375fd15c13888f9ccff".to_owned()),
    MasterListItem::new("between_event".to_owned(), 428, "47de207e13a7e68976587d82ea452a92".to_owned()),
    MasterListItem::new("event_scorechallenge_stage_reward_achievement".to_owned(), 964, "2ac8b01d631722b5414845bad78986a2".to_owned()),
    MasterListItem::new("download_page".to_owned(), 556, "7c9016d1f4946edce1956c3504cca7a0".to_owned()),
    MasterListItem::new("okword".to_owned(), 84, "e0e060defe9a8c8ca1c2d41d8675d0b1".to_owned()),
    MasterListItem::new("menu_theme".to_owned(), 520, "411c02010ae8da495095d7896e4fb653".to_owned()),
    MasterListItem::new("battle_position".to_owned(), 1512, "89dbb5618eab7e8e8544163cba1d1248".to_owned()),
    MasterListItem::new("shop_item".to_owned(), 18288, "266efb41ad760aa541c99f2c449f50f1".to_owned()),
    MasterListItem::new("content_release".to_owned(), 528, "8ca77c1828020854dd68cd0ce1dd8cd1".to_owned()),
    MasterListItem::new("gacha_raise_performance_lot".to_owned(), 336, "fc568856d7e222fb3653fec8d58a7d54".to_owned()),
    MasterListItem::new("event_quest_stage_itemreward".to_owned(), 31832, "cd222015fa01870641178ccbfded2727".to_owned()),
    MasterListItem::new("master_version".to_owned(), 144, "55b8d30fa8a17937875a62163c30ca63".to_owned()),
    MasterListItem::new("roulette_loginbonus_days".to_owned(), 1356, "dfebe8ce05fe98e6eece019f7335a493".to_owned()),
    MasterListItem::new("fame_quest_area".to_owned(), 912, "ddaed85646a8ca991cb7c5ee86e0ff29".to_owned()),
    MasterListItem::new("skill_pa_fame_set_chara".to_owned(), 268, "18b0d186101b4e706907bfafc4db9ba6".to_owned()),
    MasterListItem::new("scorechallenge_cheat".to_owned(), 4720, "25b3889e3dcb6b52e988bc3dc2011827".to_owned()),
    MasterListItem::new("omikuji_lot".to_owned(), 1084, "aa6c9464fd7a32b8fdb3e11d20665cf8".to_owned()),
    MasterListItem::new("presenttext".to_owned(), 13156, "db803da0ee2aa60ce37b31f75de6f05b".to_owned()),
    MasterListItem::new("event_scorechallenge_stage".to_owned(), 748, "9832c4ff07ec25ac0682ee76898d1b3b".to_owned()),
    MasterListItem::new("friend_greeting_item".to_owned(), 104, "c4a7797e1853f942d49ae5f0ea819c9f".to_owned()),
    MasterListItem::new("event_scorechallenge".to_owned(), 220, "481ac9fc879f55e2b28b0c097f82efdf".to_owned()),
    MasterListItem::new("skill_enemy_details".to_owned(), 145636, "3370e40af7f9ab784c00314535fdd081".to_owned()),
    MasterListItem::new("huntingquest_stage".to_owned(), 7876, "b5e5f027a8fdb45e116353107650fb59".to_owned()),
    MasterListItem::new("skill_pa_details".to_owned(), 34244, "f3725f539cc45849396bdbf3e896e7a0".to_owned()),
    MasterListItem::new("dungeon_benefit_lot".to_owned(), 596, "b407e3fa7332dc7e7cbc783eaedf9894".to_owned()),
    MasterListItem::new("lottery".to_owned(), 256, "68e6982ef4eb15074971b8778c5b574e".to_owned()),
    MasterListItem::new("skill_pa_field_details".to_owned(), 580, "873807d7574b1c291d95edb344aa061a".to_owned()),
    MasterListItem::new("rewrite_fame_quest_stage_position".to_owned(), 380, "bd3306c974187f3754bb9abffe149429".to_owned()),
    MasterListItem::new("scorechallenge_reward".to_owned(), 29076, "3815771e5d50b9990bbd60dd480950a6".to_owned()),
    MasterListItem::new("random_loginbonus_interaction".to_owned(), 1312, "4d07ed12ff738f6d45656e7b2d37124e".to_owned()),
    MasterListItem::new("fame_rank".to_owned(), 312, "f7fccaf49bdda287599e594bd26b2b0c".to_owned()),
    MasterListItem::new("skill_pa_fame_lot_rare".to_owned(), 224, "031164d88c5d7506f5dae5f357377e07".to_owned()),
    MasterListItem::new("member_medal_rate".to_owned(), 92, "592f0a0f6c6a8f4a23c029c1b57040f1".to_owned()),
    MasterListItem::new("pack_progresslist".to_owned(), 10080, "caf93d80eb9433665347133db9e02ba6".to_owned()),
    MasterListItem::new("shop_balloon".to_owned(), 10156, "58ad6700fe0969d4f4f4ad0e4e59f231".to_owned()),
    MasterListItem::new("exchange".to_owned(), 6492, "34bd23f36083ea1eb3f3bac8118ad827".to_owned()),
    MasterListItem::new("scorechallenge_ex_stage".to_owned(), 3084, "8d815078a4585e8fbbfd712962f3b1ae".to_owned()),
    MasterListItem::new("surprise_story".to_owned(), 408, "640270646eed0edb6741f3d898644b18".to_owned()),
    MasterListItem::new("multi_battle_const".to_owned(), 192, "0842e3b65ba849920290c172d39a8134".to_owned()),
    MasterListItem::new("constant".to_owned(), 8320, "376ee7a7094f1bf0c0803a0da4815ed3".to_owned()),
    MasterListItem::new("campaign_box_gacha".to_owned(), 308, "301da2c3577feeda96fe6ce78c2f1fb3".to_owned()),
    MasterListItem::new("fame_random_potion_lot".to_owned(), 204, "08eb29238a3739c1c70ca80c2517d14d".to_owned()),
    MasterListItem::new("special_box_gacha_reward".to_owned(), 588, "aef987503635ebbad96ce5519d1d2645".to_owned()),
    MasterListItem::new("comeback_present_item".to_owned(), 148, "47de8df3e6694dee636ee5c9fe916d7e".to_owned()),
    MasterListItem::new("honor".to_owned(), 6600, "390134fc385553ee89e5833fcb6373fd".to_owned()),
    MasterListItem::new("fame_quest_rank".to_owned(), 264, "54244d1a5893521008533ec6c0dce1d0".to_owned()),
    MasterListItem::new("surprise_short_result".to_owned(), 1216, "854ac0a0924ed99407df27a06c38013a".to_owned()),
    MasterListItem::new("omikuji".to_owned(), 260, "747688e35315296d885d646778c8d616".to_owned()),
    MasterListItem::new("costume".to_owned(), 8876, "bdc0d8bcd3eaf1a4d798b04c393596a5".to_owned()),
    MasterListItem::new("navi_looks_replace".to_owned(), 852, "fd4cd3bb7c4d34656b858e30aecd1e94".to_owned()),
    MasterListItem::new("gacha_continuation".to_owned(), 328, "c06cd57d6843b329bce948d497123159".to_owned()),
    MasterListItem::new("dungeon_benefit_level".to_owned(), 1696, "965aa431214fcc8d9e3fe25f58ea6d41".to_owned()),
    MasterListItem::new("gacha_member_select".to_owned(), 276, "fa6ac7f9c2b361183e0118dfbc0a67cf".to_owned()),
    MasterListItem::new("assist_medal_rate".to_owned(), 124, "5d4fbd0093fd61017fd898ab832163a5".to_owned()),
    MasterListItem::new("gallery_movie".to_owned(), 644, "868953864ac504c8ba9d4692bd47927a".to_owned()),
    MasterListItem::new("skill_pa_party_filter".to_owned(), 248, "0553261aee39199ced4a4fc0957a4688".to_owned()),
    MasterListItem::new("mainquest_stage".to_owned(), 51504, "45e614944854b9013e0f4405406f9453".to_owned()),
    MasterListItem::new("character_enhance_trial".to_owned(), 1948, "317e543e84eb5ab27c168aca1eeb52a3".to_owned()),
    MasterListItem::new("huntingquest_schedule".to_owned(), 396, "c5c03f722b5d608eda3c0b69130dfdf6".to_owned()),
    MasterListItem::new("surprise_short_interaction_group".to_owned(), 268, "b877bda60e41fa8e576d48dfbb783e4e".to_owned()),
    MasterListItem::new("battle_enemy_ai".to_owned(), 110324, "e0afc3ed2ee3dcb00a529910d1d015ce".to_owned()),
    MasterListItem::new("initialitem".to_owned(), 664, "a0a185a9606b02b9eee0a85cf7388ecb".to_owned()),
    MasterListItem::new("event_marathon_quest_stage_boss_multi".to_owned(), 13004, "52e4a646de10ef635319f016cb041eff".to_owned()),
    MasterListItem::new("character_enhance_stage_position".to_owned(), 584, "2a23458986a64c1f2840692f37896626".to_owned()),
    MasterListItem::new("target_costume".to_owned(), 648, "42ab04d48650026ede802bee46a7d652".to_owned()),
    MasterListItem::new("skill_pa_filter".to_owned(), 952, "d6a4ba5ebe0b6ef6b9feb974f2f83a82".to_owned()),
    MasterListItem::new("notice".to_owned(), 264, "625b9b5788e8a047a77173f7c01e0945".to_owned()),
    MasterListItem::new("apple_tier_price".to_owned(), 688, "7694b10a3f4e988b3286fdf970f9bed3".to_owned()),
    MasterListItem::new("skill_sp_group".to_owned(), 1224, "485f72d54f22a50b9eb16b5b657c26b5".to_owned()),
    MasterListItem::new("battle_enemy".to_owned(), 256460, "bdba5749c2472220e0e2303104ad85aa".to_owned()),
    MasterListItem::new("intimacy_exp".to_owned(), 640, "bb45bca2282e976906a90e3360a0944e".to_owned()),
    MasterListItem::new("battle_wave".to_owned(), 259084, "dedac12407a6162b2db97b2e275a1a64".to_owned()),
    MasterListItem::new("voice".to_owned(), 131408, "b8f79614f6df56f127c57a8b8246a341".to_owned()),
    MasterListItem::new("plan_map_progress".to_owned(), 404, "8d0d4bd9c1a37a298dc17def913851be".to_owned()),
    MasterListItem::new("event_member".to_owned(), 1884, "4c04cd83cae07a461475539184450994".to_owned()),
    MasterListItem::new("fame_quest_mission".to_owned(), 288, "545a556006c200aacf0fc0c8ec684f6e".to_owned()),
    MasterListItem::new("campaign_item_gacha_lot".to_owned(), 240, "63a2ff049b2535865e49880c95e37427".to_owned()),
    MasterListItem::new("gacha_encore".to_owned(), 184, "22e6367fc21dffd3cde00e517ded60d2".to_owned()),
    MasterListItem::new("mission_recommend".to_owned(), 144, "3bc8e6b0748b3d62e40337d38631e34b".to_owned()),
    MasterListItem::new("fame_quest_reward".to_owned(), 2136, "287810b90ccf81ea7ebf03a56ae2d763".to_owned()),
    MasterListItem::new("assetname".to_owned(), 2021212, "ed8cd04fcc6401ca0be6b819f2d78268".to_owned()),
    MasterListItem::new("lottery_win_number".to_owned(), 224, "638e4a6be4773a18610c2bf2112cbfd4".to_owned()),
    MasterListItem::new("random_loginbonus".to_owned(), 716, "8b3bab3066b8ae851e00f1f4009def2a".to_owned()),
    MasterListItem::new("scorechallenge_ex".to_owned(), 2980, "0ec7f39930f3ba2bc6be7c7ee700857e".to_owned()),
    MasterListItem::new("member_lv_limit".to_owned(), 84, "ca1de4fa5f15eb75f99dc8eb8e6c4939".to_owned()),
    MasterListItem::new("material_list".to_owned(), 3568, "4df5001b83e98a19e508a386c6ce7288".to_owned()),
    MasterListItem::new("surprise_story_select_pattern".to_owned(), 580, "a57a3c82bdc435269f6b51e7c4adbd3b".to_owned()),
    MasterListItem::new("random_loginbonus_selection".to_owned(), 1128, "03d5bfd014bfb8d66d121a0c7099e268".to_owned()),
    MasterListItem::new("expedition".to_owned(), 268, "cb1a2b0ea9c64a0ed71cf4b80b6ae700".to_owned()),
    MasterListItem::new("assist_details".to_owned(), 9144, "7584917780ad1591a2c182086685c571".to_owned()),
    MasterListItem::new("partyoffer".to_owned(), 160, "8e66fc6daa97dc064f54bf4cdee649aa".to_owned()),
    MasterListItem::new("ad_movie".to_owned(), 3712, "ca5dc4dc6eae5f80fdfaab49016c97b8".to_owned()),
    MasterListItem::new("scorechallenge_reward_achievement".to_owned(), 115384, "4dc74bc1b3e34edd422ed47edb3c5418".to_owned()),
    MasterListItem::new("story_member".to_owned(), 8376, "2e3ce696dc9bd01c64fe118114ba024f".to_owned()),
    MasterListItem::new("skill_pa_fame_probability_table".to_owned(), 452, "4999f16be8cf5b7184245078e351020d".to_owned()),
    MasterListItem::new("surprise_quest".to_owned(), 124, "92d7b5331dd6eede81764efb9c8d56ef".to_owned()),
    MasterListItem::new("campaign_item_gacha".to_owned(), 256, "e68d8fc0744ea4641cc941cba4c9aa80".to_owned()),
    MasterListItem::new("gacha_character_piece_rate".to_owned(), 100, "770c92195a9264fd313bf77f23b97800".to_owned()),
    MasterListItem::new("sd_position".to_owned(), 180, "5864df0ac70b61cc3d1f55bb1ea7ad78".to_owned()),
    MasterListItem::new("story_main_force_open".to_owned(), 208, "82bf99b87d51511c6313fc1683d15130".to_owned()),
    MasterListItem::new("exp".to_owned(), 3576, "050a6fa273c51098a1bde9a288523276".to_owned()),
    MasterListItem::new("character_piece".to_owned(), 392, "a44295c763be02d9d4f3b75679c62509".to_owned()),
    MasterListItem::new("dungeon_stage_item_reward".to_owned(), 4864, "5f6a24d132e9ba04968a63f75d37720c".to_owned()),
    MasterListItem::new("pack_mission".to_owned(), 288, "a0c49c3a8be8bb4987cac22581147243".to_owned()),
    MasterListItem::new("member_limitbreak_materiall".to_owned(), 33952, "6f5c1fe54084e1b08890081dfa1ce71a".to_owned()),
    MasterListItem::new("roulette_loginbonus_view".to_owned(), 568, "4b0343a6bb6bc5c754c00e5bacaa643c".to_owned()),
    MasterListItem::new("surprise_short_reward_view".to_owned(), 464, "43d9da83f2328fdb17a4942f77ffd0ef".to_owned()),
    MasterListItem::new("skill_pa_field_group".to_owned(), 372, "8575254e52a403591fb9de16c4b46096".to_owned()),
    MasterListItem::new("expedition_drop_chara".to_owned(), 984, "e86907f38c5bbaa47467b3080f6431e4".to_owned()),
    MasterListItem::new("skill_assist".to_owned(), 3624, "f176a0cb42c188d754f9c4939b9ae7c1".to_owned()),
    MasterListItem::new("mission_panel".to_owned(), 23500, "cebd86573aa2df5dc40abc43763a44b8".to_owned()),
    MasterListItem::new("surprise_short_selection_voice".to_owned(), 532, "73c8cbdbb0a38d46ee90a9a19d117114".to_owned()),
    MasterListItem::new("dungeon_area".to_owned(), 2468, "c079f1694268b6f2f4de6d79bd00d877".to_owned()),
    MasterListItem::new("battle_enemy_condition".to_owned(), 608, "2ffa9b7eb7947d46c65151ec19c627f1".to_owned()),
    MasterListItem::new("surprise_event_stock_limit".to_owned(), 524, "64d5219625b26be3e295d516961a953b".to_owned()),
    MasterListItem::new("fame_quest_stage".to_owned(), 5212, "f2a3d6931cfef04bf013ffae18a45243".to_owned()),
    MasterListItem::new("story_etc".to_owned(), 6504, "cfd02812677dcfca324098a708128dcd".to_owned()),
    MasterListItem::new("scorechallenge_tab".to_owned(), 11032, "fe8446c8590041e70ab3102c19008c10".to_owned()),
    MasterListItem::new("member_lv_limitbreak".to_owned(), 84, "507b8a7a48d4b5717575bc2aa37cbb64".to_owned()),
    MasterListItem::new("skipticket".to_owned(), 616, "646f0f4568ab534fc0614a19f6bc4e20".to_owned()),
    MasterListItem::new("skill_ac_details".to_owned(), 69900, "4f4861f3d022692887d6f1e8586d4a38".to_owned()),
    MasterListItem::new("surprise_story_result".to_owned(), 720, "6a817622403656641c1de2ad6204b89b".to_owned()),
    MasterListItem::new("background".to_owned(), 4116, "bae4640527b765486507ddb045030f70".to_owned()),
    MasterListItem::new("character_looks_replace".to_owned(), 464, "7053eaa01326716842c158f048c5e0dd".to_owned()),
    MasterListItem::new("scorechallenge_ex_rarity_bonus".to_owned(), 120, "06eebe46f8cd224f640b69a85f12b5cd".to_owned()),
    MasterListItem::new("event_scorechallenge_tab".to_owned(), 464, "cfb07a2afecbb2fdf37f0f0b9c27679c".to_owned()),
    MasterListItem::new("multi_battle_survivor_coefficient".to_owned(), 84, "4ce06428e2d2fdaa5abc054e908ccae7".to_owned()),
    MasterListItem::new("battle_voice".to_owned(), 50984, "73dd85ce86960738d921e7bb5b90fc8b".to_owned()),
    MasterListItem::new("skill_enemy".to_owned(), 26776, "43286f0baf2a61aafd2182b82ca61819".to_owned()),
    MasterListItem::new("bonus_skill_pa_fame_lot_rare".to_owned(), 216, "13532c6a76bec25e7610d0e5ceb6116c".to_owned()),
    MasterListItem::new("dungeon_clear_rank".to_owned(), 1020, "ed1d97c34e0c7e4e7bf8361ec38ef770".to_owned()),
    MasterListItem::new("exchange_currency".to_owned(), 104, "9d4fbba2fb2cc4ef50928463ea93a9f5".to_owned()),
    MasterListItem::new("mission_honor".to_owned(), 8024, "cd28f53eb1eeddaa51e73bc0da0dd36f".to_owned()),
    MasterListItem::new("target_type".to_owned(), 140, "e032124e6285765ffcb1e4707faa1ded".to_owned()),
    MasterListItem::new("skill_pa_assist".to_owned(), 2452, "715d16ba7153e913274ab073d219fe9d".to_owned()),
    MasterListItem::new("gacha_limit".to_owned(), 4128, "be74b467e6a751268647f43719fe7880".to_owned()),
    MasterListItem::new("character_enhance_change_display".to_owned(), 128, "9cca472c762bd2124c92bbe3eec07a9e".to_owned()),
    MasterListItem::new("plan_map_appeal".to_owned(), 340, "f13a01db4520d2d288180c603a14b064".to_owned()),
    MasterListItem::new("serial_code".to_owned(), 248, "74b067505bf92f491592454a54638c76".to_owned()),
    MasterListItem::new("gacha_member_select_list".to_owned(), 12068, "f1b53e7e1e5233b4b6de0ec79ab582ac".to_owned()),
    MasterListItem::new("twitter_campaign".to_owned(), 352, "6564de0dabdaf2bb0a3848e48d24cdd5".to_owned()),
    MasterListItem::new("surprise_quest_wave_luck".to_owned(), 1136, "3544e5dca67ca75a1da0807db958e4f3".to_owned()),
    MasterListItem::new("errortext".to_owned(), 16696, "7d79e181d99c16ace9f2ebe203a81417".to_owned()),
    MasterListItem::new("navi".to_owned(), 514012, "b934018d667fd69f302a072403c7edc5".to_owned()),
    MasterListItem::new("loginbonus".to_owned(), 11076, "1b7eb6a310ebc6717874669492d42b9f".to_owned()),
    MasterListItem::new("scorechallenge_ex_survival_bonus".to_owned(), 188, "c702e9d46ce865918fbec2826c56312d".to_owned()),
    MasterListItem::new("fame_status_reflected_value".to_owned(), 172, "46d8bda60cf91ac5d1e9017a121d3508".to_owned()),
  ]
});
