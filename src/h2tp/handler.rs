use std::future::Future;
use std::pin::Pin;
use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;

type BoxedFuture = Pin<Box<dyn Future<Output=Result<Response, Error>> + Send>>;
type FuncType = fn(req: Request) -> BoxedFuture;

pub trait Handler {
	fn handle(&self, req: Request) -> BoxedFuture;
}

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
	fn handle(&self, req: Request) -> BoxedFuture {
		(self.f)(req)
	}
}

