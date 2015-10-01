use ::bitreader::BitReader;

use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;
use std::result;

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
	BFinal(BFinal),
	BType(BType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecompressorError {
	UnexpectedEOF,
	BlockTypeReserved
}

impl Display for DecompressorError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for DecompressorError {
	fn description(&self) -> &str {
		match self {
			&DecompressorError::UnexpectedEOF => "Encountered unexpected EOF",
			&DecompressorError::BlockTypeReserved => "Reserved block type in deflate header",
		}
	}
}

impl Decompressor {
	pub fn new() -> Decompressor {
		Decompressor{
			header: Header::new(),
			state: State::HeaderBegin,
		}
	}

	fn parse_bfinal<R: Read>(ref mut in_stream: &mut BitReader<R>) -> result::Result<State, DecompressorError> {
		match in_stream.read_bit() {
			Ok(bfinal) => Ok(State::BFinal(bfinal)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_btype<R: Read>(ref mut in_stream: &mut BitReader<R>) -> result::Result<State, DecompressorError> {
		match in_stream.read_n_bits(2) {
			Ok(btype) => match (btype[1], btype[0]) {
				(false, false) => Ok(State::BType(BType::NoCompression)),
				(false, true) => Ok(State::BType(BType::CompressedWithFixedHuffmanCodes)),
				(true, false) => Ok(State::BType(BType::CompressedWithDynamicHuffmanCodes)),
				(true, true) => Err(DecompressorError::BlockTypeReserved),
			},
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	pub fn decompress<R: Read>(&mut self, ref mut in_stream: &mut BitReader<R>) -> io::Result<Vec<u8>>{
		loop {
			match self.state.clone() {
				State::HeaderBegin => {
					self.state = match Self::parse_bfinal(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::BFinal(bfinal) => {
					self.header.bfinal = Some(bfinal);
					self.state = match Self::parse_btype(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::BType(btype) => {
					self.header.btype = Some(btype);

					println!("{:?}", self.header);
					self.state = unimplemented!()
				}
			}
		}
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
