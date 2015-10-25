extern crate brotli;

fn main() {
	use std::io::{self, Read};
	use brotli::Decompressor;

	let mut input = vec![];
	let _ = Decompressor::new(&b"\x1b\x3f\xff\xff\xdb\x4f\xe2\x99\x80\x12".to_vec() as &[u8]).read_to_end(&mut input);
}

