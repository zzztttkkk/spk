use std::collections::HashMap;

struct AryMap {
	keys: Vec<String>,
	values: Vec<Box<Vec<String>>>,
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
				Some(&(*(self.values[idx])))
			}
			None => {
				None
			}
		};
	}

	fn getmut(&mut self, key: &str) -> Option<&mut Vec<String>> {
		return match self.find_idx(key) {
			Some(idx) => {
				Some(&mut (*(self.values[idx])))
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
				self.values.push(Box::new(vec![val.to_string()]));
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
				self.values.push(Box::new(vec![val.to_string()]));
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
	map: Option<HashMap<String, Box<Vec<String>>>>,
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
				match mapref.get(key) {
					Some(v) => {
						return Some(&(*v));
					}
					None => {
						None
					}
				}
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

	#[inline]
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
						mapref.insert(key.to_string(), Box::new(vec![val.to_string()]));
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
						unsafe {
							let np = aryref.values[i].clone();
							mapref.insert(k.to_string(), Box::from_raw(Box::into_raw(np)));
						}
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
						mapref.insert(key.to_string(), Box::new(vec![val.to_string()]));
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

	pub fn each(&self, func: fn(key: &str, val: &str)) {
		match self.map.as_ref() {
			Some(mapref) => {
				for (k, vs) in mapref.iter() {
					for v in vs.iter() {
						func(k, v);
					}
				}
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						for i in 0..aryref.keys.len() {
							let k = &aryref.keys[i];
							let vs = &aryref.values[i];
							for v in vs.iter() {
								func(k, v);
							}
						}
					}
					None => {}
				}
			}
		}
	}
}


#[cfg(test)]
mod tests {
	// use std::borrow::Borrow;
	use std::cell::{Ref, RefCell};
	use std::ops::Deref;
	use std::rc::Rc;
	use crate::h2tp::utils::multi_map::MultiMap;

	fn get_value(a: &Rc<RefCell<i32>>) -> Ref<i32> {
		return a.borrow();
	}

	#[test]
	fn test_ref() {
		let num = Rc::new(RefCell::new(12));
		println!("{}", get_value(&num));
	}

	#[test]
	fn test_multi_map() {
		let mut mm = MultiMap::new();
		mm.append("a", "12");
		mm.append("b", "23");
		mm.append("a", "56");
		mm.set("c", "66");

		mm.each(|k, v| {
			println!("K {}; V {}", k, v);
		});
	}
}