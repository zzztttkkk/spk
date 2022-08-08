mod server;
mod conn;
mod cfg;
mod message;
mod utils;

pub fn create_server() -> server::Server {
	return server::Server::new();
}