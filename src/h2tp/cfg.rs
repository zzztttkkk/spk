use std::sync::atomic::Ordering;

pub const ATOMIC_ORDERING: Ordering = Ordering::Relaxed;
pub const MESSAGE_BUFFER_SIZE: usize = 4096;

struct Cfg {
	atomic_ordering: Ordering,
	message_buffer_size: usize,
}

impl Cfg {
	fn new() -> Self {
		return Self {
			atomic_ordering: Ordering::Relaxed,
			message_buffer_size: 4096,
		};
	}
}
