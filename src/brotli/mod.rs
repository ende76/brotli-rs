use ::bitreader::{ BitReader, BitReaderError };
use ::huffman;
use ::huffman::tree::Tree;

use std::collections::VecDeque;
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
/// use compression::brotli::Decompressor;
/// use std::fs::File;
///
/// let mut f = try!(File::open("compressed.txt.gz"));
/// let mut brotli = Decompressor::new(f);
pub struct Decompressor<R: Read> {
	in_stream: BitReader<R>,
	header: Header,
	buf: VecDeque<u8>,
	state: State,
	meta_block: MetaBlock
}

type WBits = u8;
type CodeLengths = Vec<usize>;
type HuffmanCodes = huffman::tree::Tree;
type IsLast = bool;
type IsLastEmpty = bool;
type MNibbles = u8;
type MSkipBytes = u8;
type MSkipLen = u32;

#[derive(Debug, Clone, PartialEq)]
struct Header {
	wbits: Option<WBits>,
	wbits_codes: Option<HuffmanCodes>,
	window_size: Option<usize>,
}

impl Header {
	fn new() -> Header {
		Header{
			wbits: None,
			wbits_codes: None,
			window_size: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
struct MetaBlock {
	header: MetaBlockHeader,
}

impl MetaBlock {
	fn new() -> MetaBlock {
		MetaBlock{
			header: MetaBlockHeader::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
struct MetaBlockHeader {
	is_last: Option<IsLast>,
	is_last_empty: Option<IsLastEmpty>,
	m_nibbles: Option<MNibbles>,
	m_skip_bytes: Option<MSkipBytes>,
	m_skip_len: Option<MSkipLen>,
}

impl MetaBlockHeader {
	fn new() -> MetaBlockHeader {
		MetaBlockHeader{
			is_last: None,
			is_last_empty: None,
			m_nibbles: None,
			m_skip_bytes: None,
			m_skip_len: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum State {
	StreamBegin,
	HeaderBegin,
	WBitsCodes(HuffmanCodes),
	WBits(WBits),
	HeaderEnd,
	HeaderMetaBlockBegin,
	IsLast(IsLast),
	IsLastEmpty(IsLastEmpty),
	MNibbles(MNibbles),
	MSkipBytes(MSkipBytes),
	MSkipLen(MSkipLen),
	StreamEnd,
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	UnexpectedEOF,
	NonZeroFillBit,
	NonZeroReservedBit,
	NonZeroTrailerBit,
	ExpectedEndOfStream,
	InvalidMSkipLen,
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
			&DecompressorError::NonZeroFillBit => "Enocuntered non-zero fill bit",
			&DecompressorError::NonZeroReservedBit => "Enocuntered non-zero reserved bit",
			&DecompressorError::NonZeroTrailerBit => "Enocuntered non-zero bit trailing the stream",
			&DecompressorError::ExpectedEndOfStream => "Expected end-of-stream, but stream did not end",
			&DecompressorError::InvalidMSkipLen => "Most significant byte of MSKIPLEN was zero",
		}
	}
}

impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: BitReader<R>) -> Decompressor<R> {
		Decompressor{
			in_stream: in_stream,
			header: Header::new(),
			buf: VecDeque::new(),
			state: State::StreamBegin,
			meta_block: MetaBlock::new(),
		}
	}

	fn create_wbits_codes() -> result::Result<State, DecompressorError> {
		let bit_patterns = vec![
			vec![true, false, false, false, false, true, false],
			vec![true, false, false, false, true, true, false],
			vec![true, false, false, false, false, false, true],
			vec![true, false, false, false, true, false, true],
			vec![true, false, false, false, false, true, true],
			vec![true, false, false, false, true, true, true],
			vec![false],
			vec![true, false, false, false, false, false, false],
			vec![true, true, false, false],
			vec![true, false, true, false],
			vec![true, true, true, false],
			vec![true, false, false, true],
			vec![true, true, false, true],
			vec![true, false, true, true],
			vec![true, true, true, true],
		];
		let symbols = vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24];
		let mut codes = Tree::new();

		for i in 0..bit_patterns.len() {
			codes.insert(bit_patterns[i].clone(), symbols[i]);
		}

		Ok(State::WBitsCodes(codes))
	}

	fn parse_wbits(&mut self) -> result::Result<State, DecompressorError> {
		let mut tree = self.header.wbits_codes.as_ref().unwrap().clone();

		loop {
			match self.in_stream.read_bit() {
				Ok(bit) =>
					match tree.lookup(bit) {
						Some(Tree::Leaf(symbol)) => return Ok(State::WBits(symbol as WBits)),
						Some(inner) => tree = inner,
						None => unreachable!(),
					},
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}
	}

	fn parse_is_last(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsLast(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_is_last_empty(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsLastEmpty(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_nibbles(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(3) => Ok(State::MNibbles(0)),
			Ok(my_u8) => Ok(State::MNibbles(my_u8 + 4)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_skip_bytes(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => Ok(State::MSkipBytes(my_u8)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_skip_len(&mut self) -> result::Result<State, DecompressorError> {
		let bytes = match self.in_stream.read_fixed_length_string(self.meta_block.header.m_skip_bytes.unwrap() as usize) {
			Ok(bytes) => bytes,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		let l = bytes.len();
		if l > 1 && bytes[l - 1] == 0 {
			return Err(DecompressorError::InvalidMSkipLen);
		}

		Ok(State::MSkipLen({
			let mut m_skip_len: MSkipLen = 0;
			let mut i = 0;
			for byte in bytes {
				m_skip_len = m_skip_len | ((byte as MSkipLen) << i);
				i += 1;
			}
			m_skip_len + 1
		}))
	}

	fn decompress(&mut self) -> result::Result<usize, DecompressorError> {
		loop {
			match self.state.clone() {
				State::StreamBegin => {

					self.state = State::HeaderBegin;
				},
				State::HeaderBegin => {
					self.state = match Self::create_wbits_codes() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::WBitsCodes(wbits_codes) => {
					self.header.wbits_codes = Some(wbits_codes);

					self.state = match self.parse_wbits() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::WBits(wbits) => {
					self.header.wbits = Some(wbits);
					self.header.window_size = Some((1 << wbits) - 16);

					println!("(WBITS, Window Size) = {:?}", (wbits, self.header.window_size));

					self.state = State::HeaderEnd;
				},
				State::HeaderEnd => {
					self.state = State::HeaderMetaBlockBegin;
				},
				State::HeaderMetaBlockBegin => {
					self.meta_block.header = MetaBlockHeader::new();

					self.state = match self.parse_is_last() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::IsLast(true) => {
					self.meta_block.header.is_last = Some(true);

					self.state = match self.parse_is_last_empty() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::IsLast(false) => {
					self.meta_block.header.is_last = Some(false);

					self.state = match self.parse_m_nibbles() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::IsLastEmpty(true) => {
					self.meta_block.header.is_last_empty = Some(true);

					self.state = State::StreamEnd;
				},
				State::IsLastEmpty(false) => {
					self.meta_block.header.is_last_empty = Some(false);

					self.state = match self.parse_m_nibbles() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::MNibbles(0) => {
					match self.in_stream.read_bit() {
						Ok(true) => return Err(DecompressorError::NonZeroReservedBit),
						Ok(false) => {},
						Err(_) => return Err(DecompressorError::UnexpectedEOF)
					}

					self.meta_block.header.m_nibbles = Some(0);

					self.state = match self.parse_m_skip_bytes() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::MNibbles(m_nibbles) => {
					self.meta_block.header.m_nibbles = Some(m_nibbles);
					unimplemented!();
				},
				State::MSkipBytes(0) => {
					self.meta_block.header.m_skip_bytes = Some(0);

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = if self.meta_block.header.is_last.unwrap() {
						State::StreamEnd
					} else {
						State::HeaderMetaBlockBegin
					};
				},
				State::MSkipBytes(m_skip_bytes) => {
					self.meta_block.header.m_skip_bytes = Some(m_skip_bytes);

					println!("MSKIPBYTES = {:?}", m_skip_bytes);

					self.state = match self.parse_m_skip_len() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::MSkipLen(m_skip_len) => {
					self.meta_block.header.m_skip_len = Some(m_skip_len);

					println!("MSKIPLEN = {:?}", m_skip_len);

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					match self.in_stream.read_fixed_length_string(m_skip_len as usize) {
						Ok(_) => {},
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = if self.meta_block.header.is_last.unwrap() {
						State::StreamEnd
					} else {
						State::HeaderMetaBlockBegin
					};
				},
				State::StreamEnd => {
					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroTrailerBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}

					match self.in_stream.read_u8() {
						Err(BitReaderError::EOF) => return Ok(self.buf.len()),
						Ok(_) => return Err(DecompressorError::ExpectedEndOfStream),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				}
			};
		}
	}
}

impl<R: Read> Read for Decompressor<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.buf.len() == 0 {
			match self.decompress() {
				Err(e) => {

					panic!(format!("{:?}", e.description()));
				},
				Ok(_) => {},
			}
		}

		let l = if self.buf.len() < buf.len() {

			self.buf.len()
		} else {

			buf.len()
		};

		for i in 0..l {

			buf[i] = self.buf.pop_back().unwrap();
		}

		Ok(l)
	}
}
