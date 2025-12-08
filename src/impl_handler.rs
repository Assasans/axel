use std::future::Future;
use std::marker::PhantomData;

use tracing::error;

use crate::extractor::FromContext;
use crate::handler::{BoxFuture, Handler, HandlerContext, IntoHandlerResponse};

pub struct HandlerFn<F, Args> {
  f: F,
  _marker: PhantomData<fn(Args) -> ()>,
}

pub trait IntoHandler<Args>: Clone + Send + Sync + 'static {
  fn into_handler(self) -> Box<dyn Handler>;
}

macro_rules! impl_handler {
  ($($ty:ident),*) => {
    #[allow(non_snake_case, unused_variables)]
    impl<F, R, Fut, $($ty,)*> IntoHandler<($($ty,)*)> for F
    where
      F: Fn($($ty,)*) -> Fut + Clone + Send + Sync + 'static,
      R: IntoHandlerResponse + 'static,
      Fut: Future<Output = R> + Send + 'static,
      $($ty: FromContext + Send + 'static,)*
    {
      fn into_handler(self) -> Box<dyn Handler> {
        Box::new(HandlerFn {
          f: {
            let f = self;
            move |$($ty,)*| {
              let fut = f($($ty,)*);
              async move {
                let res = fut.await;
                Box::new(res) as Box<dyn IntoHandlerResponse>
              }
            }
          },
          _marker: PhantomData::<fn(($($ty,)*)) -> ()>,
        })
      }
    }

    #[allow(non_snake_case, unused_variables)]
    impl<F, Fut, $($ty,)*> Handler for HandlerFn<F, ($($ty,)*)>
    where
      F: Fn($($ty,)*) -> Fut + Clone + Send + Sync + 'static,
      Fut: Future<Output = Box<dyn IntoHandlerResponse>> + Send + 'static,
      $($ty: FromContext + Send + 'static,)*
    {
      #[allow(unused_mut)]
      fn call(&self, mut ctx: HandlerContext) -> BoxFuture<Box<dyn IntoHandlerResponse>> {
        $(
          let $ty = match $ty::from_context(&mut ctx) {
            Ok(val) => val,
            // return Box::pin(async move { Err(e) })
            Err(error) => {
              error!("failed to extract parameter: {:?}", error);
              todo!()
            },
          };
        )*

        let fut = (self.f)($($ty,)*);
        Box::pin(fut)
      }
    }
  };
}

impl_handler!();
impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
