use crate::api::master_all::get_master_manager;
use crate::api::RemoteDataItemType;
use crate::member::{FetchUserMembers, MemberPrototype};
use crate::user::overrides::FetchUserOverrides;
use crate::user::session::Session;
use anyhow::Context;
use deadpool_postgres::Client;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::LazyLock;
use tracing::{debug, trace, warn};

#[rustfmt::skip]
type MigrationFn = Box<dyn for<'a> Fn(
  &'a Session,
  &'a mut Client
) -> Pin<Box<dyn Future<Output = u64> + Send + 'a>> + Send + Sync>;

pub struct Migration {
  pub name: &'static str,
  pub function: MigrationFn,
}

macro_rules! migration {
  ($func:expr) => {
    Migration {
      name: stringify!($func),
      function: Box::new(|session, client| Box::pin($func(session, client))),
    }
  };
}

pub async fn run_migrations(session: &Session, client: &mut Client) {
  static MIGRATIONS: LazyLock<Vec<Migration>> = LazyLock::new(|| {
    vec![
      migration!(add_user_characters),
      migration!(add_user_members),
      migration!(add_user_members_reserve),
      migration!(add_user_parties),
      migration!(add_user_items),
      migration!(add_user_equipment),
      migration!(add_user_character_special_skills),
      migration!(add_user_party_form_skills),
      migration!(add_user_member_skills),
      migration!(add_user_home_illustrations),
      migration!(set_user_current_home_member),
    ]
  });

  let overrides = FetchUserOverrides::new(&*client)
    .await
    .unwrap()
    .run(session.user_id)
    .await
    .unwrap();

  for migration in &*MIGRATIONS {
    if let Some(value) = overrides.get(&format!("migration:{}", migration.name))
      && value == "skip"
    {
      debug!(
        user_id = %session.user_id,
        name = migration.name,
        "skipping migration due to user override"
      );
      continue;
    }

    let rows_modified = (migration.function)(session, client).await;
    debug!(
      user_id = %session.user_id,
      name = migration.name,
      rows_modified,
      "migration applied"
    );
  }
  debug!(user_id = %session.user_id, "applied {} migrations", MIGRATIONS.len());
}

async fn add_user_characters(session: &Session, client: &mut Client) -> u64 {
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_characters (user_id, character_id, intimacy)
      select $1, m.id::bigint, 0
      from unnest($2::bigint[]) as m(id)
      where not exists (
        select 1 from user_characters uc
        where uc.user_id = $1 and uc.character_id = m.id::bigint
      )
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  let character_ids: Vec<i64> = get_master_manager()
    .get_master("character")
    .iter()
    .map(|data| data.get("id").unwrap().as_str().unwrap().parse::<i64>().unwrap())
    .collect();
  client
    .execute(&statement, &[&session.user_id, &character_ids])
    .await
    .context("failed to execute query")
    .unwrap()
}

async fn add_user_members(session: &Session, client: &mut Client) -> u64 {
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_members (user_id, member_id, xp, promotion_level)
      select
        $1,
        m.id::bigint,
        (floor(random() * 50000) + 5000)::int as xp,
        (floor(random() * 6))::int
      from unnest($2::bigint[]) as m(id)
      where not exists (
        select 1 from user_members um
        where um.user_id = $1 and um.member_id = m.id::bigint
      )
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  let member_ids: Vec<i64> = get_master_manager()
    .get_master("member")
    .iter()
    .map(|data| data.get("id").unwrap().as_str().unwrap().parse::<i64>().unwrap())
    .collect();
  client
    .execute(&statement, &[&session.user_id, &member_ids])
    .await
    .context("failed to execute query")
    .unwrap()
}

async fn add_user_members_reserve(session: &Session, client: &mut Client) -> u64 {
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_members_reserve (user_id, member_id)
      select $1, m.id::bigint
      from unnest($2::bigint[]) as m(id)
             cross join lateral generate_series(1, floor(random() * 10 + 1)::int) as s
      where not exists (select 1
                        from user_members_reserve um
                        where um.user_id = $1
                          and um.member_id = m.id::bigint)
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  let member_ids: Vec<i64> = get_master_manager()
    .get_master("member")
    .iter()
    .map(|data| data.get("id").unwrap().as_str().unwrap().parse::<i64>().unwrap())
    .collect();
  client
    .execute(&statement, &[&session.user_id, &member_ids])
    .await
    .context("failed to execute query")
    .unwrap()
}

/// Create 8 default parties if none exist, for each new party create 5 default forms.
async fn add_user_parties(session: &Session, client: &mut Client) -> u64 {
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      with inserted_parties as (
        insert into user_parties (user_id, party_id, name)
        select
          $1,
          p.party_id,
          'Axel' || p.party_id::text
        from generate_series(1, 8) as p(party_id)
        where not exists (
          select 1 from user_parties up
          where up.user_id = $1 and up.party_id = p.party_id
        )
        returning user_id, party_id
      )
      insert into user_party_forms (user_id, party_id, form_id, main_member_id, sub1_member_id, sub2_member_id, weapon_id, accessory_id)
      select
        ip.user_id,
        ip.party_id,
        f.form_id,
        1001100, -- main_member_id
        0,       -- sub1_member_id
        0,       -- sub2_member_id
        0,       -- weapon_id
        0        -- accessory_id
      from inserted_parties ip
      join generate_series(1, 5) as f(form_id) on true
      where not exists (
        select 1 from user_party_forms upf
        where upf.user_id = ip.user_id and upf.party_id = ip.party_id and upf.form_id = f.form_id
      )
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  client
    .execute(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")
    .unwrap()
}

/// Give some default items if none exist.
async fn add_user_items(session: &Session, client: &mut Client) -> u64 {
  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_items (user_id, item_type, item_id, quantity)
      select $1 as user_id,
             v.item_type,
             v.item_id,
             v.quantity
      from (select item_type, item_id, sum(quantity) as quantity
            from (values (18::bigint, 1::bigint, 30000::integer),
                         (18::bigint, 2::bigint, 20000::integer),
                         (18::bigint, 3::bigint, 10000::integer)) as t(item_type, item_id, quantity)
            group by item_type, item_id) as v
      on conflict (user_id, item_type, item_id) do nothing
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  client
    .execute(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")
    .unwrap()
}

async fn add_user_equipment(session: &Session, client: &mut Client) -> u64 {
  // Check if user has any equipment items
  #[rustfmt::skip]
  let check_statement = client
    .prepare(/* language=postgresql */ r#"
      select exists(
        select 1 from user_items_equipment
        where user_id = $1
      )
    "#)
    .await
    .context("failed to prepare check statement")
    .unwrap();

  let has_items: bool = client
    .query_one(&check_statement, &[&session.user_id])
    .await
    .context("failed to check for existing items")
    .unwrap()
    .get(0);

  if has_items {
    0
  } else {
    let items = get_master_manager()
      .get_master("equip_weapon_details")
      .into_iter()
      .filter_map(|data| {
        let item_id = data["item_id"].as_str().unwrap().parse::<i64>().unwrap();
        let level = data["lv"].as_str().unwrap().parse::<i32>().unwrap();
        if matches!(level, 0 | 5) {
          Some((
            session.user_id,
            Into::<i32>::into(RemoteDataItemType::Weapon) as i64,
            item_id,
            level,
          ))
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    let transaction = client
      .transaction()
      .await
      .context("failed to start transaction")
      .unwrap();

    #[rustfmt::skip]
    let copy_statement = transaction
      .prepare(/* language=postgresql */ r#"
        copy user_items_equipment (user_id, item_type, item_id, level)
        from stdin with (format binary)
      "#)
      .await
      .context("failed to prepare COPY statement")
      .unwrap();

    let sink = transaction
      .copy_in(&copy_statement)
      .await
      .context("failed to start COPY")
      .unwrap();

    let writer = tokio_postgres::binary_copy::BinaryCopyInWriter::new(
      sink,
      &[
        tokio_postgres::types::Type::INT8, // user_id
        tokio_postgres::types::Type::INT8, // item_type
        tokio_postgres::types::Type::INT8, // item_id
        tokio_postgres::types::Type::INT4, // level
      ],
    );

    tokio::pin!(writer);

    for item in &items {
      writer
        .as_mut()
        .write(&[&item.0, &item.1, &item.2, &item.3])
        .await
        .context("failed to write row")
        .unwrap();
    }

    let rows_inserted = writer.finish().await.context("failed to finish COPY").unwrap();

    transaction
      .commit()
      .await
      .context("failed to commit transaction")
      .unwrap();

    rows_inserted
  }
}

async fn add_user_character_special_skills(session: &Session, client: &mut Client) -> u64 {
  // Create missing character skills, read 'sp_skill' master and filter by character_id for each character user has
  let skills_by_character: HashMap<i64, Vec<&serde_json::Value>> = get_master_manager()
    .get_master("skill_sp")
    .iter()
    .fold(HashMap::new(), |mut acc, skill| {
      let character_id: i64 = skill["character_id"].as_str().unwrap().parse::<i64>().unwrap();
      acc.entry(character_id).or_default().push(skill);
      acc
    });
  trace!(?skills_by_character, "mapped skills by character");

  // build (user_id, character_id, skill_id, level) tuples and insert missing ones
  let skills = {
    #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select character_id
          from user_characters
          where user_id = $1
        "#)
        .await
        .context("failed to prepare statement")
        .unwrap();
    let rows = client
      .query(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")
      .unwrap();
    rows
      .iter()
      .flat_map(|row| {
        let character_id: i64 = row.get(0);
        skills_by_character
          .get(&character_id)
          .into_iter()
          .flatten()
          .map(move |skill| {
            let skill_id: i64 = skill["skill_id"].as_str().unwrap().parse::<i64>().unwrap();
            (session.user_id, character_id, skill_id, 1i32)
          })
      })
      .collect::<Vec<_>>()
  };

  // remove existing skills
  let skills = {
    #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select character_id, skill_id
          from user_character_special_skills
          where user_id = $1
        "#)
        .await
        .context("failed to prepare statement")
        .unwrap();
    let rows = client
      .query(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")
      .unwrap();
    let existing_skills: HashMap<(i64, i64), ()> = rows
      .iter()
      .map(|row| {
        let character_id: i64 = row.get(0);
        let skill_id: i64 = row.get(1);
        ((character_id, skill_id), ())
      })
      .collect();
    skills
      .into_iter()
      .filter(|skill| !existing_skills.contains_key(&(skill.1, skill.2)))
      .collect::<Vec<_>>()
  };

  // use copy from
  if skills.is_empty() {
    0
  } else {
    let transaction = client
      .transaction()
      .await
      .context("failed to start transaction")
      .unwrap();

    #[rustfmt::skip]
      let copy_statement = transaction
        .prepare(/* language=postgresql */ r#"
          copy user_character_special_skills (user_id, character_id, skill_id, level)
          from stdin with (format binary)
        "#)
        .await
        .context("failed to prepare COPY statement")
        .unwrap();

    let sink = transaction
      .copy_in(&copy_statement)
      .await
      .context("failed to start COPY")
      .unwrap();

    let writer = tokio_postgres::binary_copy::BinaryCopyInWriter::new(
      sink,
      &[
        tokio_postgres::types::Type::INT8, // user_id
        tokio_postgres::types::Type::INT8, // character_id
        tokio_postgres::types::Type::INT8, // skill_id
        tokio_postgres::types::Type::INT4, // level
      ],
    );

    tokio::pin!(writer);

    for skill in &skills {
      writer
        .as_mut()
        .write(&[&skill.0, &skill.1, &skill.2, &skill.3])
        .await
        .context("failed to write row")
        .unwrap();
    }

    let rows_inserted = writer.finish().await.context("failed to finish COPY").unwrap();

    transaction
      .commit()
      .await
      .context("failed to commit transaction")
      .unwrap();

    rows_inserted
  }
}

async fn add_user_party_form_skills(session: &Session, client: &mut Client) -> u64 {
  let skills_by_character: HashMap<i64, Vec<&serde_json::Value>> = get_master_manager()
    .get_master("skill_sp")
    .iter()
    .fold(HashMap::new(), |mut acc, skill| {
      let character_id: i64 = skill["character_id"].as_str().unwrap().parse::<i64>().unwrap();
      acc.entry(character_id).or_default().push(skill);
      acc
    });
  trace!(?skills_by_character, "mapped skills by character");

  let mut transaction = client
    .transaction()
    .await
    .context("failed to start transaction")
    .unwrap();
  // XXX: This does not check for inconsistent skill / main character combinations
  #[rustfmt::skip]
    let statement = transaction
      .prepare(/* language=postgresql */ r#"
        select user_id, party_id, form_id, main_member_id
        from user_party_forms
        where user_id = $1
          and special_skill_id is null
        for update
      "#)
      .await
      .context("failed to prepare statement")
      .unwrap();

  let rows = transaction
    .query(&statement, &[&session.user_id])
    .await
    .context("failed to execute query")
    .unwrap();
  debug!("found {} party_forms with null special_skill_id", rows.len());
  let mut updated_count = 0;
  for row in rows.iter() {
    let user_id: i64 = row.get(0);
    let party_id: i64 = row.get(1);
    let form_id: i64 = row.get(2);
    let main_member_id: i64 = row.get(3);

    let character_id = MemberPrototype::load_from_id(main_member_id).character_id;

    // get some default skill for main_member_id
    let default_skill_id = {
      let skills = skills_by_character.get(&character_id);
      if let Some(skills) = skills {
        // TODO: Not first?
        let skill = skills.first().unwrap();
        skill["skill_id"].as_str().unwrap().parse::<i64>().unwrap()
      } else {
        warn!(
          ?user_id,
          ?party_id,
          ?form_id,
          ?main_member_id,
          ?character_id,
          "no skills found, skipping special_skill_id update"
        );
        continue;
      }
    };

    #[rustfmt::skip]
      let update_statement = transaction
        .prepare(/* language=postgresql */ r#"
          update user_party_forms
          set special_skill_id = $4
          where user_id = $1
            and party_id = $2
            and form_id = $3
        "#)
        .await
        .context("failed to prepare update statement")
        .unwrap();
    transaction
      .execute(&update_statement, &[&user_id, &party_id, &form_id, &default_skill_id])
      .await
      .context("failed to execute update")
      .unwrap();
    updated_count += 1;
    debug!(
      ?user_id,
      ?party_id,
      ?form_id,
      ?main_member_id,
      ?default_skill_id,
      "updated special_skill_id for party_form"
    );
  }

  transaction
    .commit()
    .await
    .context("failed to commit transaction")
    .unwrap();
  updated_count
}

async fn add_user_member_skills(session: &Session, client: &mut Client) -> u64 {
  // For each user member, get MemberPrototype and insert missing active skills
  let fetch_members = FetchUserMembers::new(&*client).await.unwrap();
  let members = fetch_members.run(session.user_id).await.unwrap();

  let existing_skills: HashSet<(i64, i64)> = {
    #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select member_id, skill_id
          from user_member_skills
          where user_id = $1
        "#)
        .await
        .context("failed to prepare statement")
        .unwrap();
    let rows = client
      .query(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")
      .unwrap();
    let mut set = HashSet::new();
    for row in rows.iter() {
      let member_id: i64 = row.get("member_id");
      let skill_id: i64 = row.get("skill_id");
      set.insert((member_id, skill_id));
    }
    set
  };

  let mut total_inserted = 0;
  // COPY FROM
  let transaction = client
    .transaction()
    .await
    .context("failed to start transaction")
    .unwrap();
  if members.is_empty() {
    trace!("no members found for user, skipping member skills migration");
  } else {
    #[rustfmt::skip]
      let copy_statement = transaction
        .prepare(/* language=postgresql */ r#"
          copy user_member_skills (user_id, member_id, skill_id, level)
          from stdin with (format binary)
        "#)
        .await
        .context("failed to prepare COPY statement")
        .unwrap();

    let sink = transaction
      .copy_in(&copy_statement)
      .await
      .context("failed to start COPY")
      .unwrap();

    let writer = tokio_postgres::binary_copy::BinaryCopyInWriter::new(
      sink,
      &[
        tokio_postgres::types::Type::INT8, // user_id
        tokio_postgres::types::Type::INT8, // member_id
        tokio_postgres::types::Type::INT8, // skill_id
        tokio_postgres::types::Type::INT4, // level
      ],
    );

    tokio::pin!(writer);

    for member in members.iter() {
      // .flatten() to skip None skills
      for skill in member.prototype.active_skills.iter().flatten() {
        if !existing_skills.contains(&(member.id as i64, skill.id)) {
          writer
            .as_mut()
            .write(&[&session.user_id, &(member.id as i64), &skill.id, &1i32])
            .await
            .context("failed to write row")
            .unwrap();
          total_inserted += 1;
        }
      }
    }

    writer.finish().await.context("failed to finish COPY").unwrap();
  }
  transaction
    .commit()
    .await
    .context("failed to commit transaction")
    .unwrap();

  total_inserted
}

async fn add_user_home_illustrations(session: &Session, client: &mut Client) -> u64 {
  // XXX: First member conveniently has same ID as its default illustration
  let member_id: i64 = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select member_id
        from user_members
        where user_id = $1
        order by member_id
        limit 1
      "#)
      .await
      .context("failed to prepare statement")
      .unwrap();
    let row = client
      .query_one(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")
      .unwrap();
    row.get(0)
  };

  #[rustfmt::skip]
  let statement = client
    .prepare(/* language=postgresql */ r#"
      insert into user_home_illustrations (user_id, slot, member_id)
      select $1, h.slot, h.member_id
      from (values (1::int, $2::bigint),
                   (2::int, null),
                   (3::int, null),
                   (4::int, null),
                   (5::int, null)) as h(slot, member_id)
      where not exists (select 1
                        from user_home_illustrations uhi
                        where uhi.user_id = $1
                          and uhi.slot = h.slot)
    "#)
    .await
    .context("failed to prepare statement")
    .unwrap();
  client
    .execute(&statement, &[&session.user_id, &member_id])
    .await
    .context("failed to execute query")
    .unwrap()
}

async fn set_user_current_home_member(session: &Session, client: &mut Client) -> u64 {
  // If users.home_current_illustration_id is null, set it to first slot in user_home_illustrations
  let current_member_id: Option<i64> = {
    #[rustfmt::skip]
    let statement = client
      .prepare(/* language=postgresql */ r#"
        select home_current_illustration_id
        from users
        where id = $1
        for update
      "#)
      .await
      .context("failed to prepare statement")
      .unwrap();
    let row = client
      .query_one(&statement, &[&session.user_id])
      .await
      .context("failed to execute query")
      .unwrap();
    row.get(0)
  };

  if current_member_id.is_some() {
    // XXX: Ideally we should check whether member also exists in user_members invalidate it if not
    0
  } else {
    let member_id: i64 = {
      #[rustfmt::skip]
      let statement = client
        .prepare(/* language=postgresql */ r#"
          select member_id
          from user_home_illustrations
          where user_id = $1 and member_id is not null
          order by slot
          limit 1
        "#)
        .await
        .context("failed to prepare statement")
        .unwrap();
      let row = client
        .query_one(&statement, &[&session.user_id])
        .await
        .context("failed to execute query")
        .unwrap();
      row.get(0)
    };

    #[rustfmt::skip]
    let update_statement = client
      .prepare(/* language=postgresql */ r#"
        update users
        set home_current_illustration_id = $2
        where id = $1
      "#)
      .await
      .context("failed to prepare update statement")
      .unwrap();
    client
      .execute(&update_statement, &[&session.user_id, &member_id])
      .await
      .context("failed to execute update")
      .unwrap()
  }
}
