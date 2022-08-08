use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

type Values = Rc<RefCell<Vec<String>>>;

fn values(v: &str) -> Values {
	return Values::new(RefCell::new(vec![v.to_string()]));
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
				let mut vals = self.vals[idx].borrow_mut();
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
				let mut vals = self.vals[idx].borrow_mut();
				vals.clear();
				vals.push(v.to_string());
			}
			None => {}
		}
	}

	fn get(&self, k: &str) -> Option<Ref<Vec<String>>> {
		return match self.idx(k) {
			Some(idx) => {
				return Some(self.vals[idx].borrow());
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
						valsref.borrow_mut().push(v.to_string());
					}
					None => {
						mapref.insert(k.to_string(), values(v));
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
					let mut mapref = self.map.as_mut().unwrap();

					for idx in 0..aryref.keys.len() {
						let _k = &aryref.keys[idx];
						let _vals = aryref.vals[idx].clone();
						mapref.insert(_k.to_string(), _vals);
					}
				}
				aryref.append(k, v);
			}
		}
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
						let mut vals: RefMut<Vec<String>> = valsref.borrow_mut();
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

	pub fn get(&self, k: &str) -> Option<Ref<Vec<String>>> {
		return match self.map.as_ref() {
			Some(mapref) => {
				let valsref = mapref.get(k);
				match valsref {
					Some(valsref) => {
						Some(valsref.borrow())
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

	fn getone(&self, k: &str) -> Option<&String> {
		return match self.get(k) {
			Some(vals) => {
				let first = vals.first();
				match first {
					Some(e) => {

						unsafe {
							Some(&*(e as *const String))
						}
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
}

#[cfg(test)]
mod tests {
	use crate::h2tp::utils::multi_map::{AryMap, MultiMap};

	#[test]
	fn test_mm() {
		let mut mm = MultiMap::new();
		mm.append("a", "1");
		mm.clear();
		mm.remove("a");
		mm.append("a", "2");
		mm.append("a", "4");

		println!("{:?}", mm.get("a"));
		println!("{:?}", mm.getone("a"));
	}
}