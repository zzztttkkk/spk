mod h2tp;

#[tokio::main]
async fn main() {
	let (shutdown_signal_sender, shutdown_signal_receiver) = tokio::sync::mpsc::unbounded_channel();

	let mut server = h2tp::create_server();
	let mut shutdown_done_receiver = server.graceful_shutdown(shutdown_signal_receiver, None);

	tokio::spawn(async move {
		server.listen("127.0.0.1:8080").await;
	});

	match tokio::signal::ctrl_c().await {
		Ok(()) => {
			shutdown_signal_sender.send(()).unwrap();
			shutdown_done_receiver.recv().await;
		}
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
		}
	}
}
