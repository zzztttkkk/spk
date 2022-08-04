use std::io::Write;
use std::net::TcpStream;

pub struct Conn {
	stream: TcpStream,
}

impl Conn {
	pub fn new(stream: TcpStream) -> Self {
		return Self { stream };
	}

	pub fn handle(&mut self) {
		println!("{:?} {}", std::thread::current().id(), self.stream.peer_addr().unwrap());
		let mut sref = &self.stream;
		sref.write(b"HTTP/1.0 200 OK\r\nContent-Length: 12\r\n\r\nHello World!").err();
		sref.flush().err();
	}
}