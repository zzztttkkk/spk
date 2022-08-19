use crate::h2tp::utils::multi_map::MultiMap;
use std::fmt;
use std::fmt::Formatter;

macro_rules! pub_str_const {
	($name:ident, $val:expr) => {
		pub const $name: &str = $val;
	};
}

pub mod hns {
	pub_str_const!(CONTENT_LENGTH, "content-length");
	pub_str_const!(CONTENT_TYPE, "content-type");
	pub_str_const!(TRANSFER_ENCODING, "transfer-encoding");
	pub_str_const!(EXPIRES, "expires");
	pub_str_const!(CONNECTION, "connection");
	pub_str_const!(KEEP_ALIVE, "keep-alive");
	pub_str_const!(LAST_MODIFIED, "last-modified");
	pub_str_const!(IF_MODIFIED_SINCE, "if-modified-since");
	pub_str_const!(IF_UNMODIFIED_SINCE, "if-unmodified-since");
	pub_str_const!(E_TAG, "e-tag");
	pub_str_const!(IF_MATCH, "if-match");
	pub_str_const!(IF_NONE_MATCH, "if-none-match");
	pub_str_const!(ACCEPT_ENCODING, "accept-encoding");
	pub_str_const!(COOKIE, "cookie");
	pub_str_const!(SET_COOKIE, "set-cookie");
	pub_str_const!(CONTENT_DISPOSITION, "content-disposition");
	pub_str_const!(CONTENT_ENCODING, "content-encoding");
	pub_str_const!(VIA, "via");
	pub_str_const!(LOCATION, "location");
	pub_str_const!(HOST, "host");
	pub_str_const!(USER_AGENT, "user-agent");
	pub_str_const!(ALLOW, "allow");
	pub_str_const!(SERVER, "server");
	pub_str_const!(ACCEPT_RANGE, "accept-range");
	pub_str_const!(RANGE, "range");
	pub_str_const!(IF_RANGE, "if-range");
	pub_str_const!(CONTENT_RANGE, "content-range");
	pub_str_const!(SEC_WEBSOCKET_KEY, "sec-websocket-key");
	pub_str_const!(SEC_WEBSOCKET_ACCEPT, "sec-websocket-accept");
	pub_str_const!(SEC_WEBSOCKET_VERSION, "sec-websocket-version");
	pub_str_const!(SEC_WEBSOCKET_PROTOCOL, "sec-websocket-protocol");
	pub_str_const!(SEC_WEBSOCKET_EXTENSIONS, "sec-websocket-extensions");
	pub_str_const!(DATE, "date");
	pub_str_const!(RETRY_AFTER, "retry-after");
	pub_str_const!(UPGRADE, "upgrade");
}

pub mod mime {
	pub_str_const!(STREAM, "application/octet-stream");

	pub_str_const!(TEXT, "text/plain");
	pub_str_const!(CSS, "text/css");
	pub_str_const!(HTML, "text/html");
	pub_str_const!(JAVASCRIPT, "text/javascript");

	pub_str_const!(GIF, "image/gif");
	pub_str_const!(JPEG, "image/jpeg");
	pub_str_const!(PNG, "image/png");
	pub_str_const!(SVG, "image/svg+xml");
	pub_str_const!(WEBP, "image/webp");

	pub_str_const!(JSON, "application/json");
	pub_str_const!(WWW_FORM_URLENCODED, "application/x-www-form-urlencoded");
	pub_str_const!(MULTIPART_FORM, "multipart/form-data");

	pub_str_const!(WOFF, "font/woff");
}

pub struct Headers {
	m: MultiMap,
}

pub struct Builder<'h> {
	headers: &'h mut Headers,
}

impl<'h> Builder<'h> {
	pub fn append(&mut self, k: &str, v: &str) -> &mut Self {
		self.headers.m.append(k, v);
		return self;
	}

	pub fn reset(&mut self, k: &str, v: &str) -> &mut Self {
		self.headers.m.reset(k, v);
		return self;
	}

	pub fn clear(&mut self) -> &mut Self {
		self.headers.m.clear();
		return self;
	}

	pub fn content_length(&mut self, size: usize) -> &mut Self {
		return self.reset(hns::CONTENT_LENGTH, size.to_string().as_str());
	}

	pub fn content_type(&mut self, ct: &str) -> &mut Self {
		return self.reset(hns::CONTENT_TYPE, ct);
	}

	pub fn transfer_encoding(&mut self, encoding: &str) -> &mut Self {
		return self.append(hns::TRANSFER_ENCODING, encoding);
	}
}

macro_rules! getone {
	($name:ident -> $key:expr) => {
		pub fn $name(&self) -> Option<&String> {
			return self.m.getone($key);
		}
	};
}

impl Headers {
	pub fn new() -> Self {
		return Self { m: MultiMap::new() };
	}

	pub fn builder(&mut self) -> Builder {
		return Builder { headers: self };
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
					Err(_) => None,
				};
			}
			None => None,
		}
	}

	getone!(content_type -> hns::CONTENT_TYPE);

	getone!(transfer_encoding -> hns::TRANSFER_ENCODING);

	pub fn is_chunked(&self) -> bool {
		return match self.transfer_encoding() {
			Some(v) => v.contains("chunked"),
			None => false,
		};
	}
}

impl fmt::Debug for Headers {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "Headers(")?;
		self.m.each(|k, v, is_last| {
			if is_last {
				_ = write!(f, " {} = {}", k, v);
			} else {
				_ = write!(f, " {} = {};", k, v);
			}
		});
		write!(f, ")")
	}
}
