use std::sync::Arc;
use tokio::sync::Mutex;

mod cfg;
mod conn;
mod error;
mod handler;
mod headers;
mod message;
mod methods;
mod request;
mod response;
mod server;
mod status_code;
mod types;
mod url;
mod utils;
mod router;
mod fs;

pub async fn shutdown(handler: &Arc<Mutex<server::ShutdownHandler>>, timout: u64) {
	let mut guard = handler.lock().await;
	if (*guard).shutdown(timout).await {
		println!("Graceful Shutdown OK");
	} else {
		println!("Graceful Shutdown Failed");
	}
}

pub fn server() -> server::Server {
	return server::Server::new();
}

pub use handler::FuncHandler;
pub use request::Request;
pub use response::Response;
pub use methods::*;
pub use headers::hns;

#[macro_export]
macro_rules! func {
	($content:expr) => {
		crate::h2tp::FuncHandler::new(|_, _| std::boxed::Box::pin(async move { $content }))
	};
	(_, _, $content:expr) => {
		crate::h2tp::FuncHandler::new(|_, _| std::boxed::Box::pin(async move { $content }))
	};
	($req:ident, _, $content:expr) => {
		crate::h2tp::FuncHandler::new(|$req, _| std::boxed::Box::pin(async move { $content }))
	};
	(_, $resp:ident, $content:expr) => {
		crate::h2tp::FuncHandler::new(|_, $resp| std::boxed::Box::pin(async move { $content }))
	};
	($req:ident, $resp:ident, $content:expr) => {
		crate::h2tp::FuncHandler::new(|$req, $resp| std::boxed::Box::pin(async move { $content }))
	};
}
