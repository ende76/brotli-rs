extern crate compression;

fn main() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0xa1, 0x00, 0x00, 0x00, 0x00, 0x81, 0x15, 0x08, 0x04, 0x00,
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("X", decompressed);

	println!("{:?}", decompressed);
}

