use core::fmt;
use std::fmt::Formatter;

pub struct Url<'a> {
	scheme: &'a str,
	username: &'a str,
	password: &'a str,
	host: &'a str,
	port: &'a str,
	path: &'a str,
	rawquery: &'a str,
	fragment: &'a str,
}

pub struct ParseErr {
	msg: &'static str,
}

impl ParseErr {
	fn new(m: &'static str) -> Self {
		return Self {
			msg: m,
		};
	}
}

impl fmt::Debug for ParseErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "UrlParseError: {}", self.msg)
	}
}

impl<'a> fmt::Debug for Url<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f,
			   "Url(\n\tScheme: {}\n\tUsername: {}\n\tPassword: {}\n\tHost: {}\n\tPort: {}\n\tPath: {}\n\tRawQuery: {}\n\tFragment: {}\n)",
			   self.scheme,
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

impl<'a> Url<'a> {
	pub fn parse(v: &'a str) -> Result<Self, ParseErr> {
		let mut obj = Self {
			scheme: "",
			username: "",
			password: "",
			host: "",
			port: "",
			path: "",
			rawquery: "",
			fragment: "",
		};

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
}

#[cfg(test)]
mod tests {
	use crate::h2tp::url::Url;

	#[test]
	fn test_parse() {
		println!("{:?}", Url::parse("http://:4555@a.com:567/ddd?e=45fff#err"));
		println!("{:?}", Url::parse("er@:45/ddd?rtt"));
	}
}