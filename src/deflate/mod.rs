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
	huffman_codes_literal_length: Option<HuffmanCodes>,
	huffman_codes_distance: Option<HuffmanCodes>,
	output_buf: Vec<u8>,
}

type BFinal = bool;

#[derive(Debug, Clone, PartialEq)]
enum BType {
	NoCompression,
	CompressedWithFixedHuffmanCodes,
	CompressedWithDynamicHuffmanCodes,
}

type HLit = u16;
type HDist = u8;
type HCLen = u8;
type CodeLengths = Vec<usize>;
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
	hlit: Option<HLit>,
	hdist: Option<HDist>,
	hclen: Option<HCLen>,
	code_lengths_for_code_length_alphabet: Option<CodeLengths>,
	huffman_codes_code_length: Option<HuffmanCodes>,
}

impl Header {
	fn new() -> Header {
		Header{
			bfinal: None,
			btype: None,
			hlit: None,
			hdist: None,
			hclen: None,
			code_lengths_for_code_length_alphabet: None,
			huffman_codes_code_length: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum State {
	HeaderBegin,
	BFinal(BFinal),
	BType(BType),
	HandlingHuffmanCodes(BType),
	HLit(HLit),
	HDist(HDist),
	HCLen(HCLen),
	CodeLengthsForCodeLengthAlphabet(CodeLengths),
	HuffmanCodesForCodeLengths(HuffmanCodes),
	HuffmanCodes((HuffmanCodes, HuffmanCodes)),
	Symbol(Symbol),
	Length(Length),
	LengthDistanceCode(LengthDistanceCode),
	LengthDistance(LengthDistance),
	EndOfBlock,
	Finished,
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
			huffman_codes_literal_length: None,
			huffman_codes_distance: None,
			output_buf: Vec::with_capacity(32768),
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
		let lengths_literal_length = [vec!(8; 144), vec!(9; 112), vec!(7; 24), vec!(8; 8)].concat();
		let lengths_distance = [vec!(5; 32)].concat();

		Ok(State::HuffmanCodes((huffman::codes_from_lengths(lengths_literal_length), huffman::codes_from_lengths(lengths_distance))))
	}

	fn create_huffman_codes_for_code_lengths(lengths_code_lengths: CodeLengths) ->  result::Result<State, DecompressorError> {

		Ok(State::HuffmanCodesForCodeLengths(huffman::codes_from_lengths(lengths_code_lengths)))
	}

	fn parse_hlit<R: Read>(ref mut in_stream: &mut BitReader<R>) -> result::Result<State, DecompressorError> {
		match in_stream.read_u16_from_n_bits(5) {
			Ok(hlit) => Ok(State::HLit(hlit + 257)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_hdist<R: Read>(ref mut in_stream: &mut BitReader<R>) -> result::Result<State, DecompressorError> {
		match in_stream.read_u8_from_n_bits(5) {
			Ok(hdist) => Ok(State::HDist(hdist + 1)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_hclen<R: Read>(ref mut in_stream: &mut BitReader<R>) -> result::Result<State, DecompressorError> {
		match in_stream.read_u8_from_n_bits(4) {
			Ok(hclen) => Ok(State::HCLen(hclen + 4)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_code_lengths_for_code_length_alphabet<R: Read>(ref mut in_stream: &mut BitReader<R>, hclen: u8) -> result::Result<State, DecompressorError> {
		let mut code_lengths = vec![0; 19];
		let alphabet_code_lengths = &[16usize, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

		for i in 0..hclen as usize {
			match in_stream.read_u8_from_n_bits(3) {
				Ok(code_length) => code_lengths[alphabet_code_lengths[i]] = code_length as usize,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		Ok(State::CodeLengthsForCodeLengthAlphabet(code_lengths))
	}

	fn parse_code_lengths_for_length_and_distance<R: Read>(ref mut in_stream: &mut BitReader<R>, huffman_codes: &HuffmanCodes, hlit: HLit, hdist: HDist) -> result::Result<State, DecompressorError> {
		let expected_count = hlit as usize + hdist as usize;
		let mut code_lengths = Vec::with_capacity(expected_count);
		let mut count_code_lengths = 0usize;

		println!("Expected Count = {:?}", expected_count);

		while count_code_lengths < expected_count {
			let mut tree = huffman_codes.clone();
			let symbol = {
				let length_symbol;

				loop {
					match in_stream.read_bit() {
						Ok(bit) =>
							match tree.lookup(bit) {
								Some(Tree::Leaf(symbol)) => {
									length_symbol = symbol;
									break;
								},
								Some(inner) => tree = inner,
								None => unreachable!(),
							},
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				}

				length_symbol
			};

			let length_code = symbol as usize;

			println!("Length Code = {:?}", length_code);

			match length_code {
				0...15 => {
					code_lengths.push(length_code);
					count_code_lengths += 1;
				},
				16 => {
					let last_code_length = code_lengths[count_code_lengths - 1];
					let repeat = match in_stream.read_u8_from_n_bits(2) {
						Ok(my_u8) => my_u8 + 3,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					println!("Repeat = {:?}", repeat);

					for _ in 0..repeat {
						code_lengths.push(last_code_length);
						count_code_lengths += 1;
					}
				},
				17 => {
					let repeat = match in_stream.read_u8_from_n_bits(3) {
						Ok(my_u8) => my_u8 + 3,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					println!("Repeat = {:?}", repeat);

					for _ in 0..repeat {
						code_lengths.push(0);
						count_code_lengths += 1;
					}
				},
				18 => {
					let repeat = match in_stream.read_u8_from_n_bits(7) {
						Ok(my_u8) => my_u8 + 11,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					println!("Repeat = {:?}", repeat);

					for _ in 0..repeat {
						code_lengths.push(0);
						count_code_lengths += 1;
					}
				},
				_ => break,
			}

		}
		println!("code_lengths = {:?}, count = {:?}", code_lengths, code_lengths.len());

		unimplemented!();
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

			println!("Count extra bits for length code = {:?}", count_extra_bits);

			match in_stream.read_u8_from_n_bits(count_extra_bits) {
				Ok(my_u8) => Ok(State::Length(base_length + (my_u8 as Length))),
				Err(_) => Err(DecompressorError::UnexpectedEOF),
			}
		}
	}

	fn parse_distance_code_for_length<R: Read>(ref mut in_stream: &mut BitReader<R>, length: Length, huffman_codes: &HuffmanCodes) -> result::Result<State, DecompressorError> {
		let mut tree = huffman_codes.clone();

		loop {
			match in_stream.read_bit() {
				Ok(bit) =>
					match tree.lookup(bit) {
						Some(Tree::Leaf(symbol)) => return Ok(State::LengthDistanceCode((length, symbol as u8))),
						Some(inner) => tree = inner,
						None => unreachable!(),
					},
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
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

			println!("Count extra bits for distance code = {:?}", count_extra_bits);

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

					println!("BFinal = {:?}", self.header.bfinal);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_btype(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::BType(btype) => {
					self.header.btype = Some(btype.clone());

					println!("BType = {:?}", self.header.btype);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = State::HandlingHuffmanCodes(btype);
				},
				State::HandlingHuffmanCodes(BType::NoCompression) => {
					unimplemented!();
				},
				State::HandlingHuffmanCodes(BType::CompressedWithFixedHuffmanCodes) => {
					self.state = match Self::create_fixed_huffman_codes() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::HandlingHuffmanCodes(BType::CompressedWithDynamicHuffmanCodes) => {
					self.state = match Self::parse_hlit(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::HLit(hlit) => {
					self.header.hlit = Some(hlit);

					println!("HLit = {:?}", hlit);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_hdist(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::HDist(hdist) => {
					self.header.hdist = Some(hdist);

					println!("HDist = {:?}", hdist);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_hclen(*in_stream) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::HCLen(hclen) => {
					self.header.hclen = Some(hclen);

					println!("HCLen = {:?}", hclen);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_code_lengths_for_code_length_alphabet(*in_stream, self.header.hclen.unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::CodeLengthsForCodeLengthAlphabet(code_lengths) => {
					self.header.code_lengths_for_code_length_alphabet = Some(code_lengths);

					println!("Code Lengths for Code Length Alphabet = {:?}", self.header.code_lengths_for_code_length_alphabet.as_ref().unwrap().clone());
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::create_huffman_codes_for_code_lengths(self.header.code_lengths_for_code_length_alphabet.as_ref().unwrap().clone()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::HuffmanCodesForCodeLengths(huffman_codes_code_length) => {
					self.header.huffman_codes_code_length = Some(huffman_codes_code_length);

					println!("Huffman Codes for Code Lengths = {:?}", self.header.huffman_codes_code_length);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_code_lengths_for_length_and_distance(
						*in_stream,
						self.header.huffman_codes_code_length.as_ref().unwrap(),
						self.header.hlit.unwrap(),
						self.header.hdist.unwrap(),
					) {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::HuffmanCodes((huffman_codes_literal_length, huffman_codes_distance)) => {
					self.huffman_codes_literal_length = Some(huffman_codes_literal_length);
					self.huffman_codes_distance = Some(huffman_codes_distance);
					self.state = match Self::parse_next_symbol(*in_stream, self.huffman_codes_literal_length.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::Symbol(byte @ 0...255) => {
					// literal byte
					buf.push_front(byte as u8);
					self.output_buf.push(byte as u8);

					println!("Literal byte = {:?}", byte);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_next_symbol(*in_stream, self.huffman_codes_literal_length.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};

					println!("{:?}", buf);

					return buf;
				},
				State::Symbol(256) => {
					// end of block
					println!("End-Of-Block");
					self.state = State::EndOfBlock;
				},
				State::Symbol(length_code @ 257...285) => {
					// length code
					println!("length code {:?}", length_code);
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);
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
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					self.state = match Self::parse_distance_code_for_length(*in_stream, length, self.huffman_codes_distance.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::LengthDistanceCode((length, distance_code)) => {
					println!("<Length, Distance Code> = {:?}", (length, distance_code));
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);
					self.state = match Self::parse_extra_bits_for_distance_code(*in_stream, (length, distance_code)) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::LengthDistance((length, distance)) => {
					println!("<Length, Distance> = {:?}", (length, distance));
					println!("@ bytes: {} and bits: {}", in_stream.global_bit_pos / 8, in_stream.global_bit_pos % 8);

					let l = self.output_buf.len();
					let from = l - distance as usize;
					let to = from + length as usize;
					let slice = &self.output_buf.clone()[from..if to > l { l } else { to }];
					let sl = slice.len();

					for i in 0..length as usize{
						self.output_buf.push(slice[(length as usize - i - 1) % sl]);
						buf.push_front(slice[(length as usize - i - 1) % sl]);
					}

					self.state = match Self::parse_next_symbol(*in_stream, self.huffman_codes_literal_length.as_ref().unwrap()) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};

					println!("{:?}", buf);

					return buf;
				},
				State::EndOfBlock => {
					self.state = if self.header.bfinal.unwrap() {
						State::Finished
					} else {
						State::HeaderBegin
					};
				},
				State::Finished => {
					return buf;
				}
			}
		}
	}
}
