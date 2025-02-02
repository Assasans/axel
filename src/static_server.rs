use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::{MatchedPath, Path, Request};
use axum::handler::HandlerWithoutStateExt;
use axum::http::response::Builder;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, ServiceExt};
use reqwest::Client;
use serde::Serialize;
use tower::{Layer, Service};
use tower_http::services::ServeDir;
use tower_http::set_status::SetStatus;
use tower_http::trace::TraceLayer;
use tracing::info;
use url::Url;

use crate::normalize_path::normalize_path;
use crate::AppError;

pub async fn start() -> io::Result<()> {
  let app = Router::new()
    .route("/versions/:version", get(get_version))
    .nest_service(
      "/bundles",
      ServeDir::new("static/bundles").fallback(ServeRemoteResource::new(
        "https://smb.assasans.dev/konofd/bundles/".parse().unwrap(),
      )),
    )
    .nest_service(
      "/banners",
      ServeDir::new("static/banners").fallback(ServeRemoteResource::new(
        "https://smb.assasans.dev/konofd/banners/".parse().unwrap(),
      )),
    )
    .nest_service(
      "/webview",
      ServeDir::new("static/webview").fallback(ServeRemoteResource::new(
        "https://smb.assasans.dev/konofd/webview/".parse().unwrap(),
      )),
    )
    .layer(
      TraceLayer::new_for_http()
        // Create our own span for the request and include the matched path. The matched
        // path is useful for figuring out which handler the request was routed to.
        .make_span_with(|req: &Request| {
          let method = req.method();
          let uri = req.uri();

          // axum automatically adds this extension.
          let matched_path = req
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str());

          tracing::info_span!("request", %method, %uri, matched_path)
        })
        // By default, `TraceLayer` will log 5xx responses but we're doing our specific
        // logging of errors so disable that
        .on_failure(()),
    );
  let middleware = tower::util::MapRequestLayer::new(normalize_path);
  let app = middleware.layer(app);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:2021").await.unwrap();
  info!("static server started at {:?}", listener.local_addr().unwrap());
  axum::serve(listener, app.into_make_service()).await
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

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, request: Request) -> Self::Future {
    let path = &request.uri().path()[1..];
    let remote_url = self.remote_url.join(path).unwrap();
    info!("get bundle remote: {} -> {}", path, remote_url);

    Box::pin(async {
      let client = Client::new();
      let response = client.get(remote_url).send().await.unwrap();

      let status = StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

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

async fn get_version(Path(version): Path<String>) -> axum::response::Result<impl IntoResponse, AppError> {
  info!("get version info: {}", version);

  Ok(Json(GetVersion {
    app_version: "4.11.6".to_string(),
    asset_version: "2025012110".to_string(),
    api_url: "api.konosuba.local/".to_string(),
    asset_url: "static.konosuba.local/bundles/4.11.6/".to_string(),
    webview_url: "static.konosuba.local/webview/".to_string(),
    banner_url: "static.konosuba.local/banners/".to_string(),
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
