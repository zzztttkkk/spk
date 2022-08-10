use std::fmt;
use bytes::BytesMut;
use tokio::net::tcp::ReadHalf;
use tokio::net::TcpStream;
use crate::h2tp::headers::Headers;
use crate::h2tp::message::{Message, ParseError};

pub struct Request {
	msg: Message,
}

pub struct RequestBuild<'req> {
	req: &'req mut Request,
}

impl<'req> RequestBuild<'req> {
	fn new(v: &'req mut Request) -> Self {
		return Self {
			req: v,
		};
	}

	fn method(&mut self, method: &str) -> &mut Self {
		self.req.msg.startline.0 = method.to_string();
		return self;
	}

	fn path(&mut self, path: &str) -> &mut Self {
		return self;
	}
}


impl fmt::Debug for Request {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Request <{} {} {} @ {:#x}>",
			   self.method(), self.path(), self.version(),
			   (self as *const Request as u64),
		)
	}
}

impl Request {
	pub fn new() -> Self {
		return Self {
			msg: Message::new(),
		};
	}

	pub fn clear(&mut self) {
		self.msg.clear();
	}

	pub async fn from<'sl>(&mut self, stream: &mut ReadHalf<'sl>) -> Option<ParseError> {
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

	pub fn builder(&mut self) -> RequestBuild {
		return RequestBuild::new(self);
	}
}