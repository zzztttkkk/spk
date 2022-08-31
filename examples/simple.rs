use std::sync::Arc;
use spk::h2tp;

#[tokio::main]
async fn main() {
	let mut server = h2tp::server();

	// server.tls("./dist/spk.local.pem", "./dist/spk.local-key.pem");

	let shutdownhandler = server.shutdownhandler();

	tokio::spawn(async move {
		server
			.listen(
				"127.0.0.1:8080",
				Some(Arc::new(spk::func!(req, resp, {
					println!("{req:?}; Resp @ {resp:p}");
					let _ = std::io::Write::write(resp, b"Hello World!");
				}))),
			)
			.await;
	});

	match tokio::signal::ctrl_c().await {
		Ok(()) => {
			h2tp::shutdown(&shutdownhandler, 5000).await;
		}
		Err(err) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
		}
	}
}