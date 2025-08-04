use crate::client_ip::ClientIp;
use axum::extract::{MatchedPath, Request};
use tower_http::trace::{
  DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest, DefaultOnResponse, HttpMakeClassifier, TraceLayer,
};
use tracing::{info_span, Span};

/// Creates a span for the request and includes the matched path.
///
/// The matched path is useful for figuring out which handler the request was routed to.
pub fn log_requests() -> TraceLayer<
  HttpMakeClassifier,
  impl Fn(&Request) -> Span + Clone,
  DefaultOnRequest,
  DefaultOnResponse,
  DefaultOnBodyChunk,
  DefaultOnEos,
  (),
> {
  TraceLayer::new_for_http()
    .make_span_with(|request: &Request| {
      let method = request.method();
      let uri = request.uri();
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
    // By default, `TraceLayer` will log 5xx responses, but we're doing our specific
    // logging inside [AppError::into_response] so disable that
    .on_failure(())
}
