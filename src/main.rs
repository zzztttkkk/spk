mod h2tp;

fn main() {
	let mut serv = h2tp::create_server(3);
	serv.listen("127.0.0.1:8080");
}
