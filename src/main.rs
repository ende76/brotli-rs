extern crate compression;

fn main() {
	use std::io::{ Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(std::fs::File::open("data/asyoulik.txt.compressed").unwrap());

	let mut decompressed = &mut Vec::new();
	let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);

	let mut expected = &mut Vec::new();
	let _ = std::fs::File::open("data/asyoulik.txt").unwrap().read_to_end(&mut expected);

	assert_eq!(expected, decompressed);
}

