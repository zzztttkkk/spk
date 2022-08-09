mod server;
mod conn;
mod cfg;
mod message;
mod utils;
mod headers;

pub fn create_server() -> server::Server {
	return server::Server::new();
}