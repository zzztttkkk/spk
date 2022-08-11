use crate::h2tp::error::H2tpError;
use crate::h2tp::response::Response;
use crate::h2tp::status_code::StatusCode;

pub trait H2tpResult {
	fn to(&self, resp: &mut Response);
}

impl H2tpResult for String {
	fn to(&self, resp: &mut Response) {
		todo!()
	}
}

impl H2tpResult for StatusCode {
	fn to(&self, resp: &mut Response) {
		todo!()
	}
}

impl<T> H2tpResult for T where T: H2tpError {
	fn to(&self, resp: &mut Response) {
		todo!()
	}
}