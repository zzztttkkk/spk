use std::sync::{Arc};
use tokio::sync::Mutex;

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
mod result;

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

