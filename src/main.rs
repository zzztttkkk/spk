mod h2tp;

#[tokio::main]
async fn main() {
	let mut server = h2tp::create_server();
	let (shutdown_signal_sender, mut shutdown_done_receiver) = server.graceful_shutdown(5000, None);

	tokio::spawn(async move {
		server.listen("127.0.0.1:8080").await;
	});

	match tokio::signal::ctrl_c().await {
		Ok(()) => {
			shutdown_signal_sender.send(()).unwrap();
			match shutdown_done_receiver.recv().await {
				Some(ok) => {
					if ok {
						println!("Graceful Shutdown Done.");
					} else {
						println!("Shutdown Timeout.");
					}
				}
				None => {
					println!("Graceful Shutdown Failed.");
				}
			}
		}
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
		}
	}
}
