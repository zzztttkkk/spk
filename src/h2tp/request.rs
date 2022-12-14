use crate::h2tp::headers::Headers;
use crate::h2tp::message::{Message, ParseError};
use bytes::BytesMut;
use std::fmt;

use super::types::AsyncReader;

pub struct Request<'c> {
	pub(crate) msg: Message<'c>,
}

impl<'c> fmt::Debug for Request<'c> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Request <")?;

		write!(
			f,
			"{} {} {} @ {:p}",
			self.method(),
			self.path(),
			self.version(),
			self,
		)?;

		match self.msg.conn {
			Some(conn) => {
				write!(f, " <- {}", conn.addr)?;
			}
			None => {}
		}

		write!(f, ">")
	}
}

impl<'c> Request<'c> {
	pub fn new() -> Self {
		return Self {
			msg: Message::new(),
		};
	}

	pub fn clear(&mut self) {
		self.msg.clear();
	}

	pub async fn from(&mut self, stream: &mut dyn AsyncReader) -> Option<ParseError> {
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
