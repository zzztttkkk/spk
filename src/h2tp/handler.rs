use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::future::Future;
use std::pin::Pin;
use async_trait::async_trait;

pub type HandlerFuture<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;

#[async_trait]
pub trait Handler: Send + Sync {
	async fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> ();
}

type FuncType = for<'a, 'c> fn(req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> HandlerFuture<'a>;

pub struct FuncHandler {
	f: FuncType,
}

impl FuncHandler {
	#[inline]
	pub fn new(f: FuncType) -> Self {
		return Self { f };
	}
}

#[async_trait]
impl Handler for FuncHandler {
	#[inline]
	async fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) -> () {
		(self.f)(req, resp).await
	}
}
