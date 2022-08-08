use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::net::{TcpListener};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;
use crate::h2tp::conn::Conn;
use crate::h2tp::cfg::ATOMIC_ORDERING;

pub struct Server {
	listener: Option<TcpListener>,
	shutdown_signal_receiver: Option<UnboundedReceiver<()>>,
	before_shutdown: Option<fn()>,
	shutdown_done_sender: Option<UnboundedSender<bool>>,
	shutdown_timeout: u64,
}

impl Server {
	pub fn new() -> Self {
		return Self {
			listener: None,
			shutdown_signal_receiver: None,
			before_shutdown: None,
			shutdown_done_sender: None,
			shutdown_timeout: 0,
		};
	}

	pub fn graceful_shutdown(&mut self, timeout: u64, func: Option<fn()>) -> (UnboundedSender<()>, UnboundedReceiver<bool>) {
		self.shutdown_timeout = timeout;
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

		println!("Listening...");

		if self.shutdown_signal_receiver.is_none() {
			loop {
				match lref.accept().await {
					Ok((stream, _)) => {
						tokio::spawn(async move {
							let mut conn = Conn::new(stream, None);
							conn.handle().await;
						});
					}
					Err(_) => {}
				}
			}
		} else {
			let rref = self.shutdown_signal_receiver.as_mut().unwrap();
			let alive_conn_count = Arc::new(AtomicU64::new(0));
			let closing = Arc::new(AtomicBool::new(false));

			loop {
				tokio::select! {
					result = lref.accept() => {
						match result {
							Ok((stream, _))=>{
								let alive_conn_count_clone = Arc::clone(&alive_conn_count);
								let closing_clone = Arc::clone(&closing);
								tokio::spawn(async move {
									alive_conn_count_clone.fetch_add(1, ATOMIC_ORDERING);
									let mut conn = Conn::new(stream, Some(closing_clone));
									conn.handle().await;
									alive_conn_count_clone.fetch_sub(1, ATOMIC_ORDERING);
								});
							},
							Err(_)=>{}
						}
					},
					_ = rref.recv() => {
						match self.before_shutdown {
							Some(before_shutdown)=>{
								before_shutdown();
							},
							None=>{}
						}
						closing.store(true, ATOMIC_ORDERING);
						println!("Closing...");
						break;
					}
				}
			}

			let mut timeout = self.shutdown_timeout;
			if timeout < 3000 {
				timeout = 3000;
			}
			let (q, r) = (timeout / 100, timeout % 100);
			if r != 0 {
				timeout = (q + 1) * 100;
			}

			let duration = Duration::from_millis(100);
			loop {
				if alive_conn_count.load(ATOMIC_ORDERING) != 0 {
					sleep(duration).await;
					timeout -= 100;
					if timeout == 0 {
						break;
					}
					continue;
				}
				break;
			}

			self.shutdown_done_sender.as_ref().unwrap().send(alive_conn_count.load(ATOMIC_ORDERING) == 0).err();
		}
	}
}
