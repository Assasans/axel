use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::api::ApiRequest;
use crate::handler::HandlerContext;
use crate::params_deserializer::HashMapDeserializer;
use crate::user::session::Session;
use crate::AppState;

pub trait FromContext: Sized + Send {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self>;
}

impl FromContext for ApiRequest {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self> {
    Ok(ctx.request.take().expect("ApiRequest already taken"))
  }
}

impl FromContext for Arc<AppState> {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self> {
    Ok(Arc::clone(&ctx.state))
  }
}

impl<T: FromContext> FromContext for Option<T> {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self> {
    Ok(T::from_context(ctx).ok())
  }
}

impl FromContext for Arc<Session> {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self> {
    ctx
      .session
      .clone()
      .ok_or_else(|| anyhow::anyhow!("No session available"))
  }
}

/// Extracts parameters from request body. Inner type T must implement DeserializeOwned.
pub struct Params<T>(pub T);

impl<T: DeserializeOwned + Send> FromContext for Params<T> {
  fn from_context(ctx: &mut HandlerContext) -> anyhow::Result<Self> {
    let request = ctx
      .request
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("ApiRequest already taken"))?;
    let deserializer = HashMapDeserializer::new(&request.body);
    let params = T::deserialize(deserializer)?;
    Ok(Params(params))
  }
}
