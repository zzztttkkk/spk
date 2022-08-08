use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

type Values = Rc<RefCell<Vec<String>>>;

fn values(v: &str) -> Values {
	return Values::new(RefCell::new(vec![v.to_string()]));
}

struct AryMap {}

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
					Some(vals) => {
						vals.borrow_mut().push(v.to_string());
					}
					None => {
						mapref.insert(k.to_string(), values(v));
					}
				}
			}
			None => {}
		}
	}

	pub fn clear(&mut self) {}
}

