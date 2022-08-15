use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;

type BoxedFuture = Pin<Box<dyn Future<Output=Result<(), Error>> + Send>>;
type Req = Arc<RefCell<Request>>;
type Resp = Arc<RefCell<Response>>;
type FuncType = fn(req: Req, resp: Resp) -> BoxedFuture;

pub trait Handler {
	fn handle(&self, req: Req, resp: Resp) -> BoxedFuture;
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
	fn handle(&self, req: Req, resp: Resp) -> BoxedFuture {
		(self.f)(req, resp)
	}
}

