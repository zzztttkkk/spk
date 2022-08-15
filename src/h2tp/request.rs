use std::fmt;
use bytes::BytesMut;
use crate::h2tp::{headers, types};
use crate::h2tp::headers::Headers;
use crate::h2tp::message::{Message, ParseError};
use crate::h2tp::url::Url;

pub struct Request {
	msg: Message,
}

pub struct Builder<'req> {
	req: &'req mut Request,
}

impl<'req> Builder<'req> {
	fn new(v: &'req mut Request) -> Self {
		return Self {
			req: v,
		};
	}

	pub fn method(&mut self, method: &str) -> &mut Self {
		self.req.msg.startline.0 = method.to_string();
		return self;
	}

	pub fn rawpath(&mut self, path: &str) -> &mut Self {
		self.req.msg.startline.1 = path.to_string();
		return self;
	}

	pub fn url(&mut self, url: &Url) -> &mut Self {
		let pathref = &mut self.req.msg.startline.1;
		pathref.clear();
		url.to(pathref).unwrap();
		return self;
	}

	pub fn headers(&mut self) -> headers::Builder {
		return self.req.msg.headers_builder();
	}
}


impl fmt::Debug for Request {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Request <{} {} {} @ {:p}>",
			   self.method(), self.path(), self.version(),
			   self,
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

	pub async fn from<R: types::AsyncReader>(&mut self, stream: &mut R) -> Option<ParseError> {
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

	pub fn builder(&mut self) -> Builder {
		return Builder::new(self);
	}
}