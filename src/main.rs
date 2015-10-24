extern crate brotli;

fn main() {
	use std::io::{Read, Write, stdout};
	use brotli::Decompressor;

	let brotli_stream = std::fs::File::open("data/plrabn12.txt.compressed").unwrap();

	let mut decompressed = &mut Vec::new();
	let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);

	let mut expected = &mut Vec::new();
	let _ = std::fs::File::open("data/plrabn12.txt").unwrap().read_to_end(&mut expected);

	assert_eq!(expected, decompressed);

	stdout().write_all(decompressed).ok();
}

