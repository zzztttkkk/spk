mod server;
mod conn;

pub fn create_server(threads: u8) -> server::Server {
	return server::Server::new(threads);
}