extern crate brotli;


#[test]
/// Brotli: Empty file
fn should_decompress_to_empty_string() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x06
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: Empty file #01
fn should_decompress_to_empty_string_01() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x81, 0x01
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
#[should_panic(expected="non-zero bit")]
fn should_reject_invalid_stream_with_trailing_non_zero_bits() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0xa1, 0x03,
	]);

	let mut decompressed = &mut String::new();
	let result = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
/// Brotli: Empty file #15
fn should_decompress_to_empty_string_15() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x1a,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: Empty file #16
fn should_decompress_to_empty_string_16() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x81, 0x16, 0x00, 0x58
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: Empty file
#[should_panic(expected="Expected end-of-stream, but stream did not end")]
fn should_reject_invalid_stream_with_trailing_bytes() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x1a, 0xff
	]);

	let mut decompressed = &mut String::new();
	let result = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}


#[test]
/// Brotli: Empty file #17
fn should_decompress_to_empty_string_17() {
	use std::fs::{ File };
	use std::io::{ Read };
	use brotli::Decompressor;

	let brotli_stream = File::open("data/empty.compressed.17").unwrap();

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("", decompressed);
}

#[test]
/// Brotli: X file
fn should_decompress_to_x() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x0b, 0x00, 0x80, 0x58, 0x03,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("X", decompressed);
}

#[test]
/// Brotli: X file #03
fn should_decompress_to_x_03() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0xa1, 0x00, 0x00, 0x00, 0x00, 0x81, 0x15, 0x08, 0x04, 0x00,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("X", decompressed);
}

#[test]
/// Brotli: 10x10y
fn should_decompress_to_10x10y() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x1b, 0x13, 0x00, 0x00, 0xa4, 0xb0, 0xb2, 0xea, 0x81, 0x47, 0x02, 0x8a,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("XXXXXXXXXXYYYYYYYYYY", decompressed);
}

#[test]
/// Brotli: ukkonooa
/// introduces complex prefix trees, multiple trees, non-zero context maps
fn should_decompress_ukkonooa() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x1b, 0x76,  0x00, 0x00,  0x14, 0x4a,  0xac, 0x9b,  0x7a, 0xbd,  0xe1, 0x97,  0x9d, 0x7f,  0x8e, 0xc2,
		0x82, 0x36,  0x0e, 0x9c,  0xe0, 0x90,  0x03, 0xf7,  0x8b, 0x9e,  0x38, 0xe6,  0xb6, 0x00,  0xab, 0xc3,
		0xca, 0xa0,  0xc2, 0xda,  0x66, 0x36,  0xdc, 0xcd,  0x80, 0x8d,  0x2e, 0x21,  0xd7, 0x6e,  0xe3, 0xea,
		0x4c, 0xb8,  0xf0, 0xd2,  0xb8, 0xc7,  0xc2, 0x70,  0x4d, 0x3a,  0xf0, 0x69,  0x7e, 0xa1,  0xb8, 0x45,
		0x73, 0xab,  0xc4, 0x57,  0x1e,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("ukko nooa, ukko nooa oli kunnon mies, kun han meni saunaan, pisti laukun naulaan, ukko nooa, ukko nooa oli kunnon mies.", decompressed);
}

#[test]
/// Brotli: monkey
/// introduces static dictionary reference, multiple trees for literals
fn should_decompress_monkey() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x1b, 0x4a, 0x03, 0x00, 0x8c, 0x94, 0x6e, 0xde, 0xb4, 0xd7, 0x96, 0xb1, 0x78, 0x86, 0xf2, 0x2d,
		0xe1, 0x1a, 0xbc, 0x0b, 0x1c, 0xba, 0xa9, 0xc7, 0xf7, 0xcc, 0x6e, 0xb2, 0x42, 0x34, 0x51, 0x44,
		0x8b, 0x4e, 0x13, 0x08, 0xa0, 0xcd, 0x6e, 0xe8, 0x2c, 0xa5, 0x53, 0xa1, 0x9c, 0x5d, 0x2c, 0x1d,
		0x23, 0x1a, 0xd2, 0x56, 0xbe, 0xdb, 0xeb, 0x26, 0xba, 0x03, 0x65, 0x7c, 0x96, 0x6a, 0xa2, 0x76,
		0xec, 0xef, 0x87, 0x47, 0x33, 0xd6, 0x27, 0x0e, 0x63, 0x95, 0xe2, 0x1d, 0x8d, 0x2c, 0xc5, 0xd1,
		0x28, 0x9f, 0x60, 0x94, 0x6f, 0x02, 0x8b, 0xdd, 0xaa, 0x64, 0x94, 0x2c, 0x1e, 0x3b, 0x65, 0x7c,
		0x07, 0x45, 0x5a, 0xb2, 0xe2, 0xfc, 0x49, 0x81, 0x2c, 0x9f, 0x40, 0xae, 0xef, 0x68, 0x81, 0xac,
		0x16, 0x7a, 0x0f, 0xf5, 0x3b, 0x6d, 0x1c, 0xb9, 0x1e, 0x2d, 0x5f, 0xd5, 0xc8, 0xaf, 0x5e, 0x85,
		0xaa, 0x05, 0xbe, 0x53, 0x75, 0xc2, 0xb0, 0x22, 0x8a, 0x15, 0xc6, 0xa3, 0xb1, 0xe6, 0x42, 0x14,
		0xf4, 0x84, 0x54, 0x53, 0x19, 0x5f, 0xbe, 0xc3, 0xf2, 0x1d, 0xd1, 0xb7, 0xe5, 0xdd, 0xb6, 0xd9,
		0x23, 0xc6, 0xf6, 0x9f, 0x9e, 0xf6, 0x4d, 0x65, 0x30, 0xfb, 0xc0, 0x71, 0x45, 0x04, 0xad, 0x03,
		0xb5, 0xbe, 0xc9, 0xcb, 0xfd, 0xe2, 0x50, 0x5a, 0x46, 0x74, 0x04, 0x0d, 0xff, 0x20, 0x04, 0x77,
		0xb2, 0x6d, 0x27, 0xbf, 0x47, 0xa9, 0x9d, 0x1b, 0x96, 0x2c, 0x62, 0x90, 0x23, 0x8b, 0xe0, 0xf8,
		0x1d, 0xcf, 0xaf, 0x1d, 0x3d, 0xee, 0x8a, 0xc8, 0x75, 0x23, 0x66, 0xdd, 0xde, 0xd6, 0x6d, 0xe3,
		0x2a, 0x82, 0x8a, 0x78, 0x8a, 0xdb, 0xe6, 0x20, 0x4c, 0xb7, 0x5c, 0x63, 0xba, 0x30, 0xe3, 0x3f,
		0xb6, 0xee, 0x8c, 0x22, 0xa2, 0x2a, 0xb0, 0x22, 0x0a, 0x99, 0xff, 0x3d, 0x62, 0x51, 0xee, 0x08,
		0xf6, 0x3d, 0x4a, 0xe4, 0xcc, 0xef, 0x22, 0x87, 0x11, 0xe2, 0x83, 0x28, 0xe4, 0xf5, 0x8f, 0x35,
		0x19, 0x63, 0x5b, 0xe1, 0x5a, 0x92, 0x73, 0xdd, 0xa1, 0x50, 0x9d, 0x38, 0x5c, 0xeb, 0xb5, 0x03,
		0x6a, 0x64, 0x90, 0x94, 0xc8, 0x8d, 0xfb, 0x2f, 0x8a, 0x86, 0x22, 0xcc, 0x1d, 0x87, 0xe0, 0x48,
		0x0a, 0x96, 0x77, 0x90, 0x39, 0xc6, 0x23, 0x23, 0x48, 0xfb, 0x11, 0x47, 0x56, 0xca, 0x20, 0xe3,
		0x42, 0x81, 0xf7, 0x77, 0x32, 0xc1, 0xa5, 0x5c, 0x40, 0x21, 0x65, 0x17, 0x40, 0x29, 0x17, 0x17,
		0x6c, 0x56, 0x32, 0x98, 0x38, 0x06, 0xdc, 0x99, 0x4d, 0x33, 0x29, 0xbb, 0x02, 0xdf, 0x4c, 0x26,
		0x93, 0x6c, 0x17, 0x82, 0x86, 0x20, 0xd7, 0x03, 0x79, 0x7d, 0x9a, 0x00, 0xd7, 0x87, 0x00, 0xe7,
		0x0b, 0x66, 0xe3, 0x4c, 0x66, 0x71, 0x67, 0x08, 0x32, 0xf9, 0x08, 0x3e, 0x81, 0x33, 0xcd, 0x17,
		0x72, 0x31, 0xf0, 0xb8, 0x94, 0x52, 0x4b, 0x90, 0x31, 0x8e, 0x68, 0xc1, 0xef, 0x90, 0xc9, 0xe5,
		0xf2, 0x61, 0x09, 0x72, 0x25, 0xad, 0xec, 0xc5, 0x62, 0xc0, 0x0b, 0x12, 0x05, 0xf7, 0x91, 0x75,
		0x0d, 0xee, 0x61, 0x2e, 0x2e, 0x19, 0x09, 0xc2, 0x03,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	assert_eq!("znxcvnmz,xvnm.,zxcnv.,xcn.z,vn.zvn.zxcvn.,zxcn.vn.v,znm.,vnzx.,vnzxc.vn.z,vnz.,nv.z,nvmzxc,nvzxcvcnm.,vczxvnzxcnvmxc.zmcnvzm.,nvmc,nzxmc,vn.mnnmzxc,vnxcnmv,znvzxcnmv,.xcnvm,zxcnzxv.zx,qweryweurqioweupropqwutioweupqrioweutiopweuriopweuriopqwurioputiopqwuriowuqerioupqweropuweropqwurweuqriopuropqwuriopuqwriopuqweopruioqweurqweuriouqweopruioupqiytioqtyiowtyqptypryoqweutioioqtweqruowqeytiowquiourowetyoqwupiotweuqiorweuqroipituqwiorqwtioweuriouytuioerytuioweryuitoweytuiweyuityeruirtyuqriqweuropqweiruioqweurioqwuerioqwyuituierwotueryuiotweyrtuiwertyioweryrueioqptyioruyiopqwtjkasdfhlafhlasdhfjklashjkfhasjklfhklasjdfhklasdhfjkalsdhfklasdhjkflahsjdkfhklasfhjkasdfhasfjkasdhfklsdhalghhaf;hdklasfhjklashjklfasdhfasdjklfhsdjklafsd;hkldadfjjklasdhfjasddfjklfhakjklasdjfkl;asdjfasfljasdfhjklasdfhjkaghjkashf;djfklasdjfkljasdklfjklasdjfkljasdfkljaklfj", decompressed);
}

#[test]
/// Brotli: zeros
/// introduces "Signed" context id computation
fn should_decompress_zeros() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let brotli_stream = Cursor::new(vec![
		0x5b, 0xff, 0xff, 0x03, 0x60, 0x02, 0x20, 0x1e, 0x0b, 0x28, 0xf7, 0x7e, 0x00,
	]);

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	let mut expected = &mut String::new();
	let _ = std::fs::File::open("data/zeros").unwrap().read_to_string(&mut expected);

	assert_eq!(expected, decompressed);
}

#[test]
/// Brotli: quickfox_repeated
/// introduces simple prefix code with NSYM = 4, which uses tree_select flag
fn should_decompress_quickfox_repeated() {
	use std::io::{ Read };
	use brotli::Decompressor;

	let brotli_stream = std::fs::File::open("data/quickfox_repeated.compressed").unwrap();

	let mut decompressed = &mut String::new();
	let _ = Decompressor::new(brotli_stream).read_to_string(&mut decompressed);

	let mut expected = &mut String::new();
	let _ = std::fs::File::open("data/quickfox_repeated").unwrap().read_to_string(&mut expected);

	assert_eq!(expected, decompressed);
}

#[test]
/// Brotli: asyoulik.txt
/// introduces block switch commands for literals and distances
fn should_decompress_asyoulik_txt() {
	use std::io::{ Read };
	use brotli::Decompressor;

	let brotli_stream = std::fs::File::open("data/asyoulik.txt.compressed").unwrap();

	let mut decompressed = &mut Vec::new();
	let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);

	let mut expected = &mut Vec::new();
	let _ = std::fs::File::open("data/asyoulik.txt").unwrap().read_to_end(&mut expected);

	assert_eq!(expected, decompressed);
}

#[test]
/// Brotli: alice29.txt
/// introduces NBLTYPESI >= 2
fn should_decompress_alice29_txt() {
	use std::io::{ Read };
	use brotli::Decompressor;

	let brotli_stream = std::fs::File::open("data/alice29.txt.compressed").unwrap();

	let mut decompressed = &mut Vec::new();
	let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);

	let mut expected = &mut Vec::new();
	let _ = std::fs::File::open("data/alice29.txt").unwrap().read_to_end(&mut expected);

	assert_eq!(expected, decompressed);
}

#[test]
/// Brotli: metablock_reset.txt
/// introduces a new metablock with a different max number of btype_l
fn should_decompress_metablock_reset() {
	use std::io::{ Read };
	use brotli::Decompressor;

	let brotli_stream = std::fs::File::open("data/metablock_reset.compressed").unwrap();

	let mut decompressed = &mut Vec::new();
	let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);

	let mut expected = &mut Vec::new();
	let _ = std::fs::File::open("data/metablock_reset").unwrap().read_to_end(&mut expected);

	assert_eq!(expected, decompressed);
}

#[test]
#[should_panic(expected = "Code length check sum")]
/// frewsxcv_00: fuzzer-test
/// exposes endless-loop vulnerability, if runlength code lengths are not bounded by alphabet size
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/2
fn should_reject_frewsxcv_00() {
	use std::io::{ Cursor, Read };
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(Cursor::new(vec![0x1b, 0x3f, 0xff, 0xff, 0xdb, 0x4f, 0xe2, 0x99, 0x80, 0x12])).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected = "unexpected EOF")]
/// frewsxcv: fuzzer-test
/// exposes uncaught panic in read() implementation
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/3
fn should_reject_frewsxcv_01() {
	use std::io::Read;
	use brotli::Decompressor;

    let mut input = vec![];
    let result = Decompressor::new(&b"\xb1".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected = "invalid symbol")]
/// frewsxcv: fuzzer-test
/// exposes panic in "unreachable" branch in insert-and-copy-length decoding
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/4
fn should_reject_frewsxcv_02() {
	use std::io::Read;
	use brotli::Decompressor;

    let mut input = vec![];
    let result = Decompressor::new(&b"\x1b\x30\x30\x30\x24\x30\xe2\xd9\x30\x30".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected = "invalid complex prefix code")]
/// frewsxcv: fuzzer-test
/// exposes index-out-of-bounds error created by an invalid stream that results in all-zero codelengths for a complex prefix code
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/5
fn should_reject_frewsxcv_03() {
	use std::io::Read;
	use brotli::Decompressor;
    let mut input = vec![];
    let result = Decompressor::new(&b"\x30\x30\x40\x00\x00\x00\x00\x00".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected = "Code length check sum")]
/// frewsxcv: fuzzer-test
/// edge case for block type value, which _looks_ like a u8 but is just slightly bigger
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/6
fn should_reject_frewsxcv_04() {
	use std::io::Read;
	use brotli::Decompressor;
	let mut input = vec![];
	let result = Decompressor::new(&b"\x1b\x3f\x00\xff\xff\xb0\xe2\x99\x80\x12".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected = "unexpected EOF")]
/// frewsxcv: fuzzer-test
/// exposes wrong bound checks on tree lookup array bounds
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/7
fn should_reject_frewsxcv_05() {
	use std::io::Read;
	use brotli::Decompressor;
	let mut input = vec![];
	let result = Decompressor::new(&b"\x11\x3f\x00\x00\x24\xb0\xe2\x99\x80\x12".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="Run length")]
/// frewsxcv: fuzzer-test
/// exposes shift overflow if too small a type has been chosen for runlength code
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/8
fn should_reject_frewsxcv_06() {
	use std::io::Read;
	use brotli::Decompressor;
	let mut input = vec![];
	let result = Decompressor::new(&b"\x15\x3f\x60\x00\x15\x3f\x60\x00\x27\xb0\xdb\xa8\x80\x25\x27\xb0\xdb\x40\x80\x12".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="Code length check sum")]
/// frewsxcv: fuzzer-test
/// exposes case where runlength checksum does not add up to 32
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/9
fn should_reject_frewsxcv_07() {
	use std::io::Read;
	use brotli::Decompressor;
	let mut input = vec![];
	let result = Decompressor::new(&b"\x12\x1b\x00\x1e\x11\x00\x05\x09\x21\x00\x05\x04\x43\x05\xf5\x21\x1e\x11\x00\x05\xf5\x21\x00\x05\x04\x43".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="unexpected EOF")]
/// frewsxcv: fuzzer-test
/// exposes uncaught byte value 0 in transformation code
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/10
fn should_reject_frewsxcv_08() {
	use std::io::Read;
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(&b"\x1b\x3f\x01\xf0\x24\xb0\xc2\xa4\x80\x54\xff\xd7\x24\xb0\x12".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="non-positive distance")]
/// frewsxcv: fuzzer-test
/// exposes uncaught non-positive distances
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/10
fn should_reject_frewsxcv_09() {
	use std::io::Read;
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(&b"\x5b\xff\x00\x01\x40\x0a\x00\xab\x16\x7b\xac\x14\x48\x4e\x73\xed\x01\x92\x03".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="invalid symbol")]
/// frewsxcv: fuzzer-test
/// exposes uncaught invalid block type in block switch command
/// found and reported by Corey Farwell – https://github.com/ende76/brotli-rs/issues/10
fn should_reject_frewsxcv_10() {
	use std::io::Read;
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(&b"\x51\xac\x00\x48\x2f\x73\x14\x01\x14\x00\x00\x01\x00\x14\x14\xff\x00\x02\x00\x00\x00\x00\x00\x64\x14\x24\x14\x14\x14\x14\x14\x80\x00\x00\x14\xff\xff\x00\x00\x14\x14\x14\x14\x14\x14\x80\x00\x80".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

#[test]
#[should_panic(expected="invalid symbol")]
/// afl: fuzzer-test
/// exposes uncaught invalid block type in block switch command
fn should_reject_afl_00() {
	use std::io::Read;
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(&b"\x01\xe6\x00\x76\x42\x10\x01\x1c\x24\x24\x3c\xd7\xd7\xd7\x01\x1c".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}

}

#[test]
#[should_panic(expected="invalid symbol")]
/// afl: fuzzer-test
/// exposes failure to reject simple prefix code with duplicate symbols
fn should_reject_afl_01() {
	use std::io::Read;
	use brotli::Decompressor;

	let mut input = vec![];
	let result = Decompressor::new(&b"\x9b\x01\x10\xed\xa3\xb0\x96\xd2\x81\x47\x00\x00\x01\x1e\x07\xa4\xce\xb2\xea\x81\x4b\x02\x8a".to_vec() as &[u8]).read_to_end(&mut input);

	match result {
		Err(e) => panic!("{:?}", e),
		_ => {},
	}
}

fn inverse_move_to_front_transform(v: &mut[u8]) {
	let mut mtf: Vec<u8> = vec![0; 256];
	let v_len = v.len();

	for i in 0..256 {
		mtf[i] = i as u8;
	}

	for i in 0..v_len {
		let index = v[i] as usize;
		let value = mtf[index];
		v[i] = value;

		for j in (1..index+1).rev() {
			mtf[j] = mtf[j - 1];
		}
		mtf[0] = value;
	}
}

fn move_to_front_transform(v: &mut[u8]) {
	let mut alphabet: Vec<u8> = vec![0; 256];
	let v_len = v.len();

	for i in 0..256 {
		alphabet[i] = i as u8;
	}

	for i in 0..v_len {
		let value = v[i];
		let mut index = 0;
		loop {
			if alphabet[index] == value {
				break;
			}
			index += 1
		}

		for j in (1..index+1).rev() {
			alphabet[j] = alphabet[j - 1];
		}
		alphabet[0] = value;
		v[i] = index as u8;
	}
}

#[test]
fn should_not_change() {
	let mut v: Vec<u8> = vec![0, 0, 0, 1];
	let expected = v.clone();

	inverse_move_to_front_transform(&mut v);

	assert_eq!(expected, v);
}


#[test]
fn should_compose_to_identity() {
	let mut v: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 3, 3, 3, 3, 3, 3, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
	let expected = v.clone();

	inverse_move_to_front_transform(&mut v);
	move_to_front_transform(&mut v);

	assert_eq!(expected, v);
}


