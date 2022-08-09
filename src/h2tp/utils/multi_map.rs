use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type Values = Arc<Mutex<RefCell<Vec<String>>>>;

fn values(v: &str) -> Values {
	return Values::new(Mutex::new(RefCell::new(vec![v.to_string()])));
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

	async fn append(&mut self, k: &str, v: &str) {
		match self.idx(k) {
			Some(idx) => {
				let mut guard = self.vals[idx].lock().await;
				let mut vals = (*guard).borrow_mut();
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

	async fn reset(&mut self, k: &str, v: &str) {
		match self.idx(k) {
			Some(idx) => {
				let mut guard = self.vals[idx].lock().await;
				let mut vals = (*guard).borrow_mut();

				vals.clear();
				vals.push(v.to_string());
			}
			None => {}
		}
	}

	async unsafe fn get(&self, k: &str) -> Option<&Vec<String>> {
		return match self.idx(k) {
			Some(idx) => {
				let mut guard = self.vals[idx].lock().await;
				let vals = (*guard).borrow();
				return Some(&(*(vals.as_ref() as *const Vec<String>)));
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

	pub async fn append(&mut self, k: &str, v: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let vals = mapref.get_mut(k);
				match vals {
					Some(valsref) => {
						let mut guard = valsref.lock().await;
						let mut valsref = (*guard).borrow_mut();
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

	pub async fn reset(&mut self, k: &str, v: &str) {
		match self.map.as_mut() {
			Some(mapref) => {
				let vals = mapref.get_mut(k);
				match vals {
					Some(valsref) => {
						let mut guard = valsref.lock().await;
						let mut vals: RefMut<Vec<String>> = (*guard).borrow_mut();
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

	pub async unsafe fn get(&self, k: &str) -> Option<&Vec<String>> {
		return match self.map.as_ref() {
			Some(mapref) => {
				let valsref = mapref.get(k);
				match valsref {
					Some(valsref) => {
						let guard = valsref.lock().await;
						let valsref = (*guard).borrow();
						Some(&(*(valsref.as_ref() as *const Vec<String>)))
					}
					None => {
						None
					}
				}
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						aryref.get(k).await
					}
					None => {
						None
					}
				}
			}
		};
	}

	pub async unsafe fn getone(&self, k: &str) -> Option<&String> {
		return match self.get(k).await {
			Some(vals) => {
				vals.first()
			}
			None => {
				None
			}
		};
	}

	pub async fn each(&self, func: fn(k: &str, v: &str)) {
		match self.map.as_ref() {
			Some(mapref) => {
				for (k, valsref) in mapref.iter() {
					let guard = valsref.lock().await;
					for v in (*guard).borrow().iter() {
						func(k, v);
					}
				}
			}
			None => {
				match self.ary.as_ref() {
					Some(aryref) => {
						for i in 0..aryref.keys.len() {
							let k = &aryref.keys[i];
							let guard = aryref.vals[i].lock().await;
							let valsref = (*guard).borrow();
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
	use crate::h2tp::utils::multi_map::{AryMap, MultiMap};

	#[test]
	fn test_mm() {
		tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(
			async {
				let mut mm = MultiMap::new();
				mm.append("a", "1").await;
				mm.clear();
				mm.remove("a");
				mm.append("a", "2").await;
				mm.append("a", "4").await;
			}
		);
	}
}