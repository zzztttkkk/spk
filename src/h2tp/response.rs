use bytes::BytesMut;

use crate::h2tp::message::Message;

pub enum RespBody {
	File(String),
}

pub struct Response<'c> {
	pub(crate) msg: Message<'c>,
	pub(crate) body: Option<RespBody>,
}

impl<'c> Response<'c> {
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

	pub fn resetbody(&mut self) {
		self.body = None;
		match self.msg.body.as_mut() {
			Some(body) => {
				body.clear();
			}
			None => {}
		}
	}

	fn ensure_msg_bodybuf(&mut self) -> &mut BytesMut {
		if self.body.is_some() {
			panic!("Response's body is some, call `resetbody` first");
		}
		if self.msg.body.is_none() {
			self.msg.body = Some(BytesMut::new());
		}
		return self.msg.body.as_mut().unwrap();
	}
}

impl<'c> std::io::Write for Response<'c> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let body = self.ensure_msg_bodybuf();
		body.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl<'c> std::fmt::Write for Response<'c> {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		let buf = self.ensure_msg_bodybuf();
		buf.write_str(s)
	}
}
