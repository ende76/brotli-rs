extern crate compression;

fn main() {
	use std::io::{ Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(std::fs::File::open("data/quickfox_repeated.compressed").unwrap());

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	let mut expected = &mut String::new();
	let _ = std::fs::File::open("data/quickfox_repeated").unwrap().read_to_string(&mut expected);

	assert_eq!(expected, decompressed);

	// println!("{:?}", decompressed);
}

