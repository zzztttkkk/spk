use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::path::{Path};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_rustls::{rustls, TlsAcceptor};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use crate::h2tp::conn::Conn;
use crate::h2tp::cfg::ATOMIC_ORDERING;


struct Tls {
	cert: String,
	key: String,
}

impl Tls {
	pub fn load(&self) -> (Vec<Certificate>, Vec<PrivateKey>) {
		let mut certs = Vec::new();
		for e in rustls_pemfile::certs(
			&mut BufReader::new(File::open(Path::new(self.cert.as_str())).unwrap())
		).unwrap() {
			certs.push(Certificate(e));
		}
		let mut keys = Vec::new();
		for e in rustls_pemfile::rsa_private_keys(
			&mut BufReader::new(File::open(Path::new(self.key.as_str())).unwrap())
		).unwrap() {
			keys.push(PrivateKey(e));
		}
		return (certs, keys);
	}
}

pub struct Server {
	listener: Option<TcpListener>,
	tls: Option<Tls>,
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
		return Self {
			listener: None,
			tls: None,
			shutdown_signal_receiver: srx,
			shutdown_done_sender: dtx,
			shutdownhandler: Arc::new(Mutex::new(ShutdownHandler { signal_sender: stx, done_receiver: drx })),
		};
	}

	pub fn shutdownhandler(&self) -> Arc<Mutex<ShutdownHandler>> {
		return self.shutdownhandler.clone();
	}

	#[inline(always)]
	fn spawn(stream: TcpStream, acc: &Arc<AtomicU64>, c: &Arc<AtomicBool>, _: &Option<TlsAcceptor>) {
		let accc = Arc::clone(acc);
		let cc = Arc::clone(c);
		tokio::spawn(async move {
			accc.fetch_add(1, ATOMIC_ORDERING);
			let mut ms = stream;
			let (r, w) = ms.split();
			let mut conn = Conn::new(r, w, Some(cc));
			conn.handle().await;
			accc.fetch_sub(1, ATOMIC_ORDERING);
		});
	}

	#[inline(always)]
	fn spawn_tls(stream: TcpStream, acc: &Arc<AtomicU64>, c: &Arc<AtomicBool>, acceptor: &Option<TlsAcceptor>) {
		let accc = Arc::clone(acc);
		let cc = Arc::clone(c);
		let ac = acceptor.as_ref().unwrap().clone();
		tokio::spawn(async move {
			match ac.accept(stream).await {
				Ok(stream) => {
					accc.fetch_add(1, ATOMIC_ORDERING);
					// https://github.com/rustls/rustls/issues/288
					// https://github.com/tokio-rs/tokio/issues/1108
					let (r, w) = tokio::io::split(stream);
					let mut conn = Conn::new(r, w, Some(cc));
					conn.handle().await;
					accc.fetch_sub(1, ATOMIC_ORDERING);
				}
				Err(_) => {}
			}
		});
	}

	pub async fn listen<Addr: tokio::net::ToSocketAddrs>(&mut self, addr: Addr) {
		self.listener = Some(TcpListener::bind(addr).await.unwrap());

		let mut tls_acceptor: Option<TlsAcceptor> = None;
		match self.tls.as_ref() {
			Some(tls) => {
				let (certs, mut keys) = tls.load();
				let tls_cfg = rustls::ServerConfig::builder()
					.with_safe_defaults()
					.with_no_client_auth()
					.with_single_cert(certs, keys.remove(0)).unwrap();
				tls_acceptor = Some(tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg)));
			}
			None => {}
		}


		println!("Listening...");
		let alive_conn_count = Arc::new(AtomicU64::new(0));
		let closing = Arc::new(AtomicBool::new(false));

		let spawn: fn(TcpStream, &Arc<AtomicU64>, &Arc<AtomicBool>, &Option<TlsAcceptor>) -> ();
		if tls_acceptor.is_some() {
			spawn = Self::spawn_tls;
		} else {
			spawn = Self::spawn;
		}

		let lref = self.listener.as_ref().unwrap();

		loop {
			tokio::select! {
				result = lref.accept() => {
					match result {
						Ok((stream, _)) => {
							spawn(stream, &alive_conn_count, &closing, &tls_acceptor);
						}
						Err(_)=>{}
					}
				},
				_ = self.shutdown_signal_receiver.recv() => {
						closing.store(true, ATOMIC_ORDERING);
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
