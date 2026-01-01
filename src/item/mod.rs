use crate::api::{RemoteData, RemoteDataCommand, RemoteDataItemType};
use crate::blob::IntoRemoteData;
use crate::database::QueryExecutor;
use crate::user::id::UserId;
use tokio_postgres::Statement;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct CountedItem {
  pub item: ItemReference,
  pub quantity: i32,
}

impl IntoRemoteData for CountedItem {
  fn into_remote_data(self) -> Vec<RemoteData> {
    vec![
      // TODO: This is a hack, UserParamAdd combines amount with existing amount.
      //  UserParamUpdate does not create new items. We need to either track state what was sent
      //  to the client, or delete item entirely and re-add it.
      // RemoteData {
      //   cmd: RemoteDataCommand::UserParamDelete as i32,
      //   uid: None,
      //   item_type: self.item.item_type.into(),
      //   item_id: self.item.item_id,
      //   // UserParamDelete decreases by this amount, and clamps at zero
      //   item_num: i32::MAX,
      //   uniqid: 0,
      //   lv: 0,
      //   tag: String::from("-"),
      //   member_parameter: None,
      //   character_parameter: None,
      //   is_trial: None,
      // },
      RemoteData {
        cmd: RemoteDataCommand::UserParamAdd as i32,
        uid: None,
        item_type: self.item.item_type.into(),
        item_id: self.item.item_id,
        item_num: self.quantity,
        uniqid: 12345,
        lv: 1,
        tag: String::from(""),
        member_parameter: None,
        character_parameter: None,
        is_trial: Some(false),
      },
    ]
  }
}

impl ItemReference {
  pub fn into_counted(self, quantity: i32) -> CountedItem {
    CountedItem { item: self, quantity }
  }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct ItemReference {
  pub item_type: RemoteDataItemType,
  pub item_id: i64,
}

pub trait IntoItemReference {
  fn into_item_reference(self) -> ItemReference;
}

impl IntoItemReference for (RemoteDataItemType, i64) {
  fn into_item_reference(self) -> ItemReference {
    ItemReference {
      item_type: self.0,
      item_id: self.1,
    }
  }
}

pub struct UpdateItemCountBy<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> UpdateItemCountBy<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        insert into user_items (user_id, item_type, item_id, quantity)
        values ($1, $2, $3, $4)
        on conflict (user_id, item_type, item_id)
          do update
          set quantity = user_items.quantity + excluded.quantity
        -- Check that we don't go below zero
        where user_items.quantity + excluded.quantity >= 0
        returning quantity
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId, item: impl IntoItemReference, delta: i32) -> anyhow::Result<CountedItem> {
    let item = item.into_item_reference();
    let item_type: i32 = item.item_type.into();
    let item_id: i64 = item.item_id;
    let row = self
      .executor
      .client()
      .query_one(&self.statement, &[&user_id, &(item_type as i64), &item_id, &delta])
      .await?;
    let new_quantity: i32 = row.get(0);
    Ok(item.into_counted(new_quantity))
  }
}
