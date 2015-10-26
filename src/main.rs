extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
	let mut input = vec![];
	let _ = Decompressor::new(&b"\x15\x3f\x60\x00\x15\x3f\x60\x00\x27\xb0\xdb\xa8\x80\x25\x27\xb0\xdb\x40\x80\x12".to_vec() as &[u8]).read_to_end(&mut input);

	println!("{:?}", input);
}