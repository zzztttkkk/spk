use std::collections::HashMap;

pub enum Value {
	Null,
	Number(f64),
	String(String),
	Bool(bool),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}