use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;

type BoxedFuture<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;

pub trait Handler {
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a>;
}

pub enum RouterRegisterResult {
	BadPattern(String),
	PatternConflict(String),
}

pub trait Router {
	fn register(&mut self, pattern: &str, handler: Box<dyn Handler>) -> Result<(), RouterRegisterResult>;
	fn find<'a>(&self, req: &'a Request) -> Option<&dyn Handler>;
}

impl<T> Handler for T where T: Router {
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a> {
		match self.find(req) {
			Some(handler) => {
				handler.handle(req, resp)
			}
			None => {
				let handler = FuncHandler::new(|_, response: &mut Response| {
					Box::pin(async move {
						response.msg.startline.1 = "404".to_string();
						response.msg.startline.2 = "Not Found".to_string();
					})
				});
				handler.handle(req, resp)
			}
		}
	}
}

type FuncType = for<'a> fn(req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a>;

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
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a> {
		(self.f)(req, resp)
	}
}
