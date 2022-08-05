mod h2tp;

#[tokio::main]
async fn main() {
	let mut server = h2tp::create_server();
	tokio::spawn(async move {
		server.listen("127.0.0.1:8080").await;
	});

	match tokio::signal::ctrl_c().await {
		Ok(()) => {}
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
		}
	}
}
