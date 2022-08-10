extern crate core;

use std::process::Command;

fn main() {
	Command::new("python").arg("-m").arg("gen").spawn().expect("failed to run `python -m gen`");
}