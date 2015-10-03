use ::bitreader::BitReader;
use ::huffman;
use ::huffman::tree::Tree;

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
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
	huffman_codes: Option<HuffmanCodes>,
	output_buf: VecDeque<u8>,
}

type BFinal = bool;

#[derive(Debug, Clone, PartialEq)]
enum BType {
	NoCompression,
	CompressedWithFixedHuffmanCodes,
	CompressedWithDynamicHuffmanCodes,
}

type HuffmanCodes = huffman::tree::Tree;
type Symbol = u16;
type Length = u16;
type DistanceCode = u8;
type LengthDistanceCode = (Length, DistanceCode);
type Distance = u16;
type LengthDistance = (Length, Distance);

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
	HandlingHuffmanCodes(BType),
	HuffmanCodes(HuffmanCodes),
	Symbol(Symbol),
	Length(Length),
	LengthDistanceCode(LengthDistanceCode),
	LengthDistance(LengthDistance),
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
			huffman_codes: None,
			output_buf: VecDeque::with_capacity(32768),
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

	fn create_fixed_huffman_codes() -> result::Result<State, DecompressorError> {
		let lengths = [vec!(8; 144), vec!(9; 112), vec!(7; 24), vec!(8; 8)].concat();

		Ok(State::HuffmanCodes(huffman::codes_from_lengths(lengths)))
	}

	fn parse_next_symbol<R: Read>(ref mut in_stream: &mut BitReader<R>, huffman_codes: &HuffmanCodes) -> result::Result<State, DecompressorError> {
		let mut tree = huffman_codes.clone();

		loop {
			match in_stream.read_bit() {
				Ok(bit) =>
					match tree.lookup(bit) {
						Some(Tree::Leaf(symbol)) => return Ok(State::Symbol(symbol)),
						Some(inner) => tree = inner,
						None => unreachable!(),
					},
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}
	}

	fn parse_extra_bits_for_length_code<R: Read>(ref mut in_stream: &mut BitReader<R>, length_code: Symbol) -> result::Result<State, DecompressorError> {
		let (base_length, count_extra_bits) = match length_code {
			257...264 => (length_code - 254, 0),
			265...268 => ( 2 * (length_code - 265) +  11, 1),
			269...272 => ( 4 * (length_code - 269) +  19, 2),
			273...276 => ( 8 * (length_code - 273) +  35, 3),
			277...280 => (16 * (length_code - 277) +  67, 4),
			281...284 => (32 * (length_code - 281) + 131, 5),
			285       => (258, 0),
			_         => unreachable!(),
		};

		if count_extra_bits == 0 {
			Ok(State::Length(base_length))
		} else {
			match in_stream.read_u8_from_n_bits(count_extra_bits) {
				Ok(my_u8) => Ok(State::Length(base_length + (my_u8 as Length))),
				Err(_) => Err(DecompressorError::UnexpectedEOF),
			}
		}
	}

	fn parse_distance_code_for_length<R: Read>(ref mut in_stream: &mut BitReader<R>, length: Length) -> result::Result<State, DecompressorError> {
		match in_stream.read_u8_from_n_bits(5) {
			Ok(distance_code) => Ok(State::LengthDistanceCode((length, distance_code))),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_extra_bits_for_distance_code<R: Read>(ref mut in_stream: &mut BitReader<R>, length_distance_code: LengthDistanceCode) -> result::Result<State, DecompressorError> {
		let (length, base_distance, count_extra_bits) = match length_distance_code {
			(length, distance_code @ 0... 3) => (length,          distance_code as Distance +  1,            0),
			(length, distance_code @ 4... 5) => (length, (   2 * (distance_code as Distance -  4) +     5),  1),
			(length, distance_code @ 6... 7) => (length, (   4 * (distance_code as Distance -  6) +     9),  2),
			(length, distance_code @ 8... 9) => (length, (   8 * (distance_code as Distance -  8) +    17),  3),
			(length, distance_code @10...11) => (length, (  16 * (distance_code as Distance - 10) +    33),  4),
			(length, distance_code @12...13) => (length, (  32 * (distance_code as Distance - 12) +    65),  5),
			(length, distance_code @14...15) => (length, (  64 * (distance_code as Distance - 14) +   129),  6),
			(length, distance_code @16...17) => (length, ( 128 * (distance_code as Distance - 16) +   257),  7),
			(length, distance_code @18...19) => (length, ( 256 * (distance_code as Distance - 18) +   513),  8),
			(length, distance_code @20...21) => (length, ( 512 * (distance_code as Distance - 20) +  1025),  9),
			(length, distance_code @22...23) => (length, (1024 * (distance_code as Distance - 22) +  2049), 10),
			(length, distance_code @24...25) => (length, (2048 * (distance_code as Distance - 24) +  4097), 11),
			(length, distance_code @26...27) => (length, (4096 * (distance_code as Distance - 26) +  8193), 12),
			(length, distance_code @28...29) => (length, (8192 * (distance_code as Distance - 28) + 16385), 13),
			_         => unreachable!(),
		};

		if count_extra_bits == 0 {
			Ok(State::LengthDistance((length, base_distance)))
		} else {
			match in_stream.read_u16_from_n_bits(count_extra_bits) {
				Ok(my_u16) => Ok(State::LengthDistance((length, base_distance + (my_u16 as Distance)))),
				Err(_) => Err(DecompressorError::UnexpectedEOF),
			}
		}
	}

	pub fn decompress<R: Read>(&mut self, ref mut in_stream: &mut BitReader<R>) -> VecDeque<u8> {
		let mut buf = VecDeque::new();

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
					self.header.btype = Some(btype.clone());
					self.state = State::HandlingHuffmanCodes(btype);
				},
				State::HandlingHuffmanCodes(BType::NoCompression) => {
					unimplemented!();
				},
				State::HandlingHuffmanCodes(BType::CompressedWithFixedHuffmanCodes) => {
					self.state = match Self::create_fixed_huffman_codes() {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::HandlingHuffmanCodes(BType::CompressedWithDynamicHuffmanCodes) => {
					unimplemented!();
				},
				State::HuffmanCodes(huffman_codes) => {
					self.huffman_codes = Some(huffman_codes);
					self.state = match Self::parse_next_symbol(*in_stream, self.huffman_codes.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::Symbol(byte @ 0...255) => {
					// literal byte
					buf.push_front(byte as u8);
					self.output_buf.push_front(byte as u8);

					println!("{:?}", byte);

					self.state = match Self::parse_next_symbol(*in_stream, self.huffman_codes.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};

					return buf;
				},
				State::Symbol(256) => {
					// end of block
					println!("end-of-block");
					unimplemented!()
				},
				State::Symbol(length_code @ 257...285) => {
					// length code
					println!("length code {:?}", length_code);
					self.state = match Self::parse_extra_bits_for_length_code(*in_stream, length_code) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::Symbol(_) => {
					unreachable!();
				},
				State::Length(length) => {
					println!("Length {:?}", length);

					self.state = match Self::parse_distance_code_for_length(*in_stream, length) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::LengthDistanceCode((length, distance_code)) => {
					println!("<Length, Distance Code> = {:?}", (length, distance_code));
					self.state = match Self::parse_extra_bits_for_distance_code(*in_stream, (length, distance_code)) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::LengthDistance((length, distance)) => {
					println!("<Length, Distance> = {:?}", (length, distance));
					unimplemented!();
				},
			}
		}
	}
}
