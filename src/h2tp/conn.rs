use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool};
use tokio::io::{AsyncWriteExt};
use tokio::sync::{RwLock};
use crate::h2tp::cfg::ATOMIC_ORDERING;
use crate::h2tp::handler::Handler;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use crate::h2tp::types::{AsyncReader, AsyncWriter};

pub struct Conn<R: AsyncReader, W: AsyncWriter> {
	addr: SocketAddr,
	r: R,
	w: W,
	server_is_closing: Arc<AtomicBool>,
}

impl<R: AsyncReader, W: AsyncWriter> Conn<R, W> {
	pub fn new(addr: SocketAddr, r: R, w: W, server_is_closing: Arc<AtomicBool>) -> Self {
		return Self { addr, r, w, server_is_closing };
	}

	pub async fn as_server(&mut self, handler: Arc<Box<dyn Handler + Send + Sync>>) {
		let req = Arc::new(RwLock::new(Request::new()));
		let resp = Arc::new(RwLock::new(Response::new()));

		loop {
			{
				let mut g = req.write().await;
				let req = &mut (*g);
				match req.from(&mut self.r).await {
					Some(_) => {
						break;
					}
					None => {}
				}
			}

			(handler.handle(req.clone(), resp.clone()).await).err();

			if self.server_is_closing.load(ATOMIC_ORDERING) {
				return;
			}

			self.w.write(b"HTTP/1.0 200 OK\r\nContent-Length: 11\r\n\r\nHello World").await.err();

			{
				let mut g = req.write().await;
				let req = &mut (*g);
				req.clear();
			}
		}
	}
}