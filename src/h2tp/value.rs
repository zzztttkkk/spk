use crate::h2tp::response::Response;

pub trait Value {
	fn to(&self, resp: &mut Response);
}
