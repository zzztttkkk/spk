use crate::h2tp::cfg::ATOMIC_ORDERING;
use crate::h2tp::conn::Conn;
use crate::h2tp::handler::Handler;
use crate::h2tp::FuncHandler;
use core::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;

use super::types::{AsyncReader, AsyncWriter};

struct Tls {
	cert: String,
	key: String,
}

impl Tls {
	pub fn load(&self) -> ServerConfig {
		let mut certs = Vec::new();
		for e in rustls_pemfile::certs(&mut BufReader::new(
			File::open(Path::new(self.cert.as_str())).unwrap(),
		))
		.unwrap()
		{
			certs.push(Certificate(e));
		}
		let mut keys = Vec::new();
		for e in rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(
			File::open(Path::new(self.key.as_str())).unwrap(),
		))
		.unwrap()
		{
			keys.push(PrivateKey(e));
		}
		if keys.is_empty() {
			for e in rustls_pemfile::rsa_private_keys(&mut BufReader::new(
				File::open(Path::new(self.key.as_str())).unwrap(),
			))
			.unwrap()
			{
				keys.push(PrivateKey(e));
			}
		}
		return ServerConfig::builder()
			.with_safe_defaults()
			.with_no_client_auth()
			.with_single_cert(certs, keys.remove(0))
			.unwrap();
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

pub trait PrintableToSocketAddrs: tokio::net::ToSocketAddrs + fmt::Display + Copy {}

impl<T> PrintableToSocketAddrs for T where T: tokio::net::ToSocketAddrs + fmt::Display + Copy {}

impl Server {
	pub fn new() -> Self {
		let (stx, srx) = unbounded_channel();
		let (dtx, drx) = unbounded_channel();

		return Self {
			listener: None,
			tls: None,
			shutdown_signal_receiver: srx,
			shutdown_done_sender: dtx,
			shutdownhandler: Arc::new(Mutex::new(ShutdownHandler {
				signal_sender: stx,
				done_receiver: drx,
			})),
		};
	}

	pub fn tls(&mut self, cert: &str, key: &str) {
		self.tls = Some(Tls {
			cert: cert.to_string(),
			key: key.to_string(),
		});
	}

	pub fn shutdownhandler(&self) -> Arc<Mutex<ShutdownHandler>> {
		return self.shutdownhandler.clone();
	}

	pub async fn listen<Addr: PrintableToSocketAddrs, R, W>(
		&mut self,
		addr: Addr,
		h: Option<Arc<dyn Handler<R, W> + Send + Sync>>,
	) where
		R: AsyncReader,
		W: AsyncWriter,
	{
		self.listener = Some(TcpListener::bind(addr).await.unwrap());

		let mut tls_acceptor: Option<TlsAcceptor> = None;
		match self.tls.as_ref() {
			Some(tls) => {
				let tls_cfg = tls.load();
				tls_acceptor = Some(tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg)));
				println!("TLS OK");
			}
			None => {}
		}

		println!("Listening @ {}...", addr);

		let alive_conn_count = Arc::new(AtomicU64::new(0));
		let closing = Arc::new(AtomicBool::new(false));
		let lref = self.listener.as_ref().unwrap();

		let handler = match h {
			Some(v) => v,
			None => Arc::new(FuncHandler::new(|req, _| {
				Box::pin(async move {
					println!("{req:?}");
					Ok(())
				})
			})) as _,
		};

		loop {
			tokio::select! {
				result = lref.accept() => {
					match result {
						Ok((mut stream, addr)) => {
							let accc = Arc::clone(&alive_conn_count);
							let cc = Arc::clone(&closing);
							let hc = Arc::clone(&handler);

							match tls_acceptor.as_ref() {
								Some(tls)=>{
									let acceptor = tls.clone();
									tokio::spawn(async move {
										match acceptor.accept(stream).await {
											Ok(tls_stream) => {
												accc.fetch_add(1, ATOMIC_ORDERING);
												// https://github.com/rustls/rustls/issues/288
												// https://github.com/tokio-rs/tokio/issues/1108
												let (r, w) = tokio::io::split(tls_stream);
												let mut conn = Conn::new(addr, r, w, cc);
												conn.as_server(hc).await;
												accc.fetch_sub(1, ATOMIC_ORDERING);
											}
											Err(_) => {}
										}
									});
								}
								None=>{
									tokio::spawn(async move {
										accc.fetch_add(1, ATOMIC_ORDERING);
										let (r, w) = stream.split();
										let mut conn = Conn::new(addr, r, w, cc);
										conn.as_server(hc).await;
										accc.fetch_sub(1, ATOMIC_ORDERING);
									});
								}
							}
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
