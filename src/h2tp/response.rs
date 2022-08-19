use bytes::{BufMut, BytesMut};

use crate::h2tp::message::Message;
use crate::h2tp::types::AsyncReader;
use std::io::Read;

use super::headers;
use super::status_code::StatusCode;

pub enum RespBody {
	File(String),
	AsyncStream(Box<dyn AsyncReader + Send>),
	SyncStream(Box<dyn Read + Send>),
}

pub struct Response {
	msg: Message,
	body: Option<RespBody>,
}

pub struct Builder<'a> {
	resp: &'a mut Response,
}

impl<'a> Builder<'a> {
	#[inline]
	pub fn statuscode(&mut self, code: StatusCode) -> &mut Self {
		let num = code as u32;
		self.resp.msg.startline.0 = num.to_string();
		self.resp.msg.startline.1.clear();
		self.resp.msg.startline.1.push_str(code.msg());
		return self;
	}

	#[inline]
	pub fn headers(&mut self) -> headers::Builder {
		return self.resp.msg.headers_builder();
	}
}

impl<'a> std::io::Write for Builder<'a> {
	#[inline]
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		if self.resp.msg.body.is_none() {
			self.resp.msg.body = Some(BytesMut::with_capacity(4096));
		}
		let bufref = self.resp.msg.body.as_mut().unwrap();
		bufref.put_slice(buf);
		return Ok(buf.len());
	}

	#[inline]
	fn flush(&mut self) -> std::io::Result<()> {
		return Ok(());
	}
}

impl Response {
	pub fn new() -> Self {
		return Response {
			msg: Message::new(),
			body: None,
		};
	}

	pub fn builder(&mut self) -> Builder {
		return Builder { resp: self };
	}

	pub fn clear(&mut self) {
		self.msg.clear();
		self.body = None;
	}
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use super::Response;

	#[test]
	fn test_builder_statuscode() {
		let mut resp = Response::new();
		let mut builder = resp.builder();
		builder.statuscode(crate::h2tp::status_code::StatusCode::Accepted);
		let mut headers = builder.headers();
		headers.content_length(12);

		_ = builder.write(b"----");

		println!("");
	}
}
