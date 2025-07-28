use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::{MatchedPath, Path, Request};
use axum::http::response::Builder;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{middleware, Json, Router, ServiceExt};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tower::{Layer, Service};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span, warn};
use url::Url;

use crate::client_ip::{add_client_ip, ClientIp};
use crate::normalize_path::normalize_path;
use crate::AppError;

const UPSTREAM_URL: &str = "https://smb.assasans.dev/konofd/";

pub async fn start() -> io::Result<()> {
  let app = Router::new()
    .route("/", get(get_root_friendly))
    .route("/versions/{version}", get(get_version))
    .route("/webview/EN/data/json/news.json", get(get_news))
    .nest_service(
      "/bundles",
      ServeDir::new("static/bundles").fallback(ServeRemoteResource::new(
        UPSTREAM_URL.parse::<Url>().unwrap().join("bundles/").unwrap(),
      )),
    )
    .nest_service(
      "/banners",
      ServeDir::new("static/banners").fallback(ServeRemoteResource::new(
        UPSTREAM_URL.parse::<Url>().unwrap().join("banners/").unwrap(),
      )),
    )
    .nest_service(
      "/webview",
      ServeDir::new("static/webview").fallback(ServeRemoteResource::new(
        UPSTREAM_URL.parse::<Url>().unwrap().join("webview/").unwrap(),
      )),
    )
    .layer(
      TraceLayer::new_for_http()
        // Create our own span for the request and include the matched path. The matched
        // path is useful for figuring out which handler the request was routed to.
        .make_span_with(|request: &Request| {
          let method = request.method();
          let uri = request.uri();

          // axum automatically adds this extension.
          let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());
          let client_ip = request
            .extensions()
            .get::<ClientIp>()
            .map(|client_ip| client_ip.0)
            .unwrap();

          info_span!("request", %client_ip, %method, %uri, matched_path)
        })
        // By default, `TraceLayer` will log 5xx responses but we're doing our specific
        // logging of errors so disable that
        .on_failure(()),
    )
    .layer(middleware::from_fn(add_client_ip));
  let middleware = tower::util::MapRequestLayer::new(normalize_path);
  let app = middleware.layer(app);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:2021").await.unwrap();
  info!("static server started at {:?}", listener.local_addr().unwrap());
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
  let news = vec![NewsItem {
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
  }];

  Ok(Json(news))
}

async fn get_version(Path(version): Path<String>) -> axum::response::Result<impl IntoResponse, AppError> {
  info!("get version info: {}", version);

  Ok(Json(GetVersion {
    app_version: "4.11.6".to_string(),
    asset_version: "2025012110".to_string(),
    api_url: "axel.assasans.dev/api/".to_string(),
    asset_url: "axel.assasans.dev/static/bundles/4.11.6/".to_string(),
    webview_url: "axel.assasans.dev/static/webview/".to_string(),
    banner_url: "axel.assasans.dev/static/banners/".to_string(),
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
        <p>Upstream URL: <a href=\"{UPSTREAM_URL}\">{UPSTREAM_URL}</a></p>
        <hr />
        <i>{name}/{version}</i>
      </body>
    </html>",
  )))
}
