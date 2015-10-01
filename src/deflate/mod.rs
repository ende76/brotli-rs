use ::bitreader::BitReader;

use std::io::Read;

/// Wraps an input stream and provides methods for decompressing.
///
/// # Examples
///
/// extern crate compression;
///
/// use compression::deflate::Decompressor;
/// use std::fs::File;
///
/// let mut f = try!(File::open("compressed.txt.gz"));
/// let mut deflate = Decompressor::new(f);
pub struct Decompressor {
	header: Header,
	state: State,
}

type BFinal = bool;

#[derive(Debug, Clone, PartialEq)]
enum BType {
	NoCompression,
	CompressedWithFixedHuffmanCodes,
	CompressedWithDynamicHuffmanCodes,
}

#[derive(Debug, Clone, PartialEq)]
struct Header {
	bfinal: Option<BFinal>,
	btype: Option<BType>,
}

impl Header {
	fn new() -> Header {
		Header{
			bfinal: None,
			btype: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum State {
	HeaderBegin,
}

impl Decompressor {
	pub fn new() -> Decompressor {
		Decompressor{
			header: Header::new(),
			state: State::HeaderBegin,
		}
	}

	pub fn decompress<R: Read>(&mut self, ref mut in_stream: &mut BitReader<R>) {
		unimplemented!()
	}
}



	// fn parse_bfinal(ref mut buf: &mut VecDeque<u8>, mut next_bit: &mut u8) -> result::Result<DecompressorSuccess, DecompressorError> {
	// 	if buf.len() < 1 {

	// 		Err(DecompressorError::NeedMoreBytes)
	// 	} else {
	// 		let b = if *next_bit == 7 {
	// 			buf.pop_front().unwrap()
	// 		} else {
	// 			buf[0]
	// 		};
	// 		let bit_mask = 1u8 << *next_bit;

	// 		*next_bit = (*next_bit + 1) % 8;

	// 		Ok(DecompressorSuccess::BFinal(b & bit_mask > 0))
	// 	}
	// }

	// fn parse_btype(ref mut buf: &mut VecDeque<u8>, mut next_bit: &mut u8) -> result::Result<DecompressorSuccess, DecompressorError> {
	// 	if buf.len() < 1 || (buf.len() < 2 && *next_bit == 7) {

	// 		Err(DecompressorError::NeedMoreBytes)
	// 	} else {
	// 		let b0 = if *next_bit == 7 {
	// 			buf.pop_front().unwrap()
	// 		} else {
	// 			buf[0]
	// 		};
	// 		let bit_mask0 = 1u8 << *next_bit;
	// 		*next_bit += 1;

	// 		let b1 = if *next_bit == 7 {
	// 			buf.pop_front().unwrap()
	// 		} else {
	// 			buf[0]
	// 		};
	// 		let bit_mask1 = 1u8 << *next_bit;
	// 		*next_bit += 1;

	// 		match (b1 & bit_mask1, b0 & bit_mask0) {
	// 			(0, 0) => Ok(DecompressorSuccess::BType(BType::NoCompression)),
	// 			(0, _) => Ok(DecompressorSuccess::BType(BType::CompressedWithFixedHuffmanCodes)),
	// 			(_, 0) => Ok(DecompressorSuccess::BType(BType::CompressedWithDynamicHuffmanCodes)),
	// 			(_, _) => Err(DecompressorError::ReservedBType),
	// 		}
	// 	}
	// }

	// fn create_fixed_huffman_codes() -> HuffmanCodes {
	// 	let lengths = [vec!(8; 144), vec!(9; 112), vec!(7; 24), vec!(8; 8)].concat();

	// 	huffman::codes_from_lengths(lengths)
	// }

	// fn parse_next_symbol(&mut self) -> result::Result<DecompressorSuccess, DecompressorError> {
	// 	if self.in_buf.len() < 1 {

	// 		Err(DecompressorError::NeedMoreBytes)
	// 	} else {
	// 		if self.current_symbol == None {

	// 			self.current_symbol = Some(self.huffman_codes.as_ref().unwrap() as *const HuffmanCodes);
	// 		}

	// 		loop {

	// 		}
	// 	}
	// }
