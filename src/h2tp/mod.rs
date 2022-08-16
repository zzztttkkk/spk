use std::sync::{Arc};
use tokio::sync::Mutex;

mod server;
mod conn;
mod cfg;
mod message;
mod utils;
mod headers;
mod handler;
mod error;
mod status_code;
mod methods;
mod request;
mod response;
mod url;
mod ctx;
mod types;

pub async fn shutdown(handler: &Arc<Mutex<server::ShutdownHandler>>, timout: u64) {
	let mut guard = handler.lock().await;
	if (*guard).shutdown(timout).await {
		println!("Graceful Shutdown OK");
	} else {
		println!("Graceful Shutdown Failed");
	}
}

pub fn server() -> server::Server {
	return server::Server::new();
}

pub use handler::FuncHandler;

#[macro_export]
macro_rules! func {
    ($content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, _|{
				return Box::pin($content);
			})
		)
	};
 	(_, _, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, _|{
				return Box::pin($content);
			})
		)
	};
	(mut $req:ident, _, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|__req_arc_mutex, _|{
				return Box::pin(async move {
					let mut __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let mut $req = &mut (*__req_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	(mut $req:ident, $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|__req_arc_mutex, _|{
				return Box::pin(async move {
					let mut __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let mut $req = &mut (*__req_arc_mutex_guard);
					let __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let $resp = &(*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	($req:ident, _, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|__req_arc_mutex, _|{
				return Box::pin(async move {
					let __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let $req = &(*__req_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	(_, mut $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, __resp_arc_mutex|{
				return Box::pin(async move {
					let mut __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let mut $resp = &mut (*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	($req:ident, mut $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, __resp_arc_mutex|{
				return Box::pin(async move {
					let __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let $req = &(*__req_arc_mutex_guard);
					let mut __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let mut $resp = &mut (*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	(_, $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|_, __resp_arc_mutex|{
				return Box::pin(async move {
					let __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let $resp = &(*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	($req:ident, $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|__req_arc_mutex, __resp_arc_mutex|{
				return Box::pin(async move {
					let __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let $req = &(*__req_arc_mutex_guard);
					let __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let $resp = &(*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
	(mut $req:ident, mut $resp:ident, $content:expr) => {
		Box::new(
			h2tp::FuncHandler::new(|__req_arc_mutex, __resp_arc_mutex|{
				return Box::pin(async move {
					let mut __req_arc_mutex_guard = __req_arc_mutex.lock().await;
					let $req = &mut (*__req_arc_mutex_guard);
					let mut __resp_arc_mutex_guard = __resp_arc_mutex.lock().await;
					let $resp = &mut (*__resp_arc_mutex_guard);
					{
						$content
					}
				});
			})
		)
	};
}