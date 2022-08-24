use crate::h2tp::message::Message;
use crate::h2tp::types::AsyncReader;
use std::io::Read;

pub enum RespBody {
	File(String),
	AsyncStream(Box<dyn AsyncReader + Send>),
	SyncStream(Box<dyn Read + Send>),
}

pub struct Response {
	msg: Message,
	body: Option<RespBody>,
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
}
