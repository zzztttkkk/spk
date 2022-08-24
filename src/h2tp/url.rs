use crate::h2tp::utils::multi_map::MultiMap;
use core::fmt;
use std::fmt::{Display, Error, Formatter, Write};

use super::utils::uricoding;

pub struct Builder<'a> {
	setter: &'a mut Setter,
}

macro_rules! simple_setter {
	($field:ident, $idx:expr, $vtype:ty) => {
		pub fn $field(&mut self, v: $vtype) -> &mut Self {
			self.setter.parts[$idx] = v.to_string();
			return self;
		}
	};
}

impl<'a> Builder<'a> {
	fn new(w: &'a mut Setter) -> Self {
		return Self { setter: w };
	}

	simple_setter!(scheme, 0, &str);
	simple_setter!(username, 1, &str);
	simple_setter!(password, 2, &str);
	simple_setter!(host, 3, &str);
	simple_setter!(port, 4, u16);

	/// the `unsafe` means `v` must be escaped, this method
	/// never failed even if the unescaped characters in `v`.
	pub unsafe fn rawquery(&mut self, v: &str) -> &mut Self {
		match self.setter.query.as_mut() {
			Some(qmref) => {
				qmref.clear();
			}
			None => {
				self.setter.query = Some(MultiMap::new());
			}
		}

		let qmref = self.setter.query.as_mut().unwrap();
		let mut rv = v;
		let mut key_buf: Vec<u8> = vec![];
		let mut val_buf: Vec<u8> = vec![];

		let append_ignore_err =
			|cv: &str, idx: usize, kbuf: &mut Vec<u8>, vbuf: &mut Vec<u8>, mref: &mut MultiMap| {
				kbuf.clear();
				if !uricoding::decode_uri(kbuf, &cv[..idx]) {
					return;
				}

				let key;
				match std::str::from_utf8(kbuf) {
					Ok(v) => {
						key = v;
					}
					Err(_) => {
						return;
					}
				}

				vbuf.clear();
				if !uricoding::decode_uri(vbuf, &cv[idx + 1..]) {
					return;
				}

				let val;
				match std::str::from_utf8(vbuf) {
					Ok(v) => {
						val = v;
					}
					Err(_) => {
						return;
					}
				}

				mref.append(key, val);
			};

		loop {
			if rv.is_empty() {
				break;
			}

			let cv: &str;
			match rv.find('&') {
				Some(idx) => {
					cv = &rv[..idx];
					rv = &rv[idx + 1..];
				}
				None => {
					cv = rv;
					rv = "";
				}
			}
			if cv.is_empty() {
				continue;
			}

			match cv.find('=') {
				Some(idx) => {
					append_ignore_err(cv, idx, &mut key_buf, &mut val_buf, qmref);
				}
				None => {
					key_buf.clear();
					if uricoding::decode_uri(&mut key_buf, cv) {
						match std::str::from_utf8(&key_buf) {
							Ok(key) => {
								qmref.append(key, "");
							}
							Err(_) => {}
						}
					}
				}
			}
		}
		return self;
	}

	pub fn path(&mut self, v: &str) -> &mut Self {
		let vref = &mut self.setter.parts[5];
		vref.clear();
		if v.chars().nth(0).unwrap() != (b'/' as char) {
			vref.push(b'/' as char);
		}
		vref.push_str(v);
		return self;
	}

	pub fn query(&mut self) -> &mut MultiMap {
		if self.setter.query.is_none() {
			self.setter.query = Some(MultiMap::new());
		}
		return self.setter.query.as_mut().unwrap();
	}
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

	fn rawquery<'a>(&self, dest: &'a mut Vec<u8>) -> &'a str {
		dest.clear();

		if self.query.is_none() {
			return unsafe { std::str::from_utf8_unchecked(dest.as_slice()) };
		}

		let mut kbuf = vec![];
		let mut vbuf = vec![];

		self.query.as_ref().unwrap().each(|k, v, is_last| {
			kbuf.clear();
			vbuf.clear();
			if k.len() > kbuf.capacity() {
				kbuf.reserve(k.len() - kbuf.capacity());
			}
			if v.len() > vbuf.capacity() {
				vbuf.reserve(v.len() - vbuf.capacity());
			}
			uricoding::encode_uri_component(&mut kbuf, k);
			uricoding::encode_uri_component(&mut vbuf, v);
			dest.append(&mut kbuf);
			dest.push(b'=');
			dest.append(&mut vbuf);
			if !is_last {
				dest.push(b'&');
			}
			return true;
		});
		return unsafe { std::str::from_utf8_unchecked(dest.as_slice()) };
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
	fn new(m: &'static str) -> Self {
		Self { msg: m }
	}
}

impl fmt::Debug for ParseErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "UrlParseError: {}", self.msg)
	}
}

impl Display for ParseErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "UrlParseError: {}", self.msg)
	}
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

const PATH_MISSING: &str = "Path Missing";
const IPV6_ENDING_CHAR_MISSING: &str = "Ipv6 Ending-Char Missing";

macro_rules! getter {
	($field:ident, $idx:expr) => {
		pub fn $field(&self) -> &str {
			match self.setter.as_ref() {
				Some(sref) => {
					let vref = &(sref.parts[$idx]);
					if !vref.is_empty() {
						return vref;
					}
				}
				None => {}
			}
			return self.$field;
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
			Some(e) => Err(e),
			None => Ok(obj),
		}
	}

	/// #### url:
	/// `[scheme://][[username:][password]@][host:port]/[path][?rawquery][#fragment]`
	/// #### host:
	/// - ipv4 or hostname: direct
	/// - ipv6: [2001:db8:1f70::999:de8:7648:6e8]
	pub fn from(&mut self, v: &'a str) -> Option<ParseErr> {
		let mut v = &v[0..];
		match v.find("://") {
			Some(idx) => {
				self.scheme = &v[..idx];
				v = &v[idx + 3..];
			}
			None => {}
		}

		match v.find('@') {
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

		if v.chars().nth(0) == Some('[') {
			match v.find(']') {
				Some(idx) => {
					self.host = &v[..idx + 1];
					v = &v[idx + 1..];
					hosted = true;
				}
				None => {
					return Some(ParseErr::new(IPV6_ENDING_CHAR_MISSING));
				}
			}
		} else {
			match v.rfind(':') {
				Some(idx) => {
					self.host = &v[..idx];
					v = &v[idx + 1..];
					hosted = true;
				}
				None => {}
			}
		}

		match v.find('/') {
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

		match v.find('?') {
			Some(idx) => {
				self.path = &v[..idx];
				v = &v[idx + 1..];

				match v.find('#') {
					Some(idx) => {
						self.rawquery = &v[..idx];
						self.fragment = &v[idx + 1..];
					}
					None => {
						self.rawquery = v;
					}
				}
			}
			None => match v.find('#') {
				Some(idx) => {
					self.path = &v[..idx];
					self.fragment = &v[idx + 1..];
				}
				None => {
					self.path = v;
				}
			},
		}
		return None;
	}

	pub fn builder(&mut self) -> Builder {
		if self.setter.is_none() {
			self.setter = Some(Setter::new());
		}
		return Builder::new(self.setter.as_mut().unwrap());
	}

	pub fn to<W: Write>(&self, dist: &mut W) -> Result<(), Error> {
		dist.write_char('-' as char)
	}

	getter!(scheme, 0);
	getter!(username, 1);
	getter!(password, 2);
	getter!(host, 3);

	pub fn port(&self) -> u16 {
		let v: &str;
		match self.setter.as_ref() {
			Some(setter) => {
				v = setter.parts[4].as_str();
			}
			None => {
				v = self.port;
			}
		}

		if v.is_empty() {
			return 0;
		}

		match v.parse::<u16>() {
			Ok(num) => {
				return num;
			}
			Err(_) => {
				return 0;
			}
		}
	}

	getter!(path, 5);

	pub fn rawquery(&self, dest: &'a mut Option<Vec<u8>>) -> &'a str {
		match self.setter.as_ref() {
			Some(setter) => {
				if dest.is_none() {
					*dest = Some(Vec::with_capacity(256));
				}
				return setter.rawquery(dest.as_mut().unwrap());
			}
			None => {
				return self.rawquery;
			}
		}
	}

	pub fn query(&self) {}
}

#[cfg(test)]
mod tests {
	use crate::h2tp::url::Url;

	#[test]
	fn test_parse() {
		println!(
			"{:?}",
			Url::parse(
				"https://ztk:12133_=.dd@[fe80::1ff:fe23:4567:890a:4555]:8080/ddd?e=45fff#err"
			)
		);
		println!("{:?}", Url::parse("er@:45/"));
	}

	#[test]
	fn test_build() {
		let mut url = Url::new();
		url.builder().scheme("XXX").port(12555);
		println!("Scheme: {}", url.scheme());
	}

	#[test]
	fn test_url_builder_rawquery() {
		let mut url = Url::new();
		let mut builder = url.builder();
		unsafe {
			builder.rawquery("a=er&b=rtrt&rt&yui&我=3434&yui=90#err");
		}
		let query = builder.query();
		query.append("yui", "444");
		println!("{:?}", query);

		let mut x = vec![0; 3];
		println!("{}", x.capacity());
		x.reserve(0);
		println!("{}", x.capacity());
	}

	#[test]
	fn test_query() {
		let mut url = Url::parse("/?a=34&b=546").unwrap();
		let mut opt_dest = None;
		println!("{}", url.rawquery(&mut opt_dest));
		let mut builder = url.builder();
		let query = builder.query();
		query.append("x", "er");
		query.append(" x ", "😄");
		println!("{}", url.rawquery(&mut opt_dest));
		println!("{:?}", opt_dest);
	}
}
