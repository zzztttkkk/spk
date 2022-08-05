mod server;
mod conn;

pub fn create_server() -> server::Server {
	return server::Server::new();
}