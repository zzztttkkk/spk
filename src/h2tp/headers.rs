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

	pub async fn append(&mut self, k: &str, v: &str) {
		self.m.append(k, v).await;
	}

	pub fn clear(&mut self) {
		self.m.clear();
	}

	pub async fn content_length(&self) -> Option<usize> {
		let val: Option<&String>;
		unsafe {
			val = self.m.getone(CONTENT_LENGTH).await;
		}
		match val {
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

	pub async fn content_type(&self) -> Option<&String> {
		unsafe {
			return self.m.getone(CONTENT_TYPE).await;
		}
	}

	pub async fn transfer_encoding(&self) -> Option<&String> {
		unsafe {
			return self.m.getone(TRANSFER_ENCODING).await;
		}
	}

	pub async fn is_chunked(&self) -> bool {
		return match self.transfer_encoding().await {
			Some(v) => {
				v.contains("chunked")
			}
			None => {
				false
			}
		};
	}
}
