use crate::database::QueryExecutor;
use crate::user::id::UserId;
use std::collections::HashMap;
use tokio_postgres::Statement;

pub struct FetchUserOverrides<'a> {
  executor: QueryExecutor<'a>,
  statement: Statement,
}

impl<'a> FetchUserOverrides<'a> {
  pub async fn new(executor: impl Into<QueryExecutor<'a>>) -> anyhow::Result<Self> {
    let executor = executor.into();
    Ok(Self {
      #[rustfmt::skip]
      statement: executor.prepare(/* language=postgresql */ r#"
        select key, override
        from user_overrides
        where user_id = $1
      "#).await?,
      executor,
    })
  }

  pub async fn run(&self, user_id: UserId) -> anyhow::Result<HashMap<String, String>> {
    let rows = self.executor.client().query(&self.statement, &[&user_id]).await?;
    let mut overrides = HashMap::new();
    for row in rows {
      let key: String = row.get("key");
      let override_value: String = row.get("override");
      overrides.insert(key, override_value);
    }
    Ok(overrides)
  }
}
