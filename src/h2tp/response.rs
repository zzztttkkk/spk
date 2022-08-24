use crate::h2tp::message::Message;
use crate::h2tp::types::AsyncReader;
use std::io::Read;

pub enum RespBody {
	File(String),
	AsyncStream(Box<dyn AsyncReader + Send>),
	SyncStream(Box<dyn Read + Send>),
}

pub struct Response<'c, R, W> {
	msg: Message<'c, R, W>,
	body: Option<RespBody>,
}

impl<'c, R, W> Response<'c, R, W> {
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
