#![allow(dead_code)]

mod h2tp;
mod json;

#[tokio::main]
async fn main() {
	let mut server = h2tp::server(Some(
		Box::new(h2tp::FuncHandler::new(|req, _| {
			return Box::pin(async move {
				let mut g = req.lock().await;
				let req = &mut (*g);
				println!("{:?}", req);
				return Ok(());
			});
		}))
	));

	// server.tls("./dist/spk.local.pem", "./dist/spk.local-key.pem");

	let shutdownhandler = server.shutdownhandler();

	tokio::spawn(async move {
		server.listen("127.0.0.1:8080").await;
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
