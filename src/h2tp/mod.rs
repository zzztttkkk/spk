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

pub fn create_server() -> server::Server {
	return server::Server::new();
}