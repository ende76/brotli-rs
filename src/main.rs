extern crate compression;

fn main() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x5b, 0xff, 0xff, 0x03, 0x60, 0x02, 0x20, 0x1e, 0x0b, 0x28, 0xf7, 0x7e, 0x00,
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	let mut expected = &mut String::new();
	let _ = std::fs::File::open("data/zeros").unwrap().read_to_string(&mut expected);

	assert_eq!(expected, decompressed);

	// println!("{:?}", decompressed);
}

