use std::sync::Arc;
use std::sync::atomic::{AtomicBool};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use crate::h2tp::ordering::ATOMIC_ORDERING;

pub struct Conn {
	stream: TcpStream,
	server_is_closing: Option<Arc<AtomicBool>>,
}

impl Conn {
	pub fn new(stream: TcpStream, server_is_closing: Option<Arc<AtomicBool>>) -> Self {
		return Self { stream, server_is_closing };
	}

	pub async fn handle(&mut self) {
		println!("{}", self.stream.peer_addr().unwrap());

		sleep(Duration::from_secs(5)).await;

		self.stream.write(b"HTTP/1.0 200 OK\r\nContent-Length: 11\r\n\r\nHello World").await.err();

		match &self.server_is_closing {
			Some(closing) => {
				if closing.load(ATOMIC_ORDERING) {
					return;
				}
			}
			None => {}
		}
	}
}