//! Reference: https://youtu.be/kbyEkBIw4-U

use crate::api::battle::{BattleMember, BattleParty, BattleStartResponse};
use crate::api::party_info::{Party, PartyForm, SpecialSkillInfo};
use crate::api::surprise::BasicBattlePartyForm;
use crate::api::{MemberFameStats, NotificationData, RemoteDataItemType};
use crate::call::{CallCustom, CallResponse};
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use crate::member::{FetchUserMembers, FetchUserMembersIn, Member, MemberActiveSkill, MemberPrototype, MemberStrength};
use crate::user::session::Session;
use crate::AppState;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

// See [Wonder_Api_ScorechallengeinfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeInfoResponse {
  pub challengenum: i32,
  pub normalscore: i32,
  pub hardscore: i32,
  pub veryhardscore: i32,
  pub totalscore: i32,
  pub rank: i32,
  pub in_ranking_period: i32,
  pub beforeinfo: Vec<ScoreChallengeinfoBeforeInfo>,
  pub best_score_info: Vec<ScoreChallengeMissionBestScoreInfo>,
}

impl CallCustom for ScoreChallengeInfoResponse {}

// See [Wonder_Api_ScorechallengeinfoBeforeinfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeinfoBeforeInfo {
  pub scorechallengeid: i32,
  pub score: i32,
  pub rank: i32,
}

// See [Wonder_Api_ScorechallengeMissionBestScoreInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeMissionBestScoreInfo {
  pub elemental: String,
  pub scorechallenge_id: i32,
  pub best_score: i32,
  pub reward_type: i32,
}

pub async fn score_challenge_info() -> impl IntoHandlerResponse {
  warn!("encountered stub: score_challenge_info");

  Ok(Unsigned(ScoreChallengeInfoResponse {
    challengenum: 3,
    normalscore: 15000,
    hardscore: 12000,
    veryhardscore: 8000,
    totalscore: 35000,
    rank: 42,
    in_ranking_period: 1,
    beforeinfo: vec![],
    best_score_info: vec![],
  }))
}

// See [Wonder_Api_ScorechallengerankingResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeRankingResponse {
  pub ranking: Vec<ScoreChallengeRanking>,
}

impl CallCustom for ScoreChallengeRankingResponse {}

// See [Wonder_Api_ScorechallengerankingRankingResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeRanking {
  pub user_no: String,
  pub icon: i64,
  pub name: String,
  pub score: i32,
  pub rank: i32,
  pub scoremode: i32,
  pub honor_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ScoreChallengeRankingRequest {
  /// 1 - around me, 2 - global.
  pub mode: i32,
  /// Start after: 1, 1k, 5k, 10k, 20k, 30k (29951), 0 for around me.
  /// Can be an arbitrary number if "View next" or "View previous" is pressed.
  pub ranking: i32,
}

pub async fn score_challenge_ranking(Params(params): Params<ScoreChallengeRankingRequest>) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: score_challenge_ranking");

  Ok(Unsigned(ScoreChallengeRankingResponse {
    ranking: vec![ScoreChallengeRanking {
      user_no: "-1".to_string(),
      icon: 1083110,
      name: "Megumin".to_owned(),
      score: 120_000_000,
      rank: 3,
      scoremode: params.mode,
      honor_id: 62010250,
    }],
  }))
}

// See [Wonder_Api_ScorechallengebestscorepartyResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeBestScorePartyResponse {
  pub user_name: String,
  pub icon: i64,
  pub rank: i32,
  pub bestscore: i32,
  /// Not displayed in-game.
  pub party_name: String,
  pub strength: i32,
  pub party: ScoreChallengeBestScoreParty,
  pub member: Vec<BestScorePartyMember>,
}

impl CallCustom for ScoreChallengeBestScorePartyResponse {}

// See [Wonder_Api_ScorechallengebestscorepartyMemberResponseDto_Fields]
// See [Wonder_Api_BasicBestScorePartyMemberResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct BestScorePartyMember {
  pub member_id: i64,
  pub lv: i32,
  pub ex_flg: i32,
}

// See [Wonder_Api_ScorechallengebestscorepartyPartyResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeBestScoreParty {
  pub party_forms: Vec<ScoreChallengeBestscorePartyFormInfoResponse>,
  pub assist: i32,
  pub sub_assists: Vec<i32>,
  pub party_passive_skill: ScoreChallengeBestscorePartyPassiveSkillInfoResponse,
}

// See [Wonder_Api_ScorechallengeBestscorePartyFormInfoResponseDto_Fields]
// See [Wonder_Api_BasicBestScorePartyFormInfoResponseDto_Fields]
/// Member IDs must be master IDs, not client-unique IDs, compared to other PartyForm structs.
#[derive(Debug, Serialize)]
pub struct ScoreChallengeBestscorePartyFormInfoResponse {
  pub form_no: i32,
  pub main: i32,
  pub sub1: i32,
  pub sub2: i32,
  pub weapon: i64,
  pub acc: i64,
  pub specialskill: i64,
  pub skill_pa_fame: i64,
}

// See [Wonder_Api_ScorechallengeBestscorePartyPassiveSkillInfoResponseDto_Fields]
// See [Wonder_Api_BasicBestScorePartyPassiveSkillInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeBestscorePartyPassiveSkillInfoResponse {
  pub skill_id: i64,
  pub member_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ScoreChallengeBestScorePartyRequest {
  pub user_no: i32,
  pub scorechallengeid: i32,
}

pub async fn score_challenge_best_score_party(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<ScoreChallengeBestScorePartyRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: score_challenge_best_score_party");

  let client = state.get_database_client().await?;

  // Send only members that are used in the party, no hardlock occurs, but just to expose less data
  let fetch_members = FetchUserMembers::new(&client).await.unwrap();
  let members = fetch_members.run(session.user_id).await.unwrap();

  Ok(Unsigned(ScoreChallengeBestScorePartyResponse {
    user_name: "Megumin".to_string(),
    icon: 1083110,
    rank: 42,
    bestscore: 120_000_000,
    party_name: "Kazumin".to_string(),
    strength: 99999,
    party: ScoreChallengeBestScoreParty {
      party_forms: (1..=5)
        .map(|i| ScoreChallengeBestscorePartyFormInfoResponse {
          form_no: i as i32,
          main: members.get(i - 1).map_or(0, |m| m.id),
          sub1: 0,
          sub2: 0,
          weapon: 0,
          acc: 0,
          specialskill: 100001,
          skill_pa_fame: 0,
        })
        .collect::<Vec<_>>(),
      assist: 0,
      sub_assists: vec![],
      party_passive_skill: ScoreChallengeBestscorePartyPassiveSkillInfoResponse {
        skill_id: 0,
        member_id: 0,
      },
    },
    member: members
      .iter()
      .map(|m| BestScorePartyMember {
        member_id: m.id as i64,
        lv: m.level(),
        ex_flg: 0,
      })
      .collect::<Vec<_>>(),
  }))
}

// See [Wonder_Api_ScorechallengestartResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeStartResponse {
  pub party: BattleParty,
  pub members: Vec<BattleMember>,
}

impl CallCustom for ScoreChallengeStartResponse {}

#[derive(Debug, Deserialize)]
pub struct ScoreChallengeStartRequest {
  #[serde(with = "crate::bool_as_int")]
  pub is_practice: bool,
  pub quest_id: i32,
  #[serde(rename = "party_no")]
  pub party_id: i32,
}

pub async fn score_challenge_start(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<ScoreChallengeStartRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: score_challenge_start");

  let client = state.pool.get().await.context("failed to get database connection")?;

  let statement = client
    .prepare(
      /* language=postgresql */
      r#"
      select
        up.party_id,
        -- Incidentally, client expects party name to be inside each form,
        -- which is exactly how JOIN returns it.
        up.name,
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
      order by upf.form_id
    "#,
    )
    .await
    .context("failed to prepare statement")?;
  let forms = client
    .query(&statement, &[&session.user_id, &(params.party_id as i64)])
    .await
    .context("failed to execute query")?;
  let forms = forms
    .into_iter()
    .map(|row| {
      let party_name: String = row.get(1);
      let form_id: i64 = row.get(2);
      let main_member_id: i64 = row.get(3);
      let sub1_member_id: i64 = row.get(4);
      let sub2_member_id: i64 = row.get(5);
      let weapon_id: i64 = row.get(6);
      let accessory_id: i64 = row.get(7);
      let special_skill_id: i64 = row.get(8);

      PartyForm {
        id: form_id as i32,
        form_no: form_id as i32,
        party_no: params.party_id,
        main: main_member_id as i32,
        sub1: sub1_member_id as i32,
        sub2: sub2_member_id as i32,
        weapon: weapon_id,
        acc: accessory_id,
        name: party_name,
        strength: 123,
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

  let party = Party::new(forms, params.party_id);

  // We must send only members that are used in the party, otherwise hardlock occurs
  let fetch_members = FetchUserMembersIn::new(&client).await.unwrap();
  #[rustfmt::skip]
  let members = fetch_members.run(
    session.user_id,
    &party.party_forms.iter().map(|form| form.main as i64).collect::<Vec<_>>(),
  ).await.unwrap();

  Ok(Unsigned(ScoreChallengeStartResponse {
    party: party.to_battle_party(),
    members: members
      .into_iter()
      .map(|member| {
        let form = party.party_forms.iter().find(|form| form.main == member.id).unwrap();
        member.to_battle_member(form)
      })
      .collect(),
  }))
}

// See [Wonder_Api_ScorechallengeresultResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeResultResponse {
  /// How many challenges to deduct?
  pub challengenum: i32,
  pub reward: Vec<ScoreChallengeResultReward>,
  pub totalrewardids: Vec<i32>,
  pub rank: i32,
  pub ranktype: i32,
  pub in_ranking_period: i32,
  pub totalscore: i32,
  pub previous_bestscore: i32,
  pub bestscore: i32,
  pub modescore: i32,
}

impl CallCustom for ScoreChallengeResultResponse {}

// See [Wonder_Api_ScorechallengeresultRewardResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeResultReward {
  pub itemtype: i32,
  pub itemid: i64,
  pub itemnum: i32,
  pub rewardtype: i32,
}

// body={"quest_id": "320391", "memcheckcount": "0", "score": "958528312", "party_no": "1", "seed": "958527983", "original_score": "727"}
#[derive(Debug, Deserialize)]
pub struct ScoreChallengeResultRequest {
  pub quest_id: i32,
  pub memcheckcount: i32,
  // See [_vst_an$$fhmcQTT]
  /// `decrypted_score = score ^ seed`.
  pub score: i64,
  pub party_no: i32,
  pub seed: i64,
  pub original_score: i32,
}

pub async fn score_challenge_result(
  state: Arc<AppState>,
  session: Arc<Session>,
  Params(params): Params<ScoreChallengeResultRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: score_challenge_result");

  Ok(Unsigned(ScoreChallengeResultResponse {
    challengenum: 1,
    reward: vec![ScoreChallengeResultReward {
      itemtype: RemoteDataItemType::RealMoney.into(),
      itemid: 1,
      itemnum: 42000,
      rewardtype: 1,
    }],
    totalrewardids: vec![101, 100208],
    rank: 42,
    ranktype: 1,
    in_ranking_period: 1,
    totalscore: 35000,
    previous_bestscore: 42_000,
    bestscore: 1_200_000,
    modescore: 600_000,
  }))
}

// See [Wonder_Api_ScorechallengeMissionResponseDto_Fields]
#[derive(Debug, Serialize)]
/*

struct Wonder_Api_ScorechallengeMissionResponseDto_Fields : Wonder_Api_ResponseDtoBase_Fields {
  int32_t status;
  struct System_Collections_Generic_List_ScorechallengeMissionBestScoreInfoResponseDto__o* best_score_info;
  struct System_Collections_Generic_List_ScorechallengeMissionDisplayBestScoreInfoResponseDto__o* display_best_score_info;
};
 */
pub struct ScoreChallengeMissionResponse {
  pub best_score_info: Vec<ScoreChallengeMissionBestScoreInfo>,
  pub display_best_score_info: Vec<ScoreChallengeMissionDisplayBestScoreInfo>,
}

impl CallCustom for ScoreChallengeMissionResponse {}

// See [Wonder_Api_ScorechallengeMissionDisplayBestScoreInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeMissionDisplayBestScoreInfo {
  pub elemental: String,
  pub best_score: i32,
  pub previous_best_score: i32,
}

pub async fn score_challenge_mission() -> impl IntoHandlerResponse {
  warn!("encountered stub: score_challenge_mission");

  Ok(Unsigned(ScoreChallengeMissionResponse {
    best_score_info: vec![],
    display_best_score_info: vec![],
  }))
}

// See [Wonder_Api_ScorechallengeMissionListResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeMissionListResponse {
  pub scorechallenge_mission_list: Vec<ScoreChallengeMissionListInfo>,
}

impl CallCustom for ScoreChallengeMissionListResponse {}

// See [Wonder_Api_ScorechallengeMissionListInfoResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct ScoreChallengeMissionListInfo {
  pub mission_id: i32,
  pub achieve_flag: i32,
}

pub async fn score_challenge_mission_list() -> impl IntoHandlerResponse {
  warn!("encountered stub: score_challenge_mission_list");

  Ok(Unsigned(ScoreChallengeMissionListResponse {
    scorechallenge_mission_list: vec![],
  }))
}
