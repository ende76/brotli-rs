extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
	let mut input = vec![];
	let _ = Decompressor::new(&b"\x12\x1b\x00\x1e\x11\x00\x05\x09\x21\x00\x05\x04\x43\x05\xf5\x21\x1e\x11\x00\x05\xf5\x21\x00\x05\x04\x43".to_vec() as &[u8]).read_to_end(&mut input);
}
