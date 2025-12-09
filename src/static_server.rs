use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::response::Builder;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, ServiceExt, extract, middleware};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::net::TcpListener;
use tower::{Layer, Service};
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};
use url::Url;

use crate::client_ip::add_client_ip;
use crate::normalize_path::normalize_path;
use crate::request_logging::log_requests;
use crate::{AppError, AppState};

trait RouterExt {
  fn serve_dir_with_fallback(self, path: &str, local_root: &Path, fallback_url: Option<Url>) -> Self;
}

impl<S> RouterExt for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn serve_dir_with_fallback(self, path: &str, local_root: &Path, fallback_url: Option<Url>) -> Self {
    let serve_dir = ServeDir::new(path);
    if let Some(url) = fallback_url {
      self.nest_service(path, serve_dir.fallback(ServeRemoteResource::new(url)))
    } else {
      self.nest_service(path, serve_dir)
    }
  }
}

pub async fn start(state: Arc<AppState>) -> io::Result<()> {
  let settings = &state.settings.static_server;
  let app = Router::new()
    .route("/", get(get_root_friendly))
    .route("/public.pem", get(get_public_key))
    .route("/versions/{version}", get(get_version))
    .route("/webview/EN/data/json/news.json", get(get_news))
    .serve_dir_with_fallback(
      "/bundles",
      &settings.resources_root.join("bundles/"),
      settings.upstream_url.as_ref().map(|url| url.join("bundles/").unwrap()),
    )
    .serve_dir_with_fallback(
      "/banners",
      &settings.resources_root.join("banners/"),
      settings.upstream_url.as_ref().map(|url| url.join("banners/").unwrap()),
    )
    .serve_dir_with_fallback(
      "/webview",
      &settings.resources_root.join("webview/"),
      settings.upstream_url.as_ref().map(|url| url.join("webview/").unwrap()),
    )
    .layer(log_requests())
    .layer(middleware::from_fn(add_client_ip))
    .with_state(state.clone());
  let middleware = tower::util::MapRequestLayer::new(normalize_path);
  let app = middleware.layer(app);

  let listener = TcpListener::bind(settings.bind_address).await?;
  info!(
    "static server started at {:?} -> {}",
    listener.local_addr()?,
    settings.public_url
  );
  axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await
}

#[derive(Debug, Clone)]
struct ServeRemoteResource {
  remote_url: Url,
}

impl ServeRemoteResource {
  pub fn new(remote_url: Url) -> Self {
    Self { remote_url }
  }
}

impl Service<Request> for ServeRemoteResource {
  type Response = Response;
  type Error = Infallible;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static>>;

  fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request) -> Self::Future {
    let path = request.uri().path()[1..].to_owned();
    let remote_url = self.remote_url.join(&path).unwrap();
    debug!("get bundle remote: {} -> {}", path, remote_url);

    Box::pin(async move {
      let client = Client::new();
      let response = client.get(remote_url).send().await.unwrap();

      let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
      if !status.is_success() {
        warn!("failed to fetch remote resource: {}, status: {}", path, status);
      }

      let headers = response.headers().clone();
      let stream = response.bytes_stream();
      let body = Body::from_stream(stream);

      let mut builder = Builder::new().status(status);
      for (name, value) in headers {
        let name = match name {
          Some(name) => name,
          None => continue,
        };

        builder = builder.header(name, value);
      }

      Ok(builder.body(body).unwrap_or_else(|_| {
        Response::builder()
          .status(StatusCode::INTERNAL_SERVER_ERROR)
          .body(Body::empty())
          .unwrap()
      }))
    })
  }
}

#[derive(Serialize, Deserialize)]
pub struct NewsItem {
  pub title: String,
  pub category: NewsCategory,
  pub platform: NewsPlatform,
  pub thumbnail: Option<String>,
  pub banner: Option<String>,
  pub url: String,
  pub priority: Option<i64>,
  pub start_at: i64,
  pub end_at: i64,
  pub date: String,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NewsCategory {
  AnnouncementReportError = 1,
  Event = 2,
  EventRecruit = 3,
  EventCampaign = 4,
  AnnouncementNotice = 5,
  Update = 6,
  UpdateGameMaintenance = 7,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NewsPlatform {
  All = 0,
  Apple = 1,
  Android = 2,
  Pc = 3,
}

async fn get_news() -> axum::response::Result<impl IntoResponse, AppError> {
  info!("get news");

  let time = Utc::now();
  let news = vec![
    NewsItem {
      title: "May There Be a Blessing on This Wonderful Server!".to_string(),
      category: NewsCategory::AnnouncementNotice,
      platform: NewsPlatform::All,
      thumbnail: None,
      banner: Some("https://smb.assasans.dev/konofd/story-images/1013104.png".to_string()),
      url: "./detail/00-axel.html".to_string(),
      priority: Some(1),
      start_at: (time - chrono::Duration::days(7)).timestamp(),
      end_at: (time + chrono::Duration::days(7)).timestamp(),
      date: "2025-07-28".to_owned(),
    },
    NewsItem {
      title: "May There Be a Database for This Wonderful Server!".to_string(),
      category: NewsCategory::AnnouncementNotice,
      platform: NewsPlatform::All,
      thumbnail: None,
      banner: Some("https://smb.assasans.dev/konofd/story-images/1063100.png".to_string()),
      url: "./detail/01-axel.html".to_string(),
      priority: Some(1),
      start_at: (time - chrono::Duration::days(6)).timestamp(),
      end_at: (time + chrono::Duration::days(7)).timestamp(),
      date: "2025-12-09".to_owned(),
    },
  ];

  Ok(Json(news))
}

fn format_url(url: &Url, suffix: Option<&str>) -> Result<String, &'static str> {
  if url.scheme() != "https" {
    return Err("URL scheme must be https");
  }

  // Ensure trailing slash
  let mut base = url.clone();
  if !base.path().ends_with('/') {
    base.set_path(&format!("{}/", base.path()));
  }

  let new_url = if let Some(suffix) = suffix {
    base.join(suffix).unwrap()
  } else {
    base
  };
  let without_scheme = new_url.as_str().trim_start_matches("https://").to_string();

  Ok(without_scheme)
}

async fn get_version(
  State(state): State<Arc<AppState>>,
  extract::Path(version): extract::Path<String>,
) -> axum::response::Result<impl IntoResponse, AppError> {
  info!("get version info: {}", version);

  let name = env!("CARGO_PKG_NAME");
  let version = env!("CARGO_PKG_VERSION");

  Ok(Json(GetVersion {
    app_version: format!("4.11.5/{name}-{version}"),
    asset_version: "2025012110".to_string(),
    api_url: format_url(&state.settings.api_server.public_url, None).unwrap(),
    asset_url: format_url(&state.settings.static_server.public_url, Some("bundles/4.11.6/")).unwrap(),
    webview_url: format_url(&state.settings.static_server.public_url, Some("webview/")).unwrap(),
    banner_url: format_url(&state.settings.static_server.public_url, Some("banners/")).unwrap(),
    inquiry_url: "inquiry.sesisoft.com/".to_string(),
    enable_review: "false".to_string(),
  }))
}

#[derive(Debug, Serialize)]
pub struct GetVersion {
  pub app_version: String,
  pub asset_version: String,
  pub api_url: String,
  pub asset_url: String,
  pub webview_url: String,
  pub banner_url: String,
  pub inquiry_url: String,
  pub enable_review: String,
}

async fn get_root_friendly() -> axum::response::Result<impl IntoResponse, AppError> {
  let name = env!("CARGO_PKG_NAME");
  let version = env!("CARGO_PKG_VERSION");

  Ok(Html(format!(
    "<DOCTYPE html>
    <html>
      <head>
        <title>Axel static server</title>
      </head>
      <body>
        <h1>Welcome to the Axel static server!</h1>
        <p>This server provides initial configuration and static assets for the game.</p>
        <p>Available endpoints: <code>/versions/{{version}}</code></p>
        <hr />
        <i>{name}/{version}</i>
      </body>
    </html>",
  )))
}

async fn get_public_key() -> axum::response::Result<impl IntoResponse, AppError> {
  Ok(include_str!("../pubkey.pem"))
}
