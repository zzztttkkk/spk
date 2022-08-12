pub trait AsyncReader: tokio::io::AsyncRead + Unpin {}

impl<T> AsyncReader for T where T: tokio::io::AsyncRead + Unpin {}

pub trait AsyncWriter: tokio::io::AsyncWrite + Unpin {}

impl<T> AsyncWriter for T where T: tokio::io::AsyncWrite + Unpin {}