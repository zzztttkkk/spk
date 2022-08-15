use std::sync::{Arc};
use tokio::sync::Mutex;
use crate::h2tp::handler::Handler;

mod server;
mod conn;
mod cfg;
mod message;
mod utils;
mod headers;
mod handler;
mod error;
mod status_code;
mod methods;
mod request;
mod response;
mod url;
mod ctx;
mod types;

pub async fn shutdown(handler: &Arc<Mutex<server::ShutdownHandler>>, timout: u64) {
	let mut guard = handler.lock().await;
	if (*guard).shutdown(timout).await {
		println!("Graceful Shutdown OK");
	} else {
		println!("Graceful Shutdown Failed");
	}
}

pub fn server(handler: Option<Box<dyn Handler + Send + Sync>>) -> server::Server {
	return server::Server::new(handler);
}

pub type FuncHandler = handler::FuncHandler;

#[macro_export]
macro_rules! func {
    ($content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, _|{
				return Box::pin($content);
			})
		)
	};
 	(_, _, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, _|{
				return Box::pin($content);
			})
		)
	};
	($req:ident, _, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|$req, _|{
				return Box::pin($content);
			})
		)
	};
	(_, $resp:ident,  $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, $resp|{
				return Box::pin($content);
			})
		)
	};
	($req:ident, $resp:ident,  $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|$req, $resp|{
				return Box::pin($content);
			})
		)
	};
}