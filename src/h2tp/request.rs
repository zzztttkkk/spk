use crate::h2tp::headers::Headers;
use crate::h2tp::message::{Message, ParseError};
use bytes::BytesMut;
use std::fmt;

use super::types::{AsyncReader, AsyncWriter};

pub struct Request<'c, R: AsyncReader, W: AsyncWriter> {
	msg: Message<'c, R, W>,
}

impl<'c, R: AsyncReader, W: AsyncWriter> fmt::Debug for Request<'c, R, W> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Request <{} {} {} @ {:p}>",
			self.method(),
			self.path(),
			self.version(),
			self,
		)
	}
}

impl<'c, R, W> Request<'c, R, W>
where
	R: AsyncReader,
	W: AsyncWriter,
{
	pub fn new() -> Self {
		return Self {
			msg: Message::new(),
		};
	}

	pub fn clear(&mut self) {
		self.msg.clear();
	}

	pub async fn from(&mut self, stream: &mut R) -> Option<ParseError> {
		return self.msg.from(stream).await;
	}

	pub fn method(&self) -> &str {
		return self.msg.startline.0.as_str();
	}

	pub fn path(&self) -> &str {
		return self.msg.startline.1.as_str();
	}

	pub fn version(&self) -> &str {
		return self.msg.startline.2.as_str();
	}

	pub fn headers(&self) -> Option<&Headers> {
		return self.msg.headers.as_ref();
	}

	pub fn body(&self) -> Option<&BytesMut> {
		return self.msg.body.as_ref();
	}
}
