extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
	let mut input = vec![];
	let _ = Decompressor::new(&b"\x5b\xff\x00\x01\x40\x0a\x00\xab\x16\x7b\xac\x14\x48\x4e\x73\xed\x01\x92\x03".to_vec() as &[u8]).read_to_end(&mut input);
}