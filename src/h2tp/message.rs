use std::fmt;
use std::fmt::{Formatter, write};
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::h2tp::cfg::MESSAGE_BUFFER_SIZE;
use crate::h2tp::utils::multi_map::MultiMap;

struct Message {
	startline: (String, String, String),
	headers: Option<MultiMap>,
	body: Option<BytesMut>,
	buf: Option<BytesMut>,
	bufremains: usize,
}

#[derive(PartialEq)]
enum ParseStatus {
	Empty,
	Startline1,
	Startline2,
	Startline3,
	HeadersOK,
	Done,
}

pub struct ParseError {
	ioe: Option<std::io::Error>,
	ue: Option<&'static str>,
}

impl ParseError {
	pub fn ioe(v: std::io::Error) -> Self {
		return Self {
			ioe: Some(v),
			ue: None,
		};
	}

	pub fn ue(v: &'static str) -> Self {
		return Self {
			ioe: None,
			ue: Some(v),
		};
	}
}

impl fmt::Debug for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self.ioe.as_ref() {
			Some(ioe) => {
				write!(f, "{}", ioe)
			}
			None => {
				return match self.ue.as_ref() {
					Some(pe) => {
						write!(f, "{}", pe)
					}
					None => {
						write!(f, "Empty ParseError")
					}
				};
			}
		}
	}
}

const BAD_REQUEST: &'static str = "bad request";

impl Message {
	fn new() -> Self {
		return Self {
			startline: (String::new(), String::new(), String::new()),
			headers: None,
			body: None,
			buf: None,
			bufremains: 0,
		};
	}

	fn clear(&mut self) {
		self.startline.0.clear();
		self.startline.1.clear();
		self.startline.2.clear();
		match self.headers.as_mut() {
			Some(href) => {
				href.clear();
			}
			None => {}
		}
		self.bufremains = 0;
	}

	async fn from(&mut self, stream: &mut TcpStream) -> Option<ParseError> {
		self.clear();

		if self.buf.is_none() {
			self.buf = Some(BytesMut::with_capacity(MESSAGE_BUFFER_SIZE));
		}

		let bufref = self.buf.as_mut().unwrap().as_mut();
		let mut status: ParseStatus = ParseStatus::Empty;
		let mut skip_newline = false;
		let mut hkey = String::new();
		let mut hval = String::new();
		let mut hkvsep = false;

		loop {
			let bufslice: &[u8];
			if self.bufremains > 0 {
				bufslice = &bufref[0..self.bufremains];
			} else {
				let size = stream.read(bufref).await;
				match size {
					Ok(size) => {
						self.bufremains = size;
						bufslice = bufref;
					}
					Err(e) => {
						return Some(ParseError::ioe(e));
					}
				}
			}

			for c in bufslice {
				self.bufremains -= 1;
				let c = *c;

				if skip_newline {
					if c != b'\n' {
						return Some(ParseError::ue(BAD_REQUEST));
					}

					skip_newline = false;
					continue;
				}

				match status {
					ParseStatus::Empty => {
						if c == b' ' {
							status = ParseStatus::Startline1;
						} else {
							self.startline.0.push(c as char);
						}
					}
					ParseStatus::Startline1 => {
						if c == b' ' {
							status = ParseStatus::Startline2;
						} else {
							self.startline.1.push(c as char);
						}
					}
					ParseStatus::Startline2 => {
						if c == b'\r' {
							status = ParseStatus::Startline3;
							skip_newline = true;
						} else {
							self.startline.2.push(c as char);
						}
					}
					ParseStatus::Startline3 => {
						if c == b'\r' {
							skip_newline = true;
							if hkvsep {
								if self.headers.is_none() {
									self.headers = Some(MultiMap::new());
								}
								let headersref = self.headers.as_mut().unwrap();
								headersref.append(
									&hkey.trim().to_ascii_lowercase(),
									&hval.trim(),
								);
								hkey.clear();
								hval.clear();
							} else {
								if !hkey.is_empty() || !hval.is_empty() {
									return Some(ParseError::ue(BAD_REQUEST));
								}
								status = ParseStatus::HeadersOK;
								break;
							}
						} else {
							if hkvsep {
								hval.push(c as char);
								hkvsep = false;
							} else {
								if c == b':' {
									hkvsep = true;
								} else {
									hkey.push(c as char);
								}
							}
						}
					}
					_ => {
						unreachable!("http message parse");
					}
				}
			}

			if status == ParseStatus::HeadersOK {
				break;
			}
		}

		return None;
	}
}


pub struct Request {
	msg: Message,
}

impl Request {
	pub fn new() -> Self {
		return Self {
			msg: Message::new(),
		};
	}


	pub async fn from(&mut self, stream: &mut TcpStream) -> Option<ParseError> {
		return self.msg.from(stream);
	}
}