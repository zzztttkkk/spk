use std::io::Read;
use crate::h2tp::message::Message;
use crate::h2tp::types::AsyncReader;

pub enum RespBody {
	File(String),
	AsyncStream(Box<dyn AsyncReader>),
	Stream(Box<dyn Read>),
}

pub struct Response {
	msg: Message,
}

impl Response {
	pub fn new() -> Self {
		return Response {
			msg: Message::new(),
		};
	}
}