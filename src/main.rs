mod h2tp;

#[tokio::main]
async fn main() {
	let mut server = h2tp::create_server();
	server.listen("127.0.0.1:8080").await;
}
