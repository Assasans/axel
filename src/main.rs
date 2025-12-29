#![allow(non_snake_case, unused_variables)]

pub mod api;
pub mod api_server;
pub mod blob;
pub mod bool_as_int;
pub mod build_info;
pub mod call;
pub mod client_ip;
pub mod database;
pub mod extractor;
pub mod handler;
pub mod impl_handler;
pub mod item;
pub mod level;
pub mod master;
pub mod member;
pub mod normalize_path;
pub mod notification;
pub mod params_deserializer;
pub mod request_logging;
pub mod router;
pub mod serde_compat;
pub mod settings;
pub mod static_server;
pub mod string_as_base64;
pub mod user;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io::stdout;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::api::master_all::{MASTER_MANAGER, MasterManager, get_masters};
use crate::api::{RemoteDataCommand, RemoteDataItemType};
use crate::database::create_pool;
use crate::settings::Settings;
use crate::user::id::UserId;
use crate::user::session::Session;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clap::Parser;
use deadpool_postgres::PoolError;
use tokio::join;
use tracing::info;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[derive(Parser, Debug)]
pub struct Args {}

pub struct AppState {
  pub args: Args,
  pub settings: Settings,
  pub sessions: Mutex<HashMap<UserId, Arc<Session>>>,
  pub pool: deadpool_postgres::Pool,
}

pub struct AppPoolError(PoolError);

impl From<PoolError> for AppPoolError {
  fn from(err: PoolError) -> Self {
    Self(err)
  }
}

impl Debug for AppPoolError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for AppPoolError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Deref for AppPoolError {
  type Target = PoolError;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Error for AppPoolError {}

impl AppState {
  pub async fn get_database_client(&self) -> Result<deadpool_postgres::Client, AppPoolError> {
    self.pool.get().await.map_err(AppPoolError::from)
  }
}

impl IntoResponse for AppPoolError {
  fn into_response(self) -> Response {
    let error = self;
    tracing::error!(?error, "database pool error");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)).into_response()
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  #[rustfmt::skip]
  let console_filter = EnvFilter::builder().parse_lossy(
    env::var("RUST_LOG")
      .as_deref()
      .unwrap_or("info"),
  );

  #[rustfmt::skip]
  let file_filter = EnvFilter::builder().parse_lossy(
    env::var("RUST_LOG_FILE")
      .as_deref()
      .unwrap_or("info,axel=debug"),
  );

  let file_appender = tracing_appender::rolling::hourly("logs", "rolling.log");
  let (non_blocking_file, _file_guard) = tracing_appender::non_blocking(file_appender);
  let (non_blocking_stdout, _stdout_guard) = tracing_appender::non_blocking(stdout());

  let console = tracing_subscriber::fmt::layer()
    .with_writer(non_blocking_stdout)
    .with_filter(console_filter)
    // Drop spans named "request"
    .with_filter(FilterFn::new(|meta| !(meta.is_span() && meta.name() == "request")));

  let file = tracing_subscriber::fmt::layer()
    .json()
    .with_ansi(false)
    .with_writer(non_blocking_file)
    .with_filter(file_filter);

  #[rustfmt::skip]
  tracing_subscriber::registry()
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
  let masters = get_masters().await;
  MASTER_MANAGER.get_or_init(|| MasterManager::new(masters));

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
