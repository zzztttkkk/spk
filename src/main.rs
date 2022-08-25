#![allow(dead_code)]

use std::io::Write;
use std::sync::Arc;

mod h2tp;

use ini::HelloMacro;

pub trait HelloMacro {
	fn hello_macro();
}

#[derive(HelloMacro)]
struct X {}

#[tokio::main]
async fn main() {
	X::hello_macro();

	let mut server = h2tp::server();

	// server.tls("./dist/spk.local.pem", "./dist/spk.local-key.pem");

	let shutdownhandler = server.shutdownhandler();

	tokio::spawn(async move {
		server
			.listen(
				"127.0.0.1:8080",
				Some(Arc::new(crate::func!(req, resp, {
					println!("{req:?}; Resp @ {resp:p}");
					let _ = resp.write(b"Hello World!");
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
