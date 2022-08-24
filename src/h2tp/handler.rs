use crate::h2tp::error::Error;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;

type BoxedFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>>;
type FuncType<R, W> =
	for<'a> fn(req: &'a mut Request<R, W>, resp: &'a mut Response<R, W>) -> BoxedFuture<'a>;

pub trait Handler<R, W> {
	fn handle<'a>(
		&self,
		req: &'a mut Request<'a, R, W>,
		resp: &'a mut Response<'a, R, W>,
	) -> BoxedFuture<'a>;
}

pub struct FuncHandler<R, W> {
	f: FuncType<R, W>,
}

impl<R, W> FuncHandler<R, W> {
	#[inline]
	pub fn new(f: FuncType<R, W>) -> Self {
		return Self { f };
	}
}

impl<R, W> Handler<R, W> for FuncHandler<R, W> {
	#[inline]
	fn handle<'a>(
		&self,
		req: &'a mut Request<R, W>,
		resp: &'a mut Response<R, W>,
	) -> BoxedFuture<'a> {
		(self.f)(req, resp)
	}
}
