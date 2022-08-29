use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;

pub type HandlerFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Handler: Send + Sync {
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a>;
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
