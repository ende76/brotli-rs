extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
	let mut input = vec![];
	let _ = Decompressor::new(&b"\x1b\x3f\x01\xf0\x24\xb0\xc2\xa4\x80\x54\xff\xd7\x24\xb0\x12".to_vec() as &[u8]).read_to_end(&mut input);

	println!("{:?}", String::from_utf8(input));
}