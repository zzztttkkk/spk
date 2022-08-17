use crate::h2tp::utils::uricoding_excepts::ENCODE_URI_EXCEPTS;

use super::uricoding_excepts::{ENCODE_URI_COMPONENT_EXCEPTS, HEX_TO_INT_TABLE};

const UPPERHEX: &[u8] = "0123456789ABCDEF".as_bytes();

#[inline]
fn percent_encode(dest: &mut String, v: u8) {
	dest.push('%');
	dest.push(UPPERHEX[(v as usize) >> 4] as char);
	dest.push(UPPERHEX[(v as usize) & 15] as char);
}

pub fn encode_uri(dest: &mut String, src: &str) {
	let bytes = src.as_bytes();
	for i in 0..bytes.len() {
		let b = bytes[i];
		if ENCODE_URI_EXCEPTS[b as usize] {
			dest.push(b as char);
			continue;
		}
		percent_encode(dest, b);
	}
}

pub fn encode_uri_component(dest: &mut String, src: &str) {
	let bytes = src.as_bytes();
	for i in 0..bytes.len() {
		let b = bytes[i];
		if ENCODE_URI_COMPONENT_EXCEPTS[b as usize] {
			dest.push(b as char);
			continue;
		}
		percent_encode(dest, b);
	}
}

pub fn decode_uri(dest: &mut String, src: &str) -> bool {
	let bytes = src.as_bytes();

	let mut i = 0;
	loop {
		if i >= bytes.len() {
			break;
		}

		let c = bytes[i];
		if c == b'%' {
			if i + 3 >= bytes.len() {
				return false;
			}

			let x2 = HEX_TO_INT_TABLE[bytes[i + 2] as usize];
			let x1 = HEX_TO_INT_TABLE[bytes[i + 1] as usize];
			if x1 == 16 && x2 == 16 {
				dest.push('%');
				i += 3;
				continue;
			}
			dest.push((x1 << 4 | x2) as char);
			i += 3;
		} else {
			dest.push(c as char);
			i += 1;
		}
	}
	return true;
}

#[cfg(test)]
mod tests {
	use crate::h2tp::utils::uricoding::{decode_uri, encode_uri};

	#[test]
	fn test_encode_uri() {
		let mut dist = String::with_capacity(100);
		encode_uri(&mut dist, "ABC abc 123 xxx");
		println!("1 {}", dist);
		let mut x = String::with_capacity(100);
		decode_uri(&mut x, dist.as_str());
		println!("2 {}", x);
	}
}
