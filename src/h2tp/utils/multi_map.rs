use std::collections::HashMap;

struct AryMap {
	keys: Vec<String>,
	values: Vec<Vec<String>>,
}

impl AryMap {
	pub fn new() -> Self {
		return Self {
			keys: vec![],
			values: vec![],
		};
	}

	fn find_idx(&self, key: &str) -> Option<usize> {
		return self.keys.iter().position(|e| {
			return e == key;
		});
	}

	pub fn get(&self, key: &str) -> Option<&Vec<String>> {
		return match self.find_idx(key) {
			Some(idx) => {
				Some(&(self.values[idx]))
			}
			None => {
				None
			}
		};
	}

	fn getmut(&mut self, key: &str) -> Option<&mut Vec<String>> {
		return match self.find_idx(key) {
			Some(idx) => {
				Some(&mut (self.values[idx]))
			}
			None => {
				None
			}
		};
	}

	fn append(&mut self, key: &str, val: &str) {
		match self.getmut(key) {
			Some(v) => {
				v.push(val.to_string());
			}
			None => {
				self.keys.push(key.to_string());
				self.values.push(vec![val.to_string()]);
			}
		}
	}

	fn set(&mut self, key: &str, val: &str) {
		match self.getmut(key) {
			Some(v) => {
				v.clear();
				v.push(val.to_string());
			}
			None => {
				self.keys.push(key.to_string());
				self.values.push(vec![val.to_string()]);
			}
		}
	}

	fn clear(&mut self) {
		self.keys.clear();
		self.values.clear();
	}

	fn remove(&mut self, key: &str) {
		return match self.find_idx(key) {
			Some(idx) => {
				self.keys.remove(idx);
				self.values.remove(idx);
			}
			None => {}
		};
	}

	fn each(&self, f: fn(key: &str, val: &str)) {
		for i in 0..self.keys.len() {
			let key = &self.keys[i];
			let vals = &self.values[i];
			for j in 0..vals.len() {
				f(key, &vals[j]);
			}
		}
	}
}

pub struct MultiMap {
	ary: Option<AryMap>,
	map: Option<HashMap<String, Vec<String>>>,
}

impl MultiMap {
	pub fn new() -> Self {
		return Self {
			ary: None,
			map: None,
		};
	}

	pub fn get(&self, key: &str) -> Option<&Vec<String>> {
		return match self.map.as_ref() {
			Some(mapref) => {
				mapref.get(key)
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						aryref.get(key)
					}
					None => {
						None
					}
				}
			}
		};
	}

	pub fn getone(&self, key: &str) -> Option<&String> {
		match self.get(key) {
			Some(v) => {
				if v.is_empty() {
					return None;
				}
				return Some(&(v[0]));
			}
			None => {
				None
			}
		}
	}

	pub fn exists(&self, key: &str) -> bool {
		return self.get(key).is_some();
	}

	pub fn count(&self, key: &str) -> usize {
		return match self.get(key) {
			Some(v) => {
				v.len()
			}
			None => {
				0
			}
		};
	}

	pub fn clear(&mut self) {
		match self.map.as_mut() {
			Some(mapref) => {
				mapref.clear();
			}
			None => {
				match self.ary.as_mut() {
					Some(aryref) => {
						aryref.clear();
					}
					None => {}
				}
			}
		}
	}

	pub fn remove(&mut self, key: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				mapref.remove(key);
			}
			None => {
				match self.ary.as_mut() {
					Some(aryref) => {
						aryref.remove(key);
					}
					None => {}
				}
			}
		}
	}

	pub fn append(&mut self, key: &str, val: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let v = mapref.get_mut(key);
				match v {
					Some(v) => {
						v.push(val.to_string());
					}
					None => {
						mapref.insert(key.to_string(), vec![val.to_string()]);
					}
				}
			}
			None => {
				if self.ary.is_none() {
					self.ary = Some(AryMap::new());
				}

				let aryref = self.ary.as_mut().unwrap();
				if aryref.keys.len() >= 12 {
					self.map = Some(HashMap::new());
					let mapref = self.map.as_mut().unwrap();
					for i in 0..aryref.keys.len() {
						let k = &aryref.keys[i];
						let vs = &aryref.values[i];
						// todo more effective copy
						let mut vsc = Vec::new();
						for v in vs {
							vsc.push(v.as_str().to_string());
						}
						mapref.insert(k.to_string(), vsc);
					}
					self.ary = None;
					return;
				}
				aryref.append(key, val);
			}
		}
	}

	pub fn set(&mut self, key: &str, val: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let v = mapref.get_mut(key);
				match v {
					Some(v) => {
						v.clear();
						v.push(val.to_string());
					}
					None => {
						mapref.insert(key.to_string(), vec![val.to_string()]);
					}
				}
			}
			None => {
				if self.ary.is_none() {
					self.ary = Some(AryMap::new());
				}
				let aryref = self.ary.as_mut().unwrap();
				aryref.set(key, val);
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use crate::h2tp::utils::multi_map::AryMap;

	#[test]
	fn test_multi_map() {
		let mut m = AryMap::new();
		m.append("a", "12");
		m.append("a", "45");
		m.set("a", "456");

		match m.get("a") {
			Some(v) => {
				println!("{}", v.join(","));
			}
			None => {}
		}

		m.each(|key, val| {
			println!("K: {} V: {}", key, val)
		});
	}
}