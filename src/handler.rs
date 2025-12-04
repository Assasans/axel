use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tracing::warn;

use crate::api::ApiRequest;
use crate::call::{CallCustom, CallResponse};
use crate::user::session::Session;
use crate::AppState;

pub struct HandlerContext {
  pub request: Option<ApiRequest>,
  pub state: Arc<AppState>,
  pub session: Option<Arc<Session>>,
}

pub struct HandlerResponse {
  pub response: CallResponse<dyn CallCustom>,
  /// Which session to use for signing the response, if any.
  pub signing_session: Option<Arc<Session>>,
}

impl HandlerResponse {
  pub fn unsigned(response: CallResponse<dyn CallCustom>) -> HandlerResponse {
    HandlerResponse {
      response,
      signing_session: None,
    }
  }

  pub fn signed(response: CallResponse<dyn CallCustom>, signing_session: Arc<Session>) -> HandlerResponse {
    HandlerResponse {
      response,
      signing_session: Some(signing_session),
    }
  }
}

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait Handler: Send + Sync {
  fn call(&self, ctx: HandlerContext) -> BoxFuture<Box<dyn IntoHandlerResponse>>;
}

pub trait IntoHandlerResponse {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse;
}

pub struct Unsigned<T: CallCustom + ?Sized>(pub CallResponse<T>);

impl<T: CallCustom + 'static> IntoHandlerResponse for Unsigned<T> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    // Convert CallResponse<T> into CallResponse<dyn CallCustom>
    let response = CallResponse {
      status: self.0.status,
      time: self.0.time,
      remote: self.0.remote,
      notifications: self.0.notifications,
      custom: self.0.custom as Box<dyn CallCustom>,
    };
    HandlerResponse::unsigned(response)
  }
}

impl IntoHandlerResponse for Unsigned<dyn CallCustom> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    HandlerResponse::unsigned(self.0)
  }
}

pub struct Signed(pub CallResponse<dyn CallCustom>, pub Arc<Session>);

impl IntoHandlerResponse for Signed {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    HandlerResponse::signed(self.0, self.1)
  }
}

impl<T: IntoHandlerResponse + 'static> IntoHandlerResponse for anyhow::Result<T> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    match *self {
      Ok(val) => Box::new(val).into_handler_response(),
      Err(e) => todo!(),
    }
  }
}
