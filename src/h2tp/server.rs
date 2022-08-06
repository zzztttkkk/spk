use tokio::net::{TcpListener};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::h2tp::conn::Conn;

pub struct Server {
	listener: Option<TcpListener>,
	shutdown_signal_receiver: Option<UnboundedReceiver<()>>,
	before_shutdown: Option<fn()>,
	shutdown_done_sender: Option<UnboundedSender<()>>,
}

impl Server {
	pub fn new() -> Self {
		return Self {
			listener: None,
			shutdown_signal_receiver: None,
			before_shutdown: None,
			shutdown_done_sender: None,
		};
	}

	pub fn graceful_shutdown(&mut self, func: Option<fn()>) -> (UnboundedSender<()>, UnboundedReceiver<()>) {
		let (signal_sender, signal_receiver) = tokio::sync::mpsc::unbounded_channel();
		self.shutdown_signal_receiver = Some(signal_receiver);
		self.before_shutdown = func;
		let (done_sender, done_receiver) = tokio::sync::mpsc::unbounded_channel();
		self.shutdown_done_sender = Some(done_sender);
		return (signal_sender, done_receiver);
	}

	pub async fn listen<Addr: tokio::net::ToSocketAddrs>(&mut self, addr: Addr) {
		self.listener = Some(TcpListener::bind(addr).await.unwrap());
		let lref = self.listener.as_ref().unwrap();

		if self.shutdown_signal_receiver.is_none() {
			loop {
				match lref.accept().await {
					Ok((stream, _)) => {
						tokio::spawn(async move {
							let mut conn = Conn::new(stream);
							conn.handle().await;
						});
					}
					Err(_) => {}
				}
			}
		} else {
			let rref = self.shutdown_signal_receiver.as_mut().unwrap();

			loop {
				tokio::select! {
					result = lref.accept() => {
						match result {
							Ok((stream, _))=>{
								tokio::spawn(async move {
									let mut conn = Conn::new(stream);
									conn.handle().await;
								});
							},
							Err(_)=>{}
						}
					},
					_ = rref.recv() => {
						match self.before_shutdown {
							Some(f)=>{
								f();
							},
							None=>{}
						}
						break;
					}
				}
			}
		}
		println!("Graceful Shutdown");
	}
}
