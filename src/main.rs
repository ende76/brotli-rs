extern crate compression;

fn main() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x1b, 0x13, 0x00, 0x00, 0xa4, 0xb0, 0xb2, 0xea, 0x81, 0x47, 0x02, 0x8a,
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("XXXXXXXXXXYYYYYYYYYY", decompressed);

	println!("{:?}", decompressed);
}

