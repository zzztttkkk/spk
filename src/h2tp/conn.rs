use crate::h2tp::cfg::ATOMIC_ORDERING;
use crate::h2tp::handler::Handler;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use crate::h2tp::types::{AsyncReader, AsyncWriter};
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use super::ctx::Context;

pub struct Conn<R: AsyncReader, W: AsyncWriter> {
	addr: SocketAddr,
	r: R,
	w: W,
	server_is_closing: Arc<AtomicBool>,
}

impl<R: AsyncReader, W: AsyncWriter> Conn<R, W> {
	pub fn new(addr: SocketAddr, r: R, w: W, server_is_closing: Arc<AtomicBool>) -> Self {
		return Self {
			addr,
			r,
			w,
			server_is_closing,
		};
	}

	pub async fn as_server(&mut self, handler: Arc<Mutex<Box<dyn Handler + Send + Sync>>>) {
		let mut req = Request::new();
		let mut resp = Response::new();
		let mut _ctx = Context::new(self);

		loop {
			match req.from(&mut self.r).await {
				Some(e) => {
					if !e.is_empty() && !e.is_eof() {
						println!("{e:?}");
					}
					break;
				}
				None => {}
			}

			let mut guard = handler.lock().await;
			let result = (*guard).handle(&mut req, &mut resp).await;
			match result {
				Ok(_) => {}
				Err(e) => {
					println!("{:?}", e);
				}
			}

			if self.server_is_closing.load(ATOMIC_ORDERING) {
				return;
			}

			self.w
				.write(b"HTTP/1.0 200 OK\r\nContent-Length: 11\r\n\r\nHello World")
				.await
				.err();

			req.clear();
			resp.clear();
		}
	}
}
