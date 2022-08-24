use super::{
	conn::Conn,
	types::{AsyncReader, AsyncWriter},
};

pub struct Context<'c, R: AsyncReader, W: AsyncWriter> {
	_hijacked: bool,
	conn: &'c mut Conn<R, W>,
}

impl<'c, R, W> Context<'c, R, W>
where
	R: AsyncReader,
	W: AsyncWriter,
{
	pub fn new(conn: &'c mut Conn<R, W>) -> Self {
		return Self {
			_hijacked: false,
			conn,
		};
	}

	pub fn clear(&mut self) {
		self._hijacked = false;
	}

	pub fn hijacked(&self) -> bool {
		return self._hijacked;
	}
}
