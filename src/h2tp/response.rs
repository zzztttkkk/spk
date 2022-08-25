use bytes::BytesMut;

use crate::h2tp::message::Message;
use crate::h2tp::types::AsyncReader;
use std::fmt::Write;
use std::io::Read;

pub enum RespBody {
	File(String),
	AsyncStream(Box<dyn AsyncReader + Send>),
	SyncStream(Box<dyn Read + Send>),
}

pub struct Response {
	pub(crate) msg: Message,
	pub(crate) body: Option<RespBody>,
}

impl Response {
	pub fn new() -> Self {
		return Response {
			msg: Message::new(),
			body: None,
		};
	}

	pub fn clear(&mut self) {
		self.msg.clear();
		self.body = None;
	}

	pub fn write(&mut self, val: &str) {
		if self.msg.body.is_none() {
			self.msg.body = Some(BytesMut::new());
		}

		let bodyref = self.msg.body.as_mut().unwrap();
		let _ = bodyref.write_str(val);
	}
}
