use core::fmt;
use std::fmt::Formatter;
use crate::h2tp::status_code::StatusCode;

pub struct Error {
	code: StatusCode,
	msgref: &'static str,
	msg: Option<String>,
}

impl Error {
	pub fn new(code: StatusCode, msg: &str) -> Self {
		return Self {
			code,
			msg: Some(msg.to_string()),
			msgref: "",
		};
	}

	pub fn newstatic(code: StatusCode, msg: &'static str) -> Self {
		return Self {
			code,
			msgref: msg,
			msg: None,
		};
	}

	pub fn statuscode(&self) -> StatusCode { self.code }

	pub fn msg(&self) -> &str {
		return match self.msg.as_ref() {
			Some(mref) => {
				mref
			}
			None => {
				self.msgref
			}
		};
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "HttpError({:?}, {})", self.code, self.msg())
	}
}