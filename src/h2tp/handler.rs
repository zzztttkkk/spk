use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub type HandlerFuture<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;

pub trait Handler {
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a>;
}

pub enum Control {
	Continue,
	Break,
}

pub type MiddlewareFuture<'a> = Pin<Box<dyn Future<Output=Control> + Send + 'a>>;

pub trait Middleware {
	fn name(&self) -> str;
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> MiddlewareFuture<'a>;
}

pub enum RouterRegisterResult {
	BadPattern(String),
	PatternConflict(String),
}


pub trait Router {
	fn before_middleware(&self) -> Option<&Vec<Box<dyn Middleware>>>;
	fn after_middleware(&self) -> Option<&Vec<Box<dyn Middleware>>>;

	/// apply a path pattern
	fn register(
		&mut self,
		pattern: &str,
		handler: Box<dyn Handler>,
	) -> Result<(), RouterRegisterResult>;

	fn find<'a>(&self, req: &'a Request) -> Option<&dyn Handler>;
}

thread_local! {
	static NOT_FOUND_HANDLER: Rc<dyn Handler> = Rc::new(
		FuncHandler::new(
			|_, response: &mut Response| {
				Box::pin(async move {
					response.msg.startline.1 = "404".to_string();
					response.msg.startline.2 = "Not Found".to_string();
				})
			}
		)
	);
}

pub struct MiddlewareRunning<'a> {
	vec: &'a Vec<Box<dyn Middleware>>,
	req: &'a mut Request,
	resp: &'a mut Response,
	idx: usize,
	future: Option<MiddlewareFuture<'a>>,
}

impl<'a> MiddlewareRunning<'a> {
	pub fn new(vec: &'a Vec<Box<dyn Middleware>>, req: &'a mut Request, resp: &'a mut Response) -> Self {
		return Self {
			vec,
			resp,
			req,
			idx: 0,
			future: None,
		};
	}
}

impl<'a> Future for MiddlewareRunning<'a> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		loop {
			if self.idx >= self.vec.len() {
				return Poll::Ready(());
			}

			match self.future.as_mut() {
				Some(f) => {
					return match Pin::new(f).poll(cx) {
						Poll::Pending => {
							Poll::Pending
						}
						Poll::Ready(out) => {
							match out {
								Control::Continue => {
									self.idx += 1;
									Poll::Pending
								}
								Control::Break => {
									Poll::Ready(())
								}
							}
						}
					};
				}
				None => {
					let current = self.vec[self.idx].as_ref();
					self.future = Some(current.handle(self.req, self.resp));
				}
			}
		}
	}
}


impl<T> Handler for T
	where
		T: Router,
{
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a> {
		match self.find(req) {
			Some(handler) => handler.handle(req, resp),
			None => {
				let mut handler: Option<Rc<dyn Handler>> = None;
				NOT_FOUND_HANDLER.with(|h| {
					handler = Some(h.clone());
				});
				handler.as_ref().unwrap().handle(req, resp)
			}
		}
	}
}

type FuncType = for<'a> fn(req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a>;

pub struct FuncHandler {
	f: FuncType,
}

impl FuncHandler {
	#[inline]
	pub fn new(f: FuncType) -> Self {
		return Self { f };
	}
}

impl Handler for FuncHandler {
	#[inline]
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a> {
		(self.f)(req, resp)
	}
}
