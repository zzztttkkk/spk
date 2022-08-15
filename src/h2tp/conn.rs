use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool};
use tokio::io::{AsyncWriteExt};
use tokio::sync::Mutex;
use crate::h2tp::cfg::ATOMIC_ORDERING;
use crate::h2tp::handler::Handler;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use crate::h2tp::types::{AsyncReader, AsyncWriter};

pub struct Conn<R: AsyncReader, W: AsyncWriter> {
	addr: SocketAddr,
	r: R,
	w: W,
	server_is_closing: Option<Arc<AtomicBool>>,
}

impl<R: AsyncReader, W: AsyncWriter> Conn<R, W> {
	pub fn new(addr: SocketAddr, r: R, w: W, server_is_closing: Option<Arc<AtomicBool>>) -> Self {
		return Self { addr, r, w, server_is_closing };
	}

	pub async fn as_server(&mut self, handler: Arc<Box<dyn Handler + Send + Sync>>) {
		let req = Arc::new(Mutex::new(Request::new()));
		let resp = Arc::new(Mutex::new(Response::new()));

		loop {
			{
				let mut g = req.lock().await;
				let req = &mut (*g);
				match req.from(&mut self.r).await {
					Some(_) => {
						break;
					}
					None => {}
				}
			}

			(handler.handle(req.clone(), resp.clone()).await).err();

			match &self.server_is_closing {
				Some(closing) => {
					if closing.load(ATOMIC_ORDERING) {
						return;
					}
				}
				None => {}
			}


			self.w.write(b"HTTP/1.0 200 OK\r\nContent-Length: 11\r\n\r\nHello World").await.err();

			{
				let mut g = req.lock().await;
				let req = &mut (*g);
				req.clear();
			}
		}
	}
}