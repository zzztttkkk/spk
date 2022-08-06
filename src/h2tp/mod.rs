mod server;
mod conn;
mod ordering;

pub fn create_server() -> server::Server {
	return server::Server::new();
}