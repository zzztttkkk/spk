mod server;
mod conn;
mod ordering;
mod packet;
mod utils;

pub fn create_server() -> server::Server {
	return server::Server::new();
}