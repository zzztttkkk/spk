use tokio::net::TcpStream;

pub trait AsyncReader: tokio::io::AsyncRead + Unpin + Send {}

impl<T> AsyncReader for T where T: tokio::io::AsyncRead + Unpin + Send {}

pub trait AsyncWriter: tokio::io::AsyncWrite + Unpin + Send {}

impl<T> AsyncWriter for T where T: tokio::io::AsyncWrite + Unpin + Send {}

pub type ServTlsStream = tokio_rustls::server::TlsStream<TcpStream>;
pub type CliTlsStream = tokio_rustls::client::TlsStream<TcpStream>;
