use crate::h2tp::message::Message;

pub struct Response {
	msg: Message,
}

impl Response {
	pub fn new() -> Self {
		return Response {
			msg: Message::new(),
		};
	}
}