use std::io;

use axum::extract::{MatchedPath, Path, Request};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, ServiceExt};
use serde::Serialize;
use tower::Layer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::normalize_path::normalize_path;
use crate::AppError;

pub async fn start() -> io::Result<()> {
  let app = Router::new().route("/versions/:version", get(get_version)).layer(
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

async fn get_version(Path(version): Path<String>) -> axum::response::Result<impl IntoResponse, AppError> {
  info!("get version info: {}", version);

  Ok(Json(GetVersion {
    app_version: "4.11.2".to_string(),
    asset_version: "2024073120".to_string(),
    api_url: "api.konosuba.local/".to_string(),
    asset_url: "static-prd-wonder.sesisoft.com/bundles/4.11.2/".to_string(),
    webview_url: "static-prd-wonder.sesisoft.com/webview/".to_string(),
    banner_url: "static-prd-wonder.sesisoft.com/banners/".to_string(),
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
