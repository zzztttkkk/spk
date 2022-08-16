use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use tokio::sync::RwLock;
use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;

type BoxedFuture = Pin<Box<dyn Future<Output=Result<(), Error>> + Send>>;
type FuncType<'a> = fn(req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture;

pub trait Handler<'a> {
	fn handle(&self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture;
}

pub struct FuncHandler<'a> {
	f: FuncType<'a>,
}

impl<'a> FuncHandler<'a> {
	#[inline]
	pub fn new(f: FuncType) -> Self {
		return Self { f };
	}
}

impl<'a> Handler<'a> for FuncHandler<'a> {
	#[inline]
	fn handle(&self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture {
		(self.f)(req, resp)
	}
}

