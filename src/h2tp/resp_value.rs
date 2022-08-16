use crate::h2tp::response::Response;

pub trait ResponseValue {
	fn to(&self, resp: &mut Response);
}
