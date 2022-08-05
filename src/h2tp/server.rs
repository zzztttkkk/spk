use tokio::net::{TcpListener};
use crate::h2tp::conn::Conn;

pub struct Server {
	listener: Option<TcpListener>,
}

impl Server {
	pub fn new() -> Self {
		return Self { listener: None };
	}

	pub async fn listen<Addr: tokio::net::ToSocketAddrs>(&mut self, addr: Addr) {
		self.listener = Some(TcpListener::bind(addr).await.unwrap());

		let lref = self.listener.as_ref().unwrap();
		loop {
			let (stream, _) = lref.accept().await.unwrap();
			tokio::spawn(async move {
				let mut conn = Conn::new(stream);
				conn.handle().await;
			});
		}
	}
}
