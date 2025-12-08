use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tracing::error;

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

pub trait IntoCallResponse {
  fn into_call_response(self: Box<Self>) -> CallResponse<dyn CallCustom>;
}

pub struct Unsigned<T: IntoCallResponse>(pub T);

impl<T: IntoCallResponse> IntoHandlerResponse for Unsigned<T> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    let response = Box::new(self.0).into_call_response();
    HandlerResponse::unsigned(response)
  }
}

// impl IntoHandlerResponse for Unsigned<CallResponse<dyn CallCustom>> {
//   fn into_handler_response(self: Box<Self>) -> HandlerResponse {
//     HandlerResponse::unsigned(self.0)
//   }
// }

pub struct Signed<T: IntoCallResponse>(pub T, pub Arc<Session>);

impl<T: IntoCallResponse> IntoHandlerResponse for Signed<T> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    let response = Box::new(self.0).into_call_response();
    HandlerResponse::signed(response, self.1.clone())
  }
}

impl<T: IntoHandlerResponse + 'static> IntoHandlerResponse for anyhow::Result<T> {
  fn into_handler_response(self: Box<Self>) -> HandlerResponse {
    match *self {
      Ok(val) => Box::new(val).into_handler_response(),
      Err(error) => {
        error!("handler error: {:?}", error);
        todo!()
      }
    }
  }
}

impl<T: CallCustom + 'static> IntoCallResponse for CallResponse<T> {
  fn into_call_response(self: Box<Self>) -> CallResponse<dyn CallCustom> {
    CallResponse {
      status: self.status,
      time: self.time,
      remote: self.remote,
      notifications: self.notifications,
      custom: self.custom as Box<dyn CallCustom>,
    }
  }
}

impl IntoCallResponse for CallResponse<dyn CallCustom> {
  fn into_call_response(self: Box<Self>) -> CallResponse<dyn CallCustom> {
    *self
  }
}

impl<T: CallCustom + 'static> IntoCallResponse for T {
  fn into_call_response(self: Box<Self>) -> CallResponse<dyn CallCustom> {
    CallResponse::new_success(self)
  }
}
