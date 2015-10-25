extern crate brotli;

fn main() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let mut input = vec![];
	let _ = Decompressor::new(Cursor::new(vec![0x1b, 0x3f, 0xff, 0xff, 0xdb, 0x4f, 0xe2, 0x99, 0x80, 0x12])).read_to_end(&mut input);
}

