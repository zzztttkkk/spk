#[tokio::main]
async fn main() {
	match tokio::fs::metadata("./build.rs").await {
		Ok(meta) => {
			println!("{meta:?}");
		}
		Err(e) => {
			println!("{}", e);
		}
	}
}