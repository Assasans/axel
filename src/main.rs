#![allow(non_snake_case, unused_variables)]

pub mod api;
pub mod api_server;
pub mod blob;
pub mod bool_as_int;
pub mod build_info;
pub mod call;
pub mod client_ip;
pub mod database;
pub mod master;
pub mod normalize_path;
pub mod notification;
pub mod request_logging;
pub mod serde_compat;
pub mod settings;
pub mod static_server;
pub mod string_as_base64;
pub mod user;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::stdout;
use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clap::Parser;
use tokio::join;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::api::master_all::get_masters;
use crate::api::{RemoteData, RemoteDataCommand, RemoteDataItemType};
use crate::database::create_pool;
use crate::settings::Settings;
use crate::user::id::UserId;
use crate::user::session::Session;

#[derive(Parser, Debug)]
pub struct Args {}

pub struct AppState {
  pub args: Args,
  pub settings: Settings,
  pub sessions: Mutex<HashMap<UserId, Arc<Session>>>,
  pub pool: deadpool_postgres::Pool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  #[rustfmt::skip]
  let env_filter = EnvFilter::builder().parse_lossy(
    env::var("RUST_LOG")
      .as_deref()
      .unwrap_or("info"),
  );

  let file_appender = tracing_appender::rolling::hourly("logs", "rolling.log");
  let (non_blocking_file, _file_guard) = tracing_appender::non_blocking(file_appender);
  let (non_blocking_stdout, _stdout_guard) = tracing_appender::non_blocking(stdout());
  let console = tracing_subscriber::fmt::layer().with_writer(non_blocking_stdout);
  #[rustfmt::skip]
  let file = tracing_subscriber::fmt::layer()
    .json()
    .with_ansi(false)
    .with_writer(non_blocking_file);

  #[rustfmt::skip]
  tracing_subscriber::registry()
    .with(env_filter)
    .with(console)
    .with(file)
    .init();

  info!("May There Be a Blessing on This Wonderful Server");

  let args = Args::parse();
  let settings = Settings::new().unwrap_or_else(|err| {
    panic!("Failed to load settings: {err}");
  });

  let pool = create_pool(&settings.database).await.unwrap();
  let client = pool.get().await?;
  let statement = client
    .prepare(/* language=postgresql */ "select count(*) from users")
    .await?;
  let result = client.query(&statement, &[]).await?;
  info!(
    "database connection established successfully, {} users found",
    result[0].get::<_, i64>(0)
  );

  let state = AppState {
    args,
    settings,
    sessions: Mutex::new(HashMap::new()),
    pool,
  };
  let state = Arc::new(state);

  // initialize lazies
  get_masters().await;

  let remote_data = include_str!("./login-remotedata.json");
  let remote_data: Vec<RemoteData> = serde_json::from_str(remote_data).unwrap();
  for item in &remote_data {
    if item.cmd == RemoteDataCommand::UserParamAdd as i32 {
      let item_type = RemoteDataItemType::from(item.item_type);

      // if let Some(member_parameter) = &item.member_parameter {
      //   println!(
      //     "AddMember::new({member_parameter:?}, {:?}).into_remote_data(),",
      //     item.tag
      //   );
      // }
      // if let Some(character_parameter) = &item.character_parameter {
      //   println!(
      //     "AddCharacter::new({:?}, {character_parameter:?}).into_remote_data(),",
      //     item.uniqid,
      //   );
      // }
      // if item_type == RemoteDataItemType::MemberCostume {
      //   println!(
      //     "AddMemberCostume::new({}, {}).into_remote_data(),",
      //     item.uniqid, item.item_id,
      //   );
      // }
      // if item_type == RemoteDataItemType::MemberBackground {
      //   println!(
      //     "AddMemberBackground::new({}, {}).into_remote_data(),",
      //     item.uniqid, item.item_id,
      //   );
      // }

      // info!(
      //   "adding remote data item: cmd={}, item_type={:?}, item_id={}, item_num={}, uniqid={}, lv={}, tag={}, member_parameter={:?}, character_parameter={:?}. is_trial={:?}",
      //   item.cmd, item_type, item.item_id, item.item_num, item.uniqid, item.lv, item.tag,
      //   item.member_parameter, item.character_parameter, item.is_trial
      // );
      // if let RemoteDataItemType::Another(kind) = item_type {
      //   warn!(
      //     "unknown remote data item type: {:?} (cmd={}, item_id={}, item_num={}, uniqid={}, lv={}, tag={})",
      //     kind, item.cmd, item.item_id, item.item_num, item.uniqid, item.lv, item.tag
      //   );
      // }
      //
      // if item.character_parameter.is_some() && item_type != RemoteDataItemType::Character {
      //   warn!(
      //     "remote data item with character parameter but item type is not Character: {:?} (cmd={}, item_id={}, item_num={}, uniqid={}, lv={}, tag={})",
      //     item.character_parameter, item.cmd, item.item_id, item.item_num, item.uniqid, item.lv, item.tag
      //   );
      // }
      // if item.member_parameter.is_some() && item_type != RemoteDataItemType::Member {
      //   warn!(
      //     "remote data item with member parameter but item type is not Member: {:?} (cmd={}, item_id={}, item_num={}, uniqid={}, lv={}, tag={})",
      //     item.member_parameter, item.cmd, item.item_id, item.item_num, item.uniqid, item.lv, item.tag
      //   );
      // }
      //
      // if item.tag.is_empty() {
      //   warn!(
      //     "remote data item with empty tag: (cmd={}, item_id={}, item_num={}, uniqid={}, lv={})",
      //     item.cmd, item.item_id, item.item_num, item.uniqid, item.lv
      //   );
      // }
      //
      // if item_type == RemoteDataItemType::MemberCostume && item.tag != "useflag:0" {
      //   warn!(
      //     "remote data item with MemberCostume type but tag is not 'useflag:0': (cmd={}, item_id={}, item_num={}, uniqid={}, lv={}, tag={})",
      //     item.cmd, item.item_id, item.item_num, item.uniqid, item.lv, item.tag
      //   );
      // }
    }
  }

  let (static_result, api_result) = join!(static_server::start(state.clone()), api_server::start(state.clone()));
  static_result.unwrap();
  api_result.unwrap();

  Ok(())
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let error = self.0;
    tracing::error!(?error, "api error");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)).into_response()
  }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}
