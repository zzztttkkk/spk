use std::sync::atomic::Ordering;

pub const ATOMIC_ORDERING: Ordering = Ordering::Relaxed;
pub const MESSAGE_BUFFER_SIZE: usize = 1024;