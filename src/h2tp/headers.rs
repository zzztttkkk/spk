use std::fmt;
use std::fmt::{Formatter};
use crate::h2tp::utils::multi_map::MultiMap;

pub mod hns {
	macro_rules! make {
		($name:ident, $val:expr) => {
			pub const $name: &str = $val;
		};
	}

	make!(CONTENT_LENGTH, "content-length");
	make!(CONTENT_TYPE, "content-type");
	make!(TRANSFER_ENCODING, "transfer-encoding");
	make!(EXPIRES, "expires");
	make!(CONNECTION, "connection");
	make!(KEEP_ALIVE, "keep-alive");
	make!(LAST_MODIFIED, "last-modified");
	make!(IF_MODIFIED_SINCE, "if-modified-since");
	make!(IF_UNMODIFIED_SINCE, "if-unmodified-since");
	make!(E_TAG, "e-tag");
	make!(IF_MATCH, "if-match");
	make!(IF_NONE_MATCH, "if-none-match");
	make!(ACCEPT_ENCODING, "accept-encoding");
	make!(COOKIE, "cookie");
	make!(SET_COOKIE, "set-cookie");
	make!(CONTENT_DISPOSITION, "content-disposition");
	make!(CONTENT_ENCODING, "content-encoding");
	make!(VIA, "via");
	make!(LOCATION, "location");
	make!(HOST, "host");
	make!(USER_AGENT, "user-agent");
	make!(ALLOW, "allow");
	make!(SERVER, "server");
	make!(ACCEPT_RANGE, "accept-range");
	make!(RANGE, "range");
	make!(IF_RANGE, "if-range");
	make!(CONTENT_RANGE, "content-range");
	make!(SEC_WEBSOCKET_KEY, "sec-websocket-key");
	make!(SEC_WEBSOCKET_ACCEPT, "sec-websocket-accept");
	make!(SEC_WEBSOCKET_VERSION, "sec-websocket-version");
	make!(SEC_WEBSOCKET_PROTOCOL, "sec-websocket-protocol");
	make!(SEC_WEBSOCKET_EXTENSIONS, "sec-websocket-extensions");
	make!(DATE, "date");
	make!(RETRY_AFTER, "retry-after");
	make!(UPGRADE, "upgrade");
}

pub struct Headers {
	m: MultiMap,
}


impl Headers {
	pub fn new() -> Self {
		return Self {
			m: MultiMap::new(),
		};
	}

	pub fn append(&mut self, k: &str, v: &str) {
		self.m.append(k, v);
	}

	pub fn clear(&mut self) {
		self.m.clear();
	}

	pub fn content_length(&self) -> Option<usize> {
		match self.m.getone(hns::CONTENT_LENGTH) {
			Some(v) => {
				return match v.parse::<i32>() {
					Ok(num) => {
						if num < 0 {
							return None;
						}
						return Some(num as usize);
					}
					Err(_) => {
						None
					}
				};
			}
			None => {
				None
			}
		}
	}

	#[allow(dead_code)]
	pub fn content_type(&self) -> Option<&String> {
		return self.m.getone(hns::CONTENT_TYPE);
	}

	pub fn transfer_encoding(&self) -> Option<&String> {
		return self.m.getone(hns::TRANSFER_ENCODING);
	}

	pub fn is_chunked(&self) -> bool {
		return match self.transfer_encoding() {
			Some(v) => {
				v.contains("chunked")
			}
			None => {
				false
			}
		};
	}
}

impl fmt::Debug for Headers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "Headers<\r\n")?;
		self.m.each(|k, v| {
			println!("\t{}: {}", k, v);
		});
		write!(f, ">")
	}
}