use std::collections::HashMap;

pub enum Value {
	Text(String),
	Int(i64),
	Float(f64),
	Bool(bool),
	Array(Vec<Value>),
	Map(HashMap<String, Value>),
}
