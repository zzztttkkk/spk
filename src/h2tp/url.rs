use core::fmt;
use std::fmt::{Display, Formatter};
use crate::h2tp::utils::multi_map::MultiMap;

pub struct Builder<'a> {
	writable: &'a mut Setter,
}

macro_rules! simple_setter {
    ($field:ident, $idx:expr) => {
		pub fn $field(&mut self, v: &str) -> &mut Self {
			self.writable.parts[$idx] = v.to_string();
			return self;
		}
	};
}

impl<'a> Builder<'a> {
	fn new(w: &'a mut Setter) -> Self {
		return Self {
			writable: w,
		};
	}

	simple_setter!(scheme, 0);
}

struct Setter {
	parts: [String; 8],
	query: Option<MultiMap>,
}

impl Setter {
	fn new() -> Self {
		return Self {
			parts: Default::default(),
			query: None,
		};
	}
}

pub struct Url<'a> {
	scheme: &'a str,
	username: &'a str,
	password: &'a str,
	host: &'a str,
	port: &'a str,
	path: &'a str,
	rawquery: &'a str,
	fragment: &'a str,

	setter: Option<Setter>,
}

pub struct ParseErr {
	msg: &'static str,
}

impl ParseErr {
	fn new(m: &'static str) -> Self { Self { msg: m } }
}

impl fmt::Debug for ParseErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "UrlParseError: {}", self.msg) }
}

impl Display for ParseErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "UrlParseError: {}", self.msg) }
}

impl std::error::Error for ParseErr {}

impl<'a> fmt::Debug for Url<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f,
			   "Url(\n\tScheme: {}\n\tUsername: {}\n\tPassword: {}\n\tHost: {}\n\tPort: {}\n\tPath: {}\n\tRawQuery: {}\n\tFragment: {}\n)",
			   self.scheme(),
			   self.username,
			   self.password,
			   self.host,
			   self.port,
			   self.path,
			   self.rawquery,
			   self.fragment,
		)
	}
}

const PATH_MISSING: &'static str = "Path Missing";

macro_rules! simple_getter {
    ($field:ident, $idx:expr) => {
		pub fn $field(&self) -> &str {
			match self.writable.as_ref() {
				Some(wref) => {
					&(wref.parts[$idx])
				}
				None => {
					self.$field
				}
			}
		}
	};
}

impl<'a> Url<'a> {
	pub fn new() -> Self {
		return Self {
			scheme: "",
			username: "",
			password: "",
			host: "",
			port: "",
			path: "",
			rawquery: "",
			fragment: "",
			setter: None,
		};
	}

	pub fn parse(v: &'a str) -> Result<Self, ParseErr> {
		let mut obj = Self::new();
		match obj.from(v) {
			Some(e) => {
				Err(e)
			}
			None => {
				Ok(obj)
			}
		}
	}

	pub fn from(&mut self, v: &'a str) -> Option<ParseErr> {
		let mut v = &v[0..];
		match v.find("://") {
			Some(idx) => {
				self.scheme = &v[..idx];
				v = &v[idx + 3..];
			}
			None => {}
		}

		match v.find(b'@' as char) {
			Some(idx) => {
				let userinfo = &v[..idx];
				v = &v[idx + 1..];
				match userinfo.find(b':' as char) {
					Some(idx) => {
						self.username = &userinfo[..idx];
						self.password = &userinfo[idx + 1..];
					}
					None => {
						self.username = userinfo;
					}
				}
			}
			None => {}
		}

		let mut hosted = false;
		match v.find(b':' as char) {
			Some(idx) => {
				self.host = &v[..idx];
				v = &v[idx + 1..];
				hosted = true;
			}
			None => {}
		}

		match v.find(b'/' as char) {
			Some(idx) => {
				if !hosted {
					self.host = &v[..idx];
				} else {
					self.port = &v[..idx];
				}
				v = &v[idx..];
			}
			None => {
				return Some(ParseErr::new(PATH_MISSING));
			}
		}

		match v.find(b'?' as char) {
			Some(idx) => {
				self.path = &v[..idx];
				v = &v[idx + 1..];

				match v.find(b'#' as char) {
					Some(idx) => {
						self.rawquery = &v[..idx];
						self.fragment = &v[idx + 1..];
					}
					None => {
						self.rawquery = v;
					}
				}
			}
			None => {
				match v.find(b'#' as char) {
					Some(idx) => {
						self.path = &v[..idx];
						self.fragment = &v[idx + 1..];
					}
					None => {
						self.path = v;
					}
				}
			}
		}
		return None;
	}

	pub fn builder(&mut self) -> Builder {
		if self.setter.is_none() {
			self.setter = Some(Setter::new());
		}
		return Builder::new(self.setter.as_mut().unwrap());
	}

	simple_getter!(scheme, 0);
}

#[cfg(test)]
mod tests {
	use crate::h2tp::url::{Url, Builder};

	#[test]
	fn test_parse() {
		println!("{:?}", Url::parse("https://:4555@a.com:567/ddd?e=45fff#err"));
		println!("{:?}", Url::parse("er@:45"));

		let mut url = Url::new();
		url.builder().scheme("XXX");
		println!("V: {}", url.scheme());
	}
}