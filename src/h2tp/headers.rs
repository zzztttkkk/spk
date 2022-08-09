use crate::h2tp::utils::multi_map::MultiMap;

pub struct Headers {
	m: MultiMap,
}

const CONTENT_LENGTH: &str = "content-length";
const CONTENT_TYPE: &str = "content-type";
const TRANSFER_ENCODING: &str = "transfer-encoding";

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
		match self.m.getone(CONTENT_LENGTH) {
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

	pub fn content_type(&self) -> Option<&String> {
		return self.m.getone(CONTENT_TYPE);
	}

	pub fn transfer_encoding(&self) -> Option<&String> {
		return self.m.getone(TRANSFER_ENCODING);
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
