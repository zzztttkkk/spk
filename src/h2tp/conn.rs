use tokio::net::TcpStream;

use crate::h2tp::cfg::ATOMIC_ORDERING;
use crate::h2tp::handler::Handler;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::types::{AsyncReader, AsyncWriter, CliTlsStream, ServTlsStream};

pub struct Conn {
	addr: SocketAddr,
	server_is_closing: Arc<AtomicBool>,
	stream: Option<TcpStream>,
	servtlsstream: Option<ServTlsStream>,
	clitlsstream: Option<CliTlsStream>,
}

impl Conn {
	pub fn newservtls(
		addr: SocketAddr,
		stream: ServTlsStream,
		server_is_closing: Arc<AtomicBool>,
	) -> Self {
		return Self {
			addr,
			server_is_closing,
			stream: None,
			servtlsstream: Some(stream),
			clitlsstream: None,
		};
	}

	pub fn newclitls(
		addr: SocketAddr,
		stream: CliTlsStream,
		server_is_closing: Arc<AtomicBool>,
	) -> Self {
		return Self {
			addr,
			server_is_closing,
			stream: None,
			servtlsstream: None,
			clitlsstream: Some(stream),
		};
	}

	pub fn new(addr: SocketAddr, stream: TcpStream, server_is_closing: Arc<AtomicBool>) -> Self {
		return Self {
			addr,
			server_is_closing,
			stream: Some(stream),
			servtlsstream: None,
			clitlsstream: None,
		};
	}

	fn rwpair<'a>(&'a mut self) -> (Box<dyn AsyncReader + 'a>, Box<dyn AsyncWriter + 'a>) {
		let r: Box<dyn AsyncReader>;
		let w: Box<dyn AsyncWriter>;

		match self.stream.as_mut() {
			Some(sref) => {
				let (rp, wp) = sref.split();
				r = Box::new(rp);
				w = Box::new(wp);
			}
			None => match self.servtlsstream.as_mut() {
				Some(sref) => {
					let (rp, wp) = tokio::io::split(sref);
					r = Box::new(rp);
					w = Box::new(wp);
				}
				None => match self.clitlsstream.as_mut() {
					Some(sref) => {
						let (rp, wp) = tokio::io::split(sref);
						r = Box::new(rp);
						w = Box::new(wp);
					}
					None => {
						panic!();
					}
				},
			},
		}
		return (r, w);
	}

	// https://github.com/rustls/rustls/issues/288
	// https://github.com/tokio-rs/tokio/issues/1108
	pub async fn as_server(&mut self, handler: Arc<dyn Handler + Send + Sync>) {
		let mut req = Request::new();
		req.msg.remote = Some(self.addr);

		let mut resp = Response::new();
		let cc = self.server_is_closing.clone();

		let (mut r, mut w) = self.rwpair();
		let r = r.as_mut();
		let w = w.as_mut();

		loop {
			match req.from(r).await {
				Some(e) => {
					if !e.is_empty() && !e.is_eof() {
						println!("{e:?}");
					}
					break;
				}
				None => {}
			}

			let result = handler.handle(&mut req, &mut resp).await;
			match result {
				Ok(_) => {}
				Err(e) => {
					println!("{:?}", e);
				}
			}

			if cc.load(ATOMIC_ORDERING) {
				return;
			}

			resp.msg.to(w).await;

			req.clear();
			resp.clear();
		}
	}
}
