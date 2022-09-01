use std::fs::Metadata;
use std::io::{Error, ErrorKind};
use async_trait::async_trait;
use crate::h2tp::handler::Handler;
use crate::h2tp::{Request, Response};
use crate::h2tp::response::RespBody;
use proc_macro::Handler;

pub enum ReadResult {
	File(String),
	Dir(String),
}

#[async_trait]
pub trait Readable: Handler {
	async fn meta(&self, req: &Request) -> Result<(String, Metadata), std::io::Error>;

	async fn handle<'a, 'c, 'h: 'a>(&'h self, req: &'a mut Request<'c>, resp: &'a mut Response<'c>) {
		match self.meta(req).await {
			Ok((ref path, ref meta, )) => {
				if meta.is_file() {
					self.send_file(path, meta, req, resp).await;
					return;
				}
				if meta.is_dir() {
					self.render_dir(path, meta, req, resp).await;
					return;
				}
				resp.ioe(std::io::Error::new(ErrorKind::NotFound, ""));
			}
			Err(e) => {
				resp.ioe(e);
			}
		}
	}

	async fn render_dir<'a, 'm>(&self, path: &'m String, metadate: &'m Metadata, req: &'a Request, resp: &'a mut Response) {
		if cached(metadate, req, resp) {
			return;
		}
		println!("{path}")
	}

	async fn send_file<'a, 'm>(&self, path: &'m String, metadate: &'m Metadata, req: &'a Request, resp: &'a mut Response) {
		if req.headers().is_some() && cached(metadate, req, resp) {
			return;
		}

		let result = tokio::fs::File::open(path).await;
		match result {
			Ok(file) => {
				let range: Option<(String, usize, usize)>;
				match req.headers() {
					Some(headers) => {
						range = headers.range();
					}
					None => {
						range = None;
					}
				}
				match range {
					None => {
						resp.body = Some(RespBody::File(file));
					}
					Some(range) => {
						println!("{:?}", range);
					}
				}
			}
			Err(e) => {
				resp.ioe(e);
				return;
			}
		}
	}
}

pub fn cached<'a, 'm>(metadate: &'m Metadata, req: &'a Request, resp: &'a mut Response) -> bool {
	println!("{:?} {:?} {:p}", metadate, req, resp);
	return false;
}

#[derive(Handler)]
#[From = "crate::h2tp::fs::read::Readable"]
pub struct SimpleOsReader {
	root: String,
}

#[async_trait]
impl Readable for SimpleOsReader {
	async fn meta(&self, req: &Request) -> Result<(String, Metadata), Error> {
		todo!()
	}
}
