use std::time::Duration;
use async_trait::async_trait;

use crate::h2tp::{Request, Response};
use crate::h2tp::handler::{Handler};

pub enum MiddlewareControl {
	Continue,
	Break,
	Return,
}

/// `Middleware` will return by the `Router`, then be called one by one in `Router.handle`.
#[async_trait]
pub trait Middleware: Send + Sync {
	/// `handle` take `req` and `resp`, return a `MiddlewareControl`.
	/// - if return `::Continue`, the loop of middleware group call will continue;
	/// - if return `::Break`, the loop of middleware group call will break;
	/// - if return `::Return`, the loop of middleware group call will break and
	/// the `Router.handle` will return, and the `handler` which returned by `Router.find` will not execute;
	async fn handle<'a, 'c>(&self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> MiddlewareControl;
}

pub enum RouterFindError {
	NotFound,
	MethodNotAllow(String),
	RedirectTo(String),
	RetryAfter(Duration),
	Undefined,
}

#[async_trait]
pub trait Router: Handler {
	/// `middleware` return two groups of middleware, the first group to execute before `find` and
	/// the second group will execute after a successful `find` call.
	fn middleware<'a, 'c>(&self, req: &'a Request<'c>) -> (&Vec<Box<dyn Middleware>>, &Vec<Box<dyn Middleware>>);
	/// `find` return a `& dyn Handler` or `RouterFindError`
	fn find<'a, 'c>(&self, req: &'a Request<'c>) -> Result<&dyn Handler, RouterFindError>;
	/// `onerror` handle the error that returned by `find`
	async fn onerror<'a, 'c>(&self, err: RouterFindError, req: &'a mut Request<'c>, resp: &'a mut Response<'c>);

	async fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) {
		let (before, after) = self.middleware(req);

		macro_rules! run_middleware {
		    ($name:ident) => {
				for middleware in $name {
					match middleware.handle(req, resp).await {
						MiddlewareControl::Continue => {
							continue;
						}
						MiddlewareControl::Break => {
							break;
						}
						MiddlewareControl::Return => {
							return;
						}
					}
				}
			};
		}

		run_middleware!(before);

		match self.find(req) {
			Err(e) => {
				self.onerror(e, req, resp).await;
			}
			Ok(handler) => {
				run_middleware!(after);
				handler.handle(req, resp).await;
			}
		}
	}
}

// macro_rules! impl_handler_for_router {
//     ($name:ident) => {
// 		#[async_trait]
// 		impl Handler for $name {
// 			#[inline]
// 			async fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> () {
// 				Router::handle(self, req, resp).await
// 			}
// 		}
// 	};
// }
