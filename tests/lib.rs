extern crate compression;

#[test]
/// Simple first test
fn should_decompress_xxxxxyyyyy() {
	use std::io::{ Cursor, Read };
	use compression::gzip::Decompressor;
	use compression::bitreader::BitReader;

	let gzip_stream = BitReader::new(Cursor::new(vec![0x1fu8, 0x8bu8, 0x08u8, 0x08u8, 0x9fu8, 0x30u8, 0x04u8, 0x56u8, 0x00u8, 0x03u8, 0x78u8, 0x78u8, 0x78u8, 0x78u8, 0x78u8, 0x79u8, 0x79u8, 0x79u8, 0x79u8, 0x79u8, 0x2eu8, 0x74u8, 0x78u8, 0x74u8, 0x00u8, 0xabu8, 0xa8u8, 0x00u8, 0x82u8, 0x4au8, 0x10u8, 0x00u8, 0x00u8, 0x42u8, 0x62u8, 0xddu8, 0x64u8, 0x0au8, 0x00u8, 0x00u8, 0x00u8]));
	let mut decompressed = &mut String::new();

	let _ = Decompressor::new(gzip_stream).read_to_string(&mut decompressed);

	assert_eq!("xxxxxyyyyy", decompressed);
}

#[test]
/// Tests distance codes being read correctly, and output bytes being appened in the right order to output buffer
fn should_decompress_abc() {
	use std::io::{ Cursor, Read };
	use compression::gzip::Decompressor;
	use compression::bitreader::BitReader;

	let gzip_stream = BitReader::new(Cursor::new(vec![0x1fu8, 0x8bu8, 0x08u8, 0x08u8, 0x31u8, 0x51u8, 0x10u8, 0x56u8, 0x00u8, 0x03u8, 0x61u8, 0x62u8, 0x63u8, 0x2eu8, 0x74u8, 0x78u8, 0x74u8, 0x00u8, 0x4bu8, 0x4cu8, 0x4au8, 0x4eu8, 0x49u8, 0x4du8, 0x4bu8, 0xcfu8, 0xc8u8, 0xccu8, 0xcau8, 0xceu8, 0xc9u8, 0xcdu8, 0xcbu8, 0x2fu8, 0x28u8, 0x2cu8, 0x2au8, 0x2eu8, 0x29u8, 0x2du8, 0x2bu8, 0xafu8, 0xa8u8, 0xacu8, 0x4au8, 0xa4u8, 0xaau8, 0x0cu8, 0x00u8, 0x20u8, 0x2du8, 0x2au8, 0x6au8, 0x68u8, 0x00u8, 0x00u8, 0x00u8]));
	let mut decompressed = &mut String::new();

	let _ = Decompressor::new(gzip_stream).read_to_string(&mut decompressed);

	assert_eq!("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz", decompressed);
}

#[test]
/// Tests file with dynamic huffman codes
fn should_decompress_abccba() {
	use std::io::{ Cursor, Read };
	use compression::gzip::Decompressor;
	use compression::bitreader::BitReader;

	let gzip_stream = BitReader::new(Cursor::new(vec![
		0x1f, 0x8b, 0x08, 0x08, 0xbf, 0x72, 0x10, 0x56, 0x00, 0x03, 0x61, 0x62, 0x63, 0x63, 0x62, 0x61,
		0x2e, 0x74, 0x78, 0x74, 0x00, 0x9d, 0xcb, 0xb7, 0x01, 0x00, 0x20, 0x08, 0x00, 0xb0, 0x5b, 0xb1,
		0x63, 0x45, 0xec, 0x5e, 0xef, 0x0f, 0x66, 0x0f, 0x08, 0xa9, 0xb4, 0xb1, 0x0e, 0x7d, 0x88, 0x29,
		0x17, 0xaa, 0xdc, 0xfa, 0x98, 0x6b, 0x9f, 0x7b, 0xcf, 0x5e, 0x73, 0xf4, 0xc6, 0x95, 0x4a, 0x4e,
		0x31, 0x78, 0x74, 0xd6, 0x68, 0x25, 0x05, 0xc0, 0xc7, 0x79, 0x72, 0xf1, 0x8d, 0xd6, 0x68, 0x00,
		0x00, 0x00
	]));
	let mut decompressed = &mut String::new();

	let _ = Decompressor::new(gzip_stream).read_to_string(&mut decompressed);

	assert_eq!("abcdefghijklmnopqrstuvwxyzzyxwvutsrqponmlkjihgfedcbaabcdefghijklmnopqrstuvwxyzzyxwvutsrqponmlkjihgfedcba", decompressed);
}

#[test]
/// Brotli: Empty file
fn should_decompress_to_empty_string() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x06
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: Empty file
fn should_decompress_to_empty_string_01() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x81, 0x01
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
#[should_panic]
/// Brotli: Empty file
fn should_reject_invalid_stream_with_trailing_non_zero_bits() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0xa1, 0x03,
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);
}

#[test]
/// Brotli: Empty file
fn should_decompress_to_empty_string_15() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x1a,
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: Empty file
#[should_panic(expected="Expected end-of-stream, but stream did not end")]
fn should_reject_invalid_stream_with_trailing_bytes() {
	use std::io::{ Cursor, Read };
	use compression::brotli::Decompressor;
	use compression::bitreader::BitReader;

	let brotli_stream = BitReader::new(Cursor::new(vec![
		0x1a, 0xff
	]));

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}
