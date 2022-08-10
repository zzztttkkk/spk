use crate::h2tp::error::H2tpError;
use async_trait::async_trait;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;

pub trait H2tpResult {
	fn to(&self, resp: &mut Response);
}

impl H2tpResult for String {
	fn to(&self, resp: &mut Response) {
		todo!()
	}
}

impl<T> H2tpResult for T where T: H2tpError {
	fn to(&self, resp: &mut Response) {
		todo!()
	}
}

#[async_trait]
pub trait H2tpHandler<V: H2tpResult, E: H2tpError> {
	async fn handle(&self, req: &mut Request, resp: &mut Response) -> Result<Option<V>, E>;
}
