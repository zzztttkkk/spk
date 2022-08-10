extern crate core;

mod h2tp;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	#[clap(short, long, default_value = "127.0.0.1:8080")]
	addr: String,

	#[clap(short, long, default_value_t = 3000)]
	shutdown_timeout: u64,
}

#[tokio::main]
async fn main() {
	let mut args: Args = Args::parse();
	if args.shutdown_timeout < 1000 {
		args.shutdown_timeout = 1000;
	}
	println!("{:?}", args);

	let mut server = h2tp::create_server();

	let (shutdown_signal_sender, mut shutdown_done_receiver) = server.graceful_shutdown(args.shutdown_timeout, None);

	tokio::spawn(async move {
		server.listen(args.addr).await;
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
