use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::h2tp::{Request, Response};
use crate::h2tp::handler::{Handler, HandlerFuture};

pub enum MiddlewareControl {
	Continue,
	Break,
}

pub type MiddlewareFuture<'a> = Pin<Box<dyn Future<Output=MiddlewareControl> + Send + 'a>>;

pub trait Middleware: Send + Sync {
	fn handle<'a, 'c>(&self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> MiddlewareFuture;
}

struct MiddlewareExecutor<'a, 'c, 'r: 'a> {
	req: &'a mut Request<'c>,
	resp: &'a mut Response<'c>,
	vec: &'r Vec<Box<dyn Middleware>>,
	idx: usize,
	future: Option<MiddlewareFuture<'a>>,
}

impl<'a, 'c, 'r: 'a> Future for MiddlewareExecutor<'a, 'c, 'r> {
	type Output = MiddlewareControl;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		loop {
			return match self.future.as_mut() {
				Some(future) => {
					match Pin::new(future).poll(cx) {
						Poll::Ready(ctrl) => {
							self.idx += 1;
							match ctrl {
								MiddlewareControl::Continue => {
									if self.idx >= self.vec.len() {
										return Poll::Ready(MiddlewareControl::Continue);
									}
									self.future = None;
									Poll::Pending
								}
								MiddlewareControl::Break => {
									Poll::Ready(MiddlewareControl::Break)
								}
							}
						}
						Poll::Pending => {
							Poll::Pending
						}
					}
				}
				None => {
					let middleware = &self.vec[self.idx];
					self.future = Some(middleware.handle(self.req, self.resp));
					Poll::Pending
				}
			};
		}
	}
}


pub trait Router: Send + Sync {
	fn middleware(&self) -> (&Vec<Box<dyn Middleware>>, &Vec<Box<dyn Middleware>>);
	fn find<'a, 'c>(&self, req: &'a Request<'c>) -> &dyn Handler;
}

enum RouterExecutorStatus {
	Init,
	Before,
	Find,
	After,
	Done,
}

struct RouterExecutor<'a, 'c, 'r> {
	req: &'a mut Request<'c>,
	resp: &'a mut Response<'c>,
	router: &'r dyn Router,

	status: RouterExecutorStatus,
	middleware_idx: i32,
}

impl<'a, 'c, 'r> RouterExecutor<'a, 'c, 'r> {
	fn new(req: &'a mut Request<'c>, resp: &'a mut Response<'c>, router: &'r dyn Router) -> Self {
		return Self {
			req,
			resp,
			router,

			status: RouterExecutorStatus::Init,
			middleware_idx: 0,
		};
	}
}

impl<'a, 'c, 'r> Future for RouterExecutor<'a, 'c, 'r> {
	type Output = ();

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		use RouterExecutorStatus::*;

		loop {
			match self.status {
				_ => {}
			}
		}
	}
}

impl<T> Handler for T where T: Router {
	fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> HandlerFuture<'a> {
		Box::pin(RouterExecutor::new(req, resp, self))
	}
}
