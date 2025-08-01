#![allow(non_snake_case, unused_variables)]

pub mod api;
pub mod api_server;
pub mod bool_as_int;
pub mod call;
pub mod client_ip;
pub mod master;
pub mod normalize_path;
pub mod session;
pub mod settings;
pub mod static_server;
pub mod string_as_base64;

use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::{env, str};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clap::Parser;
use tokio::join;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::api::master_all::get_masters;
use crate::session::{Session, UserId};
use crate::settings::Settings;

#[derive(Parser, Debug)]
struct Args {
  /// Enable proxy mode - save all requests and responses to `proxied/` directory.
  /// Disables API endpoints.
  #[arg(long, default_value_t = false)]
  proxy: bool,
}

pub struct AppState {
  proxy: bool,
  settings: Settings,
  sessions: Mutex<HashMap<UserId, Arc<Session>>>,
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

  if args.proxy {
    info!("proxy mode is enabled");
  }

  // initialize lazies
  get_masters().await;

  let state = AppState {
    proxy: args.proxy,
    settings,
    sessions: Mutex::new(HashMap::new()),
  };
  let state = Arc::new(state);

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
    tracing::error!(%error, "api error");

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
