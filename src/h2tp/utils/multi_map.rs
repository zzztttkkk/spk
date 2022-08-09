// use std::cell::{Ref};
use std::collections::HashMap;

// Arc<Mutex<RefCell<Vec<String>>>>
type Values = Vec<String>;

fn values(v: &str) -> Values {
	return Values::from(vec![v.to_string()]);
}

struct AryMap {
	keys: Vec<String>,
	vals: Vec<Values>,
}

impl AryMap {
	fn new() -> Self {
		return Self {
			keys: vec![],
			vals: vec![],
		};
	}

	fn idx(&self, k: &str) -> Option<usize> {
		return self.keys.iter().position(|e| { return e == k; });
	}

	fn append(&mut self, k: &str, v: &str) {
		match self.idx(k) {
			Some(idx) => {
				let vals = &mut self.vals[idx];
				vals.push(v.to_string());
			}
			None => {
				self.keys.push(k.to_string());
				self.vals.push(values(v));
			}
		}
	}

	fn clear(&mut self) {
		self.keys.clear();
		self.vals.clear();
	}

	fn remove(&mut self, k: &str) {
		match self.idx(k) {
			Some(idx) => {
				self.keys.remove(idx);
				self.vals.remove(idx);
			}
			None => {}
		}
	}

	fn reset(&mut self, k: &str, v: &str) {
		match self.idx(k) {
			Some(idx) => {
				let vals = &mut self.vals[idx];
				vals.clear();
				vals.push(v.to_string());
			}
			None => {}
		}
	}

	fn get<'a>(&'a self, k: &str) -> Option<&'a Vec<String>> {
		return match self.idx(k) {
			Some(idx) => {
				return Some(&self.vals[idx]);
			}
			None => {
				None
			}
		};
	}
}

pub struct MultiMap {
	ary: Option<AryMap>,
	map: Option<HashMap<String, Values>>,
}

impl MultiMap {
	pub fn new() -> Self {
		return Self {
			ary: None,
			map: None,
		};
	}

	pub fn append(&mut self, k: &str, v: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let vals = mapref.get_mut(k);
				match vals {
					Some(valsref) => {
						let valsref = valsref;
						valsref.push(v.to_string());
					}
					None => {
						mapref.insert(k.to_string(), values(v));
					}
				}
			}
			None => {
				// 拿 ary 的不可變引用，進行判斷。
				if let Some(ref ary) = self.ary {
					// 判斷 keys 有沒有 >= 12，如果有則進行置換。
					if ary.keys.len() >= 12 {
						self.swap_ary_to_hashmap();
					} else {
						// 拿 ary 的可變引用，並 append。
						self.ary.as_mut().unwrap().append(k, v);
					}
				} else {
					self.ary = Some(AryMap::new());
				}
			}
		}
	}

	/// 將目前 ary 的內容 swap 到 [`HashMap`]。
	fn swap_ary_to_hashmap(&mut self) -> Option<()> {
		// 取走 ary 的所有權。
		let mut ary = self.ary.take()?;

		// 建立一個 HashMap，並預先分配與 ary.keys 同等長度的空間。
		let mut map = HashMap::with_capacity(ary.keys.len());

		// 然後把 ary 全部置換到 HashMap。
		while !ary.keys.is_empty() {
			// HashMap 是無序的，所以我們只要無序 pop，
			// 直到 ary.keys 一點也不剩即可。
			let key = ary.keys.pop().expect("should be Some");
			let val = ary.vals.pop().expect("val is bound to key");

			map.insert(key, val);
		}

		// 把建立的 HashMap 放到 self.map。
		self.map = Some(map);

		Some(())
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

	pub fn remove(&mut self, k: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				mapref.remove(k);
			}
			None => {
				match self.ary.as_mut() {
					Some(aryref) => {
						aryref.remove(k);
					}
					None => {}
				}
			}
		}
	}

	pub fn reset(&mut self, k: &str, v: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let vals = mapref.get_mut(k);
				match vals {
					Some(valsref) => {
						let mut vals = valsref;
						vals.clear();
						vals.push(v.to_string());
					}
					None => {
						mapref.insert(k.to_string(), values(v));
					}
				}
			}
			None => {
				match self.ary.as_mut() {
					Some(aryref) => {
						aryref.reset(k, v);
					}
					None => {}
				}
			}
		}
	}

	pub fn get(&self, k: &str) -> Option<&Vec<String>> {
		return match self.map.as_ref() {
			Some(mapref) => {
				let valsref = mapref.get(k);
				match valsref {
					Some(valsref) => {
						Some(valsref)
					}
					None => {
						None
					}
				}
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						aryref.get(k)
					}
					None => {
						None
					}
				}
			}
		};
	}

	pub fn getone(&self, k: &str) -> Option<&String> {
		return match self.get(k) {
			Some(vals) => {
				match vals.first() {
					Some(ele) => {
						Some(ele)
					}
					None => {
						None
					}
				}
			}
			None => {
				None
			}
		};
	}

	pub fn each(&self, func: fn(k: &str, v: &str)) {
		match self.map.as_ref() {
			Some(mapref) => {
				for (k, valsref) in mapref.iter() {
					for v in valsref.iter() {
						func(k, v);
					}
				}
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						for i in 0..aryref.keys.len() {
							let k = &aryref.keys[i];
							let valsref = &aryref.vals[i];
							for v in valsref.iter() {
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
	use crate::h2tp::utils::multi_map::{MultiMap};

	#[test]
	fn test_mm() {
		let mut mm = MultiMap::new();
		mm.append("a", "1");
		mm.clear();
		mm.remove("a");
		mm.append("a", "2");
		mm.append("a", "4");
	}

	#[test]
	fn test_swap() {
		let mut mm = MultiMap::new();

		for i in 0..40 {
			mm.append(&format!("k{}", i), &format!("v{}", i));
		}
	}
}
