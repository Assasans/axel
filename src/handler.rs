use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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

pub struct Unsigned(pub CallResponse<dyn CallCustom>);

impl IntoHandlerResponse for Unsigned {
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
