mod h2tp;

#[tokio::main]
async fn main() {
	let mut server = h2tp::create_server();
	let (shutdown_signal_sender, mut shutdown_done_receiver) = server.graceful_shutdown(None);

	tokio::spawn(async move {
		server.listen("127.0.0.1:8080").await;
	});

	match tokio::signal::ctrl_c().await {
		Ok(()) => {
			shutdown_signal_sender.send(()).unwrap();
			shutdown_done_receiver.recv().await;
			println!("Graceful Shutdown.");
		}
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
		}
	}
}
