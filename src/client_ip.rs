use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;

use axum::extract::{ConnectInfo, FromRequestParts, OptionalFromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

#[derive(Debug, Clone)]
pub struct ClientIp(pub IpAddr);

impl Deref for ClientIp {
  type Target = IpAddr;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<S> FromRequestParts<S> for ClientIp
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    Ok(parts.extensions.get::<ClientIp>().cloned().unwrap())
  }
}

impl<S> OptionalFromRequestParts<S> for ClientIp
where
  S: Send + Sync,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
    Ok(parts.extensions.get::<ClientIp>().cloned())
  }
}

pub async fn add_client_ip(mut request: Request, next: Next) -> Response {
  let connection_ip = request
    .extensions()
    .get::<ConnectInfo<SocketAddr>>()
    .map(|source| source.ip())
    .expect("ConnectInfo extension not found");
  let cf_connecting_ip = request
    .headers()
    .get("CF-Connecting-IP")
    .and_then(|value| value.to_str().ok())
    .and_then(|ip_str| ip_str.parse::<IpAddr>().ok());
  let client_ip = cf_connecting_ip.unwrap_or(connection_ip);

  request.extensions_mut().insert(ClientIp(client_ip));

  next.run(request).await
}
