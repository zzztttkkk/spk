use crate::h2tp::response::Response;

pub trait Result {
	fn to(resp: &mut Response);
}
