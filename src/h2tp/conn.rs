use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct Conn {
	stream: TcpStream,
}

impl Conn {
	pub fn new(stream: TcpStream) -> Self {
		return Self { stream };
	}

	pub async fn handle(&mut self) {
		println!("{}", self.stream.peer_addr().unwrap());
		self.stream.write(b"HTTP/1.0 200 OK\r\nContent-Length: 11\r\n\r\nHello World").await.err();
	}
}