use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;

type BoxedFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
type FuncType = for<'a> fn(req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a>;

pub trait Handler {
	fn handle<'a>(&mut self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a>;
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
	fn handle<'a>(&mut self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a> {
		(self.f)(req, resp)
	}
}

pub struct ClosureHandler {
	f: Box<dyn for<'a> FnMut(&'a mut Request, &'a mut Response) -> BoxedFuture<'a>>,
}

impl ClosureHandler {
	pub fn new<T>(f: T) -> Self
	where
		T: for<'a> FnMut(&'a mut Request, &'a mut Response) -> BoxedFuture<'a> + 'static,
	{
		return Self { f: Box::new(f) };
	}
}

impl Handler for ClosureHandler {
	fn handle<'a>(&mut self, req: &'a mut Request, resp: &'a mut Response) -> BoxedFuture<'a> {
		(self.f.as_mut())(req, resp)
	}
}
