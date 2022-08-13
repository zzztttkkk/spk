use std::future::Future;
use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;

type BoxedFuture = Box<dyn Future<Output=Result<(), Error>>>;
type FuncType = fn(Box<Request>, Box<Response>) -> BoxedFuture;

pub trait Handler {
	fn handle(&self, req: Box<Request>, resp: Box<Response>) -> BoxedFuture;
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
	fn handle(&self, req: Box<Request>, resp: Box<Response>) -> BoxedFuture {
		(self.f)(req, resp)
	}
}

