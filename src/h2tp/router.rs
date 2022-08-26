use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::h2tp::{Request, Response};
use crate::h2tp::handler::{Handler, FuncHandler, HandlerFuture};
use std::rc::Rc;

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
	fn before_middleware(&self) -> &Vec<Box<dyn Middleware>>;
	fn after_middleware(&self) -> &Vec<Box<dyn Middleware>>;

	/// apply a path pattern
	fn register(
		&mut self,
		pattern: &str,
		handler: Box<dyn Handler>,
	) -> Result<(), RouterRegisterResult>;

	fn find<'a>(&self, req: &'a Request) -> Option<&dyn Handler>;
}


pub struct MiddlewareRunningFuture<'a> {
	vec: &'a Vec<Box<dyn Middleware>>,
	req: &'a mut Request,
	resp: &'a mut Response,
	idx: usize,
	future: Option<MiddlewareFuture<'a>>,
}

impl<'a> MiddlewareRunningFuture<'a> {
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

impl<'a> Future for MiddlewareRunningFuture<'a> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		loop {
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
									if self.idx >= self.vec.len() {
										return Poll::Ready(());
									}
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

enum RouterHandleRunningStatus {
	Init,
	Before,
	After,
	Handler,
	Done,
}

pub struct RouterRunningFuture<'a> {
	status: RouterHandleRunningStatus,
	router: &'a dyn Router,
	req: &'a mut Request,
	resp: &'a mut Response,
	handler: Option<&'a dyn Handler>,
	future: Option<HandlerFuture<'a>>,
}

impl<'a> RouterRunningFuture<'a> {
	fn new(router: &'a dyn Router, req: &'a mut Request, resp: &'a mut Response) -> Self {
		return Self {
			router,
			req,
			resp,
			status: RouterHandleRunningStatus::Init,
			handler: None,
			future: None,
		};
	}
}

impl<'a> Future for RouterRunningFuture<'a> {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		use RouterHandleRunningStatus as Status;

		loop {
			match self.status {
				Status::Init => {
					let mut future = MiddlewareRunningFuture::new(
						self.router.before_middleware(),
						self.req,
						self.resp,
					);
					match (Pin::new(&mut future)).poll(cx) {
						Poll::Ready(_) => {
							self.status = Status::Before;
						}
						Poll::Pending => {
							return Poll::Pending;
						}
					}
				}
				Status::Before => {
					let handler = self.router.find(self.req);
					match handler {
						Some(handler) => {
							self.handler = Some(handler);
							let mut future = MiddlewareRunningFuture::new(
								self.router.after_middleware(),
								self.req,
								self.resp,
							);
							match (Pin::new(&mut future)).poll(cx) {
								Poll::Ready(_) => {
									self.status = Status::After;
								}
								Poll::Pending => {
									return Poll::Pending;
								}
							}
						}
						None => {
							let mut handler: Option<Rc<dyn Handler>> = None;
							NOT_FOUND_HANDLER.with(|h| {
								handler = Some(h.clone());
							});
							self.future = Some(handler.as_ref().unwrap().handle(self.req, self.resp));
							self.status = Status::Handler;
						}
					}
				}
				Status::After => {
					self.future = Some(self.handler.unwrap().handle(self.req, self.resp));
					self.status = Status::Handler;
				}
				Status::Handler => {
					let future = self.future.as_mut().unwrap();
					match Pin::new(future).poll(cx) {
						Poll::Pending => {
							return Poll::Pending;
						}
						Poll::Ready(_) => {
							self.status = Status::Done;
						}
					}
				}
				Status::Done => {
					return Poll::Ready(());
				}
			}
		}
	}
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

impl<T> Handler for T
	where
		T: Router,
{
	fn handle<'a>(&self, req: &'a mut Request, resp: &'a mut Response) -> HandlerFuture<'a> {
		return Box::pin(
			RouterRunningFuture::new(
				self,
				req,
				resp,
			)
		);
	}
}