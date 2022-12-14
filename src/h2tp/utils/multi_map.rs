use std::{collections::HashMap, fmt};

/// 使用`RefCell<Vec<String>>`也可以，同时还能避免交换位置时发生`memcopy`。
/// 但`Vec`只是一个很小很简单的结构体，即使发生`memcopy`，代价也比`RefCell`要小。
/// 没有具体的性能测试，但是感觉至少相差不大。所以不如保持简单。
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
		return self.keys.iter().position(|e| {
			return e == k;
		});
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
			None => {
				self.keys.push(k.to_string());
				self.vals.push(values(v));
			}
		}
	}

	fn get(&self, k: &str) -> Option<&Vec<String>> {
		match self.idx(k) {
			Some(idx) => Some(&self.vals[idx]),
			None => None,
		}
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
						valsref.push(v.to_string());
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
					self.swap_ary_to_hashmap()
				} else {
					aryref.append(k, v);
				}
			}
		}
	}

	fn swap_ary_to_hashmap(&mut self) {
		let mut ary = self.ary.take().unwrap();
		let mut map = HashMap::with_capacity(ary.keys.len());
		while !ary.keys.is_empty() {
			map.insert(ary.keys.pop().unwrap(), ary.vals.pop().unwrap());
		}
		self.map = Some(map);
	}

	pub fn clear(&mut self) {
		match self.map.as_mut() {
			Some(mapref) => {
				mapref.clear();
			}
			None => match self.ary.as_mut() {
				Some(aryref) => {
					aryref.clear();
				}
				None => {}
			},
		}
	}

	pub fn remove(&mut self, k: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				mapref.remove(k);
			}
			None => match self.ary.as_mut() {
				Some(aryref) => {
					aryref.remove(k);
				}
				None => {}
			},
		}
	}

	pub fn reset(&mut self, k: &str, v: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let vals = mapref.get_mut(k);
				match vals {
					Some(vals) => {
						vals.clear();
						vals.push(v.to_string());
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
				aryref.reset(k, v);
			}
		}
	}

	pub fn get(&self, k: &str) -> Option<&Vec<String>> {
		return match self.map.as_ref() {
			Some(mapref) => mapref.get(k),
			None => match self.ary.as_ref() {
				Some(aryref) => aryref.get(k),
				None => None,
			},
		};
	}

	pub fn getone(&self, k: &str) -> Option<&String> {
		return match self.get(k) {
			Some(vals) => vals.first(),
			None => None,
		};
	}

	pub fn eachmut<F>(&mut self, mut func: F)
	where
		F: FnMut(&str, &mut str, bool) -> bool,
	{
		match self.map.as_mut() {
			Some(mapref) => {
				let l1 = mapref.len();
				let mut i = 0;
				for (k, valsref) in mapref.iter_mut() {
					let l2 = valsref.len();
					for j in 0..l2 {
						if !func(k, &mut (valsref[j]), i == l1 - 1 && j == l2 - 1) {
							break;
						};
					}
					i += 1;
				}
			}
			None => match self.ary.as_mut() {
				Some(aryref) => {
					let l1 = aryref.keys.len();

					for i in 0..l1 {
						let k = &aryref.keys[i];
						let valsref = &mut aryref.vals[i];
						let l2 = valsref.len();
						for j in 0..l2 {
							if !func(k, &mut valsref[j], i == l1 - 1 && j == l2 - 1) {
								break;
							}
						}
					}
				}
				None => {}
			},
		}
	}

	pub fn each<F>(&self, mut func: F)
	where
		F: FnMut(&str, &str, bool) -> bool,
	{
		match self.map.as_ref() {
			Some(mapref) => {
				let l1 = mapref.len();
				let mut i = 0;
				for (k, valsref) in mapref.iter() {
					let l2 = valsref.len();
					for j in 0..l2 {
						if !func(k, &(valsref[j]), i == l1 - 1 && j == l2 - 1) {
							break;
						};
					}
					i += 1;
				}
			}
			None => match self.ary.as_ref() {
				Some(aryref) => {
					let l1 = aryref.keys.len();

					for i in 0..l1 {
						let k = &aryref.keys[i];
						let valsref = &aryref.vals[i];
						let l2 = valsref.len();
						for j in 0..l2 {
							if !func(k, &valsref[j], i == l1 - 1 && j == l2 - 1) {
								break;
							}
						}
					}
				}
				None => {}
			},
		}
	}
}

impl fmt::Debug for MultiMap {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "MultiMap(")?;
		self.each(|k, v, is_last| {
			if is_last {
				let _ = write!(f, " {} = {}", k, v);
			} else {
				let _ = write!(f, " {} = {};", k, v);
			}
			return true;
		});
		write!(f, ")")
	}
}

#[cfg(test)]
mod tests {
	use crate::h2tp::utils::multi_map::MultiMap;
	use core::fmt;
	use std::fmt::Formatter;

	#[test]
	fn test_mm() {
		let mut mm = MultiMap::new();
		mm.append("a", "1");
		mm.clear();
		mm.remove("a");
		mm.append("a", "2");
		mm.append("a", "423");

		println!("{:?}", mm);
	}

	#[test]
	fn test_swap() {
		let mut mm = MultiMap::new();

		for i in 0..40 {
			mm.append(&format!("k{}", i), &format!("v{}", i));
		}

		println!("{:?}", mm);
	}

	struct Obj {
		num: i32,
	}

	impl Obj {
		fn new(m: i32) -> Self {
			return Self { num: m };
		}
	}

	impl fmt::Debug for Obj {
		fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
			write!(f, "{} @ {}", self.num, (self as *const Obj as u64))
		}
	}

	impl Drop for Obj {
		fn drop(&mut self) {
			println!("Dropped {} @ {}", self.num, (self as *const Obj as u64));
		}
	}

	#[test]
	fn test_option_take() {
		let mut opt = Some(vec![Obj::new(1), Obj::new(2), Obj::new(3)]);
		println!("{:?}", opt);
		let vs = opt.take().unwrap();
		println!("{:?}", vs);
	}
}
