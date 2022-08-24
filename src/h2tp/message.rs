use crate::h2tp::cfg::MESSAGE_BUFFER_SIZE;
use crate::h2tp::headers::Headers;
use crate::h2tp::{headers, types};
use bytes::BytesMut;
use std::fmt;
use std::fmt::Formatter;
use std::io::ErrorKind;
use tokio::io::AsyncReadExt;

use super::conn::Conn;
use super::types::{AsyncReader, AsyncWriter};

pub struct Message<'c, R: AsyncReader, W: AsyncWriter> {
	pub(crate) startline: (String, String, String),
	pub(crate) headers: Option<Headers>,
	pub(crate) body: Option<BytesMut>,
	buf: Option<BytesMut>,
	bufsize: usize,
	bufremains: usize,

	conn: Option<&'c Conn<R, W>>,
}

#[derive(PartialEq)]
enum ParseStatus {
	Empty,
	Startline1,
	Startline2,
	Startline3,
	HeadersOK,
}

pub struct ParseError {
	ioe: Option<std::io::Error>,
	ue: Option<&'static str>,
}

impl ParseError {
	fn ioe(v: std::io::Error) -> Self {
		return Self {
			ioe: Some(v),
			ue: None,
		};
	}

	fn ue(v: &'static str) -> Self {
		return Self {
			ioe: None,
			ue: Some(v),
		};
	}

	fn empty() -> Self {
		return Self {
			ioe: None,
			ue: None,
		};
	}

	pub fn is_empty(&self) -> bool {
		return self.ioe.is_none() && self.ue.is_none();
	}

	pub fn is_eof(&self) -> bool {
		return match self.ioe.as_ref() {
			Some(v) => v.kind() == ErrorKind::UnexpectedEof,
			None => false,
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

const BAD_REQUEST: &str = "bad request";

impl<'c, R, W> Message<'c, R, W>
where
	R: AsyncReader,
	W: AsyncWriter,
{
	pub(crate) fn new() -> Self {
		return Self {
			startline: (String::new(), String::new(), String::new()),
			headers: None,
			body: None,
			buf: None,
			bufsize: 0,
			bufremains: 0,
			conn: None,
		};
	}

	pub(crate) fn setconn(&mut self, c: &'c Conn<R, W>) {
		self.conn = Some(c);
	}

	pub(crate) fn clear(&mut self) {
		self.startline.0.clear();
		self.startline.1.clear();
		self.startline.2.clear();
		match self.headers.as_mut() {
			Some(href) => {
				href.builder().clear();
			}
			None => {}
		}
		self.bufremains = 0;
		self.bufsize = 0;
		match self.body.as_mut() {
			Some(bodyref) => {
				bodyref.clear();
			}
			None => {}
		}
	}

	pub(crate) async fn read(&mut self, stream: &mut R) -> Option<ParseError> {
		if self.bufremains > 0 {
			return None;
		}

		match stream.read(self.buf.as_mut().unwrap().as_mut()).await {
			Ok(size) => {
				if size == 0 {
					return Some(ParseError::empty());
				}
				self.bufremains = size;
				self.bufsize = size;
			}
			Err(e) => {
				return Some(ParseError::ioe(e));
			}
		}
		return None;
	}

	pub(crate) async fn read_sized_body(
		&mut self,
		stream: &mut R,
		cl: usize,
	) -> Option<ParseError> {
		if cl == 0 {
			return None;
		}

		let mut remain = cl;
		loop {
			match self.read(stream).await {
				Some(e) => {
					return Some(e);
				}
				None => {}
			}

			let bodyref = self.body.as_mut().unwrap();
			let bufref = self.buf.as_mut().unwrap().as_mut();
			let bytesslice: &[u8] = &bufref[self.bufsize - self.bufremains..self.bufsize];

			if remain >= bytesslice.len() {
				self.bufremains = 0;
				remain -= bytesslice.len();
				bodyref.extend_from_slice(bytesslice);
			} else {
				self.bufremains -= remain;
				remain = 0;
				bodyref.extend_from_slice(&bytesslice[0..remain]);
			}

			if remain == 0 {
				break;
			}
		}
		return None;
	}

	pub(crate) async fn read_byte(&mut self, stream: &mut R) -> Result<u8, ParseError> {
		let bufref = self.buf.as_mut().unwrap().as_mut();

		let c: u8;
		if self.bufremains > 0 {
			self.bufremains -= 1;
			c = bufref[self.bufsize - self.bufremains];
		} else {
			match stream.read_u8().await {
				Ok(v) => {
					c = v;
				}
				Err(e) => {
					return Err(ParseError::ioe(e));
				}
			}
		}
		return Ok(c);
	}

	pub(crate) async fn read_chunked_body(&mut self, stream: &mut R) -> Option<ParseError> {
		let mut current_chunk_size: Option<usize> = None;
		let mut numbuf = String::new();
		let mut skip_newline = false;

		loop {
			match current_chunk_size {
				None => {
					match self.read(stream).await {
						Some(e) => {
							return Some(e);
						}
						None => {}
					}

					let bufref = self.buf.as_mut().unwrap().as_mut();
					let bytesslice: &[u8] = &bufref[self.bufsize - self.bufremains..self.bufsize];
					for c in bytesslice {
						self.bufremains -= 1;
						let c = *c;

						if skip_newline {
							if c != b'\n' {
								return Some(ParseError::ue(BAD_REQUEST));
							}
							skip_newline = false;
							break;
						}

						if c == b'\r' {
							match numbuf.parse::<i32>() {
								Ok(v) => {
									if v < 0 {
										return Some(ParseError::ue(BAD_REQUEST));
									}
									current_chunk_size = Some(v as usize);
								}
								Err(_) => {
									return Some(ParseError::ue(BAD_REQUEST));
								}
							}
							continue;
						}
						numbuf.push(c as char);
					}
				}
				Some(remain) => match self.read_sized_body(stream, remain).await {
					Some(e) => {
						return Some(e);
					}
					None => {
						match self.read_byte(stream).await {
							Ok(c) => {
								if c != b'\r' {
									return Some(ParseError::ue(BAD_REQUEST));
								}
							}
							Err(e) => {
								return Some(e);
							}
						}
						match self.read_byte(stream).await {
							Ok(c) => {
								if c != b'\n' {
									return Some(ParseError::ue(BAD_REQUEST));
								}
							}
							Err(e) => {
								return Some(e);
							}
						}
						if remain == 0 {
							break;
						}
					}
				},
			}
		}
		return None;
	}

	pub(crate) async fn read_body(&mut self, stream: &mut R) -> Option<ParseError> {
		let mut cl: Option<usize> = None;
		match &self.headers {
			Some(href) => {
				cl = href.content_length();
			}
			None => {}
		}

		match cl {
			Some(cl) => {
				if self.body.is_none() {
					let buf = BytesMut::with_capacity(cl);
					self.body = Some(buf);
				}
				match self.read_sized_body(stream, cl).await {
					Some(e) => {
						return Some(e);
					}
					None => {}
				}
			}
			None => {
				let mut is_chunked = false;
				match &self.headers {
					Some(href) => {
						is_chunked = href.is_chunked();
					}
					None => {}
				}
				if is_chunked {
					if self.body.is_none() {
						let buf = BytesMut::with_capacity(4096);
						self.body = Some(buf);
					}
					match self.read_chunked_body(stream).await {
						Some(e) => {
							return Some(e);
						}
						None => {}
					}
				}
			}
		}
		return None;
	}

	pub(crate) async fn from(&mut self, stream: &mut R) -> Option<ParseError> {
		if self.buf.is_none() {
			let mut buf = BytesMut::with_capacity(MESSAGE_BUFFER_SIZE);
			unsafe {
				buf.set_len(MESSAGE_BUFFER_SIZE);
			}
			self.buf = Some(buf);
		}
		let mut status: ParseStatus = ParseStatus::Empty;
		let mut skip_newline = false;
		let mut hkey = String::new();
		let mut hval = String::new();
		let mut hkvsep = false;

		loop {
			match self.read(stream).await {
				Some(e) => {
					return Some(e);
				}
				None => {}
			}

			let bufref = self.buf.as_mut().unwrap().as_mut();
			let bytesslice: &[u8] = &bufref[self.bufsize - self.bufremains..self.bufsize];

			for c in bytesslice {
				self.bufremains -= 1;
				let c = *c;

				if skip_newline {
					if c != b'\n' {
						return Some(ParseError::ue(BAD_REQUEST));
					}

					skip_newline = false;
					if status == ParseStatus::HeadersOK {
						break;
					}
					continue;
				}

				match status {
					ParseStatus::Empty => {
						if c == b' ' {
							status = ParseStatus::Startline1;
						} else {
							if c > 127 {
								return Some(ParseError::ue(BAD_REQUEST));
							}

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
									self.headers = Some(Headers::new());
								}
								let headersref = self.headers.as_mut().unwrap();
								headersref
									.builder()
									.append(&hkey.trim().to_ascii_lowercase(), &hval.trim());
								hkey.clear();
								hval.clear();
								hkvsep = false;
							} else {
								if !hkey.is_empty() || !hval.is_empty() {
									return Some(ParseError::ue(BAD_REQUEST));
								}
								status = ParseStatus::HeadersOK;
								continue;
							}
						} else if hkvsep {
							hval.push(c as char);
						} else if c == b':' {
							hkvsep = true;
						} else {
							hkey.push(c as char);
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
		return self.read_body(stream).await;
	}

	pub(crate) fn headers_builder(&mut self) -> headers::Builder {
		if self.headers.is_none() {
			self.headers = Some(Headers::new());
		}
		return self.headers.as_mut().unwrap().builder();
	}
}
