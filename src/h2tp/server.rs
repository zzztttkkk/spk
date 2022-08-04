use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use crate::h2tp::conn::Conn;

pub struct Server {
	threads: u8,
	listener: Option<TcpListener>,
	senders: Vec<Sender<TcpStream>>,
}

impl Server {
	pub fn new(threads: u8) -> Self {
		return Self { listener: None, senders: vec![], threads };
	}

	pub fn listen<Addr: std::net::ToSocketAddrs>(&mut self, addr: Addr) {
		self.listener = Some(TcpListener::bind(addr).unwrap());

		for _ in 0..self.threads {
			let (tx, rx) = mpsc::channel();
			self.senders.push(tx);

			std::thread::spawn(move || {
				for s in rx.iter() {
					let mut conn = Conn::new(s);
					conn.handle();
				}
			});
		}

		let mut idx = 0;
		for result in self.listener.as_mut().unwrap().incoming() {
			match result {
				Ok(stream) => {
					idx = idx % self.threads;
					let tx: &Sender<TcpStream> = &(self.senders[(idx as usize)]);
					tx.send(stream).err();
					idx += 1;
				}
				Err(_) => {}
			}
		}
	}
}
