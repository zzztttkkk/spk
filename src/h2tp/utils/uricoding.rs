use super::uricoding_excepts::{
	ENCODE_URI_COMPONENT_EXCEPTS, ENCODE_URI_EXCEPTS, HEX_TO_INT_TABLE,
};

const UPPERHEX: &[u8] = "0123456789ABCDEF".as_bytes();

#[inline]
fn percent_encode(dest: &mut Vec<u8>, v: u8) {
	dest.push(b'%');
	dest.push(UPPERHEX[(v as usize) >> 4]);
	dest.push(UPPERHEX[(v as usize) & 15]);
}

fn _encode_uri_by_tabel(dest: &mut Vec<u8>, src: &str, table: &[bool; 256]) {
	let bytes = src.as_bytes();
	for i in 0..bytes.len() {
		let b = bytes[i];
		if table[b as usize] {
			dest.push(b);
			continue;
		}
		percent_encode(dest, b);
	}
}

#[inline]
pub fn encode_uri(dest: &mut Vec<u8>, src: &str) {
	_encode_uri_by_tabel(dest, src, &ENCODE_URI_EXCEPTS)
}

#[inline]
pub fn encode_uri_component(dest: &mut Vec<u8>, src: &str) {
	_encode_uri_by_tabel(dest, src, &ENCODE_URI_COMPONENT_EXCEPTS)
}

pub fn encode_formed(dest: &mut Vec<u8>, src: &str) {
	let bytes = src.as_bytes();
	for i in 0..bytes.len() {
		let b = bytes[i];
		if b == b' ' {
			dest.push(b'+');
			continue;
		}
		if ENCODE_URI_COMPONENT_EXCEPTS[b as usize] {
			dest.push(b);
			continue;
		}
		percent_encode(dest, b);
	}
}

macro_rules! write_percents {
    ($idx:ident, $bytes:ident, $dest:ident) => {
		if $idx + 2 >= $bytes.len() {
			return false;
		}

		let x2 = HEX_TO_INT_TABLE[$bytes[$idx + 2] as usize];
		let x1 = HEX_TO_INT_TABLE[$bytes[$idx + 1] as usize];
		if x1 == 16 && x2 == 16 {
			$dest.push(b'%');
			$idx += 3;
			continue;
		}
		$dest.push(x1 << 4 | x2);
		$idx += 3;
	};
}

pub fn decode_uri(dest: &mut Vec<u8>, src: &str) -> bool {
	let bytes = src.as_bytes();

	let mut i = 0;
	loop {
		if i >= bytes.len() {
			return true;
		}

		let c = bytes[i];
		if c == b'%' {
			write_percents!(i, bytes, dest);
		} else {
			dest.push(c);
			i += 1;
		}
	}
}

pub fn decode_formed(dest: &mut Vec<u8>, src: &str) -> bool {
	let bytes = src.as_bytes();

	let mut i = 0;
	loop {
		if i >= bytes.len() {
			return true;
		}

		let c = bytes[i];
		match bytes[i] {
			c if c == b'%' => {
				write_percents!(i, bytes, dest);
			}
			c if c == b'+' => {
				dest.push(b' ');
				i += 1;
			}
			_ => {
				dest.push(c);
				i += 1;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::h2tp::utils::uricoding::{decode_uri, encode_uri};

	use super::decode_formed;

	macro_rules! vec2str {
		($name:ident) => {
			std::str::from_utf8($name.as_slice()).unwrap()
		};
	}

	#[test]
	fn test_encode_uri() {
		let mut dest = Vec::with_capacity(100);
		encode_uri(&mut dest, "ABC abc 123 xxx 我😊=?xxx");
		println!("1 {}", vec2str!(dest));
		let mut raw_dest = Vec::with_capacity(100);
		decode_uri(&mut raw_dest, vec2str!(dest));
		println!("2 {}", vec2str!(raw_dest));
	}

	#[test]
	fn test_formed() {
		let mut dest = Vec::with_capacity(100);
		decode_formed(&mut dest, "dd+ddd");
		println!("{}", vec2str!(dest));
	}
}
