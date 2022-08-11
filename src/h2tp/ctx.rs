pub struct Context {
	_hijacked: bool,
}

impl Context {
	pub fn new() -> Self {
		return Self {
			_hijacked: false,
		};
	}

	pub fn clear(&mut self) {
		self._hijacked = false;
	}

	pub fn hijacked(&self) -> bool {
		return self._hijacked;
	}
}