use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::net::{TcpListener};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::h2tp::conn::Conn;
use crate::h2tp::cfg::ATOMIC_ORDERING;

pub struct Server {
	listener: Option<TcpListener>,
	closing: Arc<AtomicBool>,
	shutdown_signal_receiver: UnboundedReceiver<()>,
	shutdown_done_sender: UnboundedSender<()>,
	shutdownhandler: Arc<Mutex<ShutdownHandler>>,
}

pub struct ShutdownHandler {
	signal_sender: UnboundedSender<()>,
	done_receiver: UnboundedReceiver<()>,
}

impl ShutdownHandler {
	pub async fn shutdown(&mut self, ms: u64) -> bool {
		match self.signal_sender.send(()) {
			Ok(_) => {}
			Err(_) => {
				return false;
			}
		}
		tokio::select! {
			_ = self.done_receiver.recv() => {
				return true;
			},
			_ = sleep(Duration::from_millis(ms)) =>{
				return false;
			}
		}
	}
}

impl Server {
	pub fn new() -> Self {
		let (stx, srx) = unbounded_channel();
		let (dtx, drx) = unbounded_channel();
		let mut obj = Self {
			listener: None,
			closing: Arc::new(AtomicBool::new(false)),
			shutdown_signal_receiver: srx,
			shutdown_done_sender: dtx,
			shutdownhandler: Arc::new(Mutex::new(ShutdownHandler { signal_sender: stx, done_receiver: drx })),
		};
		return obj;
	}

	pub fn shutdownhandler(&self) -> Arc<Mutex<ShutdownHandler>> {
		return self.shutdownhandler.clone();
	}

	pub async fn listen<Addr: tokio::net::ToSocketAddrs>(&mut self, addr: Addr) {
		self.listener = Some(TcpListener::bind(addr).await.unwrap());
		let lref = self.listener.as_ref().unwrap();

		println!("Listening...");
		let alive_conn_count = Arc::new(AtomicU64::new(0));

		loop {
			tokio::select! {
					result = lref.accept() => {
						match result {
							Ok((stream, _))=>{
								let alive_conn_count_clone = Arc::clone(&alive_conn_count);
								let closing_clone = Arc::clone(&self.closing);
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
					_ = self.shutdown_signal_receiver.recv() => {
						self.closing.store(true, ATOMIC_ORDERING);
						println!("Closing...");
						break;
					}
			}
		}

		let duration = Duration::from_millis(100);
		loop {
			if alive_conn_count.load(ATOMIC_ORDERING) != 0 {
				sleep(duration).await;
				continue;
			}
			break;
		}
		self.shutdown_done_sender.send(()).err();
	}
}
