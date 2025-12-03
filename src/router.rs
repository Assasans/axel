use std::collections::HashMap;

use crate::handler::{Handler, HandlerContext, IntoHandlerResponse};
use crate::impl_handler::IntoHandler;

pub struct Router {
  handlers: HashMap<String, Box<dyn Handler>>,
}

impl Router {
  pub fn new() -> Self {
    Self {
      handlers: HashMap::new(),
    }
  }

  pub fn handle<F, Args>(mut self, name: &str, handler: F) -> Self
  where
    F: IntoHandler<Args>,
  {
    self.handlers.insert(name.to_string(), handler.into_handler());
    self
  }

  pub async fn dispatch(&self, name: &str, ctx: HandlerContext) -> Box<dyn IntoHandlerResponse> {
    let handler = self
      .handlers
      .get(name)
      .ok_or_else(|| anyhow::anyhow!("Handler not found: {}", name))
      .expect("Handler not found");

    handler.call(ctx).await
  }
}
