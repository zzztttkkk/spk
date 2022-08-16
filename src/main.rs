#![allow(dead_code)]

use std::sync::Arc;
use crate::h2tp::FuncHandler;

mod h2tp;
mod json;

#[tokio::main]
async fn main() {
	let mut server = h2tp::server();

	// server.tls("./dist/spk.local.pem", "./dist/spk.local-key.pem");

	let shutdownhandler = server.shutdownhandler();

	tokio::spawn(async move {
		server.listen("127.0.0.1:8080", Some(Arc::new(FuncHandler::new(|req, resp| {
			return Box::pin(async move {
				println!("{req:?}; Resp @ {resp:p}");
				println!("{}", req.method());
				return Ok(None);
			});
		})))).await;
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
