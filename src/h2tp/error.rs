use crate::h2tp::status_code::StatusCode;

pub trait H2tpError {
	fn status(&self) -> StatusCode;

	fn msg(&self) -> Option<&str>;
}