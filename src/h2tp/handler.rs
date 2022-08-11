use crate::h2tp::error::H2tpError;
use async_trait::async_trait;
use crate::h2tp::request::Request;
use crate::h2tp::response::Response;
use crate::h2tp::result::H2tpResult;

#[async_trait]
pub trait H2tpHandler<V: H2tpResult, E: H2tpError> {
	async fn handle(&self, req: &mut Request, resp: &mut Response) -> Result<Option<V>, E>;
}
