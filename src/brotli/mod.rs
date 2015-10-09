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
	output_window: Vec<u8>,
	state: State,
	meta_block: MetaBlock,
	count_output: usize,
}

type WBits = u8;
type CodeLengths = Vec<usize>;
type HuffmanCodes = huffman::tree::Tree;
type IsLast = bool;
type IsLastEmpty = bool;
type MNibbles = u8;
type MSkipBytes = u8;
type MSkipLen = u32;
type MLen = u32;
type IsUncompressed = bool;
type MLenLiterals = Vec<u8>;
type NBltypes = u8;
type NPostfix = u8;
type NDirect = u8;
type ContextMode = u8;
type ContextModes = Vec<ContextMode>;
type NTreesL = u8;
type NTreesD = u8;
type NSym = u8;
type Symbol = u16;
type Symbols = Vec<Symbol>;
type TreeSelect = bool;

#[derive(Debug, Clone, PartialEq)]
enum PrefixCodeKind {
	Simple,
	Complex,
}

#[derive(Debug, Clone, PartialEq)]
struct PrefixCodeSimple {
	n_sym: Option<NSym>,
	symbols: Option<Symbols>,
	tree_select: Option<TreeSelect>,
}

#[derive(Debug, Clone, PartialEq)]
enum PrefixCode {
	Simple(PrefixCodeSimple),
	Complex,
}

impl PrefixCode {
	fn new_simple() -> PrefixCode {
		PrefixCode::Simple(PrefixCodeSimple {
			n_sym: None,
			symbols: None,
			tree_select: None,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
struct Header {
	wbits: Option<WBits>,
	wbits_codes: Option<HuffmanCodes>,
	window_size: Option<usize>,
	bltype_codes: Option<HuffmanCodes>,
}

impl Header {
	fn new() -> Header {
		Header{
			wbits: None,
			wbits_codes: None,
			bltype_codes: None,
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
	m_len: Option<MLen>,
	is_uncompressed: Option<IsUncompressed>,
	n_bltypes_l: Option<NBltypes>,
	n_bltypes_i: Option<NBltypes>,
	n_bltypes_d: Option<NBltypes>,
	n_postfix: Option<NPostfix>,
	n_direct: Option<NDirect>,
	context_modes_literals: Option<ContextModes>,
	n_trees_l: Option<NTreesL>,
	n_trees_d: Option<NTreesD>,
	prefix_code_literals: Option<PrefixCode>,
}

impl MetaBlockHeader {
	fn new() -> MetaBlockHeader {
		MetaBlockHeader{
			is_last: None,
			is_last_empty: None,
			m_nibbles: None,
			m_skip_bytes: None,
			m_skip_len: None,
			m_len: None,
			is_uncompressed: None,
			n_bltypes_l: None,
			n_bltypes_i: None,
			n_bltypes_d: None,
			n_postfix: None,
			n_direct: None,
			context_modes_literals: None,
			n_trees_l: None,
			n_trees_d: None,
			prefix_code_literals: None,
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
	MLen(MLen),
	IsUncompressed(IsUncompressed),
	MLenLiterals(MLenLiterals),
	BltypeCodes(HuffmanCodes),
	NBltypesL(NBltypes),
	NBltypesI(NBltypes),
	NBltypesD(NBltypes),
	NPostfix(NPostfix),
	NDirect(NDirect),
	ContextModesLiterals(ContextModes),
	NTreesL(NTreesL),
	NTreesD(NTreesD),
	PrefixCodeLiteralsKind(PrefixCodeKind),
	NSymLiterals(NSym),
	SymbolsLiterals(Symbols),
	MetaBlockEnd,
	StreamEnd,
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	UnexpectedEOF,
	NonZeroFillBit,
	NonZeroReservedBit,
	NonZeroTrailerBit,
	NonZeroTrailerNibble,
	ExpectedEndOfStream,
	InvalidMSkipLen,
	UnexpectedPrefixCodeKind,
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
			&DecompressorError::NonZeroTrailerNibble => "Enocuntered non-zero nibble trailing",
			&DecompressorError::ExpectedEndOfStream => "Expected end-of-stream, but stream did not end",
			&DecompressorError::InvalidMSkipLen => "Most significant byte of MSKIPLEN was zero",
			&DecompressorError::UnexpectedPrefixCodeKind => "Encountered unexpected kind of prefix code",
		}
	}
}

impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: BitReader<R>) -> Decompressor<R> {
		Decompressor{
			in_stream: in_stream,
			header: Header::new(),
			buf: VecDeque::new(),
			output_window: Vec::new(),
			state: State::StreamBegin,
			meta_block: MetaBlock::new(),
			count_output: 0,
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

	fn parse_m_len(&mut self) -> result::Result<State, DecompressorError> {
		let m_nibbles = self.meta_block.header.m_nibbles.unwrap() as usize;
		let m_len = match self.in_stream.read_u32_from_n_nibbles(m_nibbles) {
			Ok(m_len) => m_len,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		if m_nibbles > 4 && (m_len >> ((m_nibbles - 1) * 4) == 0) {

			Err(DecompressorError::NonZeroTrailerNibble)
		} else {

			Ok(State::MLen(m_len + 1))
		}
	}

	fn parse_is_uncompressed(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsUncompressed(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_mlen_literals(&mut self) -> result::Result<State, DecompressorError> {
		let bytes = match self.in_stream.read_fixed_length_string(self.meta_block.header.m_len.unwrap() as usize) {
			Ok(bytes) => bytes,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		Ok(State::MLenLiterals(bytes))
	}

	fn create_block_type_codes() -> result::Result<State, DecompressorError> {
		let bit_patterns = vec![
			vec![false],
			vec![true, false, false, false],
			vec![true, true, false, false],
			vec![true, false, true, false],
			vec![true, true, true, false],
			vec![true, false, false, true],
			vec![true, true, false, true],
			vec![true, false, true, true],
			vec![true, true, true, true],
		];
		let symbols = vec![1, 2, 3, 5, 9, 17, 33, 65, 129];
		let mut codes = Tree::new();

		for i in 0..bit_patterns.len() {
			codes.insert(bit_patterns[i].clone(), symbols[i]);
		}

		Ok(State::BltypeCodes(codes))
	}

	fn parse_n_bltypes(&mut self) -> result::Result<NBltypes, DecompressorError> {
		let mut tree = self.header.bltype_codes.as_ref().unwrap().clone();

		loop {
			match self.in_stream.read_bit() {
				Ok(bit) =>
					match tree.lookup(bit) {
						Some(Tree::Leaf(symbol)) => {
							tree = Tree::Leaf(symbol);
							break;
						}
						Some(inner) => tree = inner,
						None => unreachable!(),
					},
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		let (value, extra_bits) = match tree {
			Tree::Leaf(symbol @ 1...2) => (symbol, 0),
			Tree::Leaf(symbol @     3) => (symbol, 1),
			Tree::Leaf(symbol @     5) => (symbol, 2),
			Tree::Leaf(symbol @     9) => (symbol, 3),
			Tree::Leaf(symbol @    17) => (symbol, 4),
			Tree::Leaf(symbol @    33) => (symbol, 5),
			Tree::Leaf(symbol @    65) => (symbol, 6),
			Tree::Leaf(symbol @   129) => (symbol, 7),
			_ => unreachable!(),
		};

		match self.in_stream.read_u8_from_n_bits(extra_bits) {
			Ok(extra) => Ok(value as NBltypes + extra),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_n_bltypes_l(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesL(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_bltypes_i(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesI(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_bltypes_d(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesD(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_postfix(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => Ok(State::NPostfix(my_u8)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_n_direct(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(4) {
			Ok(my_u8) => Ok(State::NDirect(my_u8 << self.meta_block.header.n_postfix.unwrap())),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_context_modes_literals(&mut self) -> result::Result<State, DecompressorError> {
		let mut context_modes = vec![0; self.meta_block.header.n_bltypes_l.unwrap() as usize];

		for i in 0..context_modes.len() {
			match self.in_stream.read_u8_from_n_bits(2) {
				Ok(my_u8) => context_modes[i] = my_u8 as ContextMode,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		Ok(State::ContextModesLiterals(context_modes))
	}

	fn parse_n_trees_l(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NTreesL(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_trees_d(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NTreesD(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_prefix_code_kind(&mut self) -> result::Result<PrefixCodeKind, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(1) => Ok(PrefixCodeKind::Simple),
			Ok(_) => Ok(PrefixCodeKind::Complex),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_prefix_code_literals_kind(&mut self) -> result::Result<State, DecompressorError> {
		match self.parse_prefix_code_kind() {
			Ok(kind) => Ok(State::PrefixCodeLiteralsKind(kind)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_sym_literals(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => Ok(State::NSymLiterals(my_u8 + 1)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_symbols_literals(&mut self) -> result::Result<State, DecompressorError> {
		let n_sym = match self.meta_block.header.prefix_code_literals.as_ref().unwrap() {
			&PrefixCode::Simple(ref code) => code.n_sym.unwrap(),
			_ => unreachable!(),
		};

		println!("NSYM = {:?}", n_sym);

		unimplemented!();
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
					self.output_window = vec![0; self.header.window_size.unwrap()];

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

					self.state = match self.parse_m_len() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::MSkipBytes(0) => {
					self.meta_block.header.m_skip_bytes = Some(0);

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = State::MetaBlockEnd;
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

					self.state = State::MetaBlockEnd;
				},
				State::MLen(m_len) => {
					self.meta_block.header.m_len = Some(m_len);

					println!("MLEN = {:?}", m_len);

					self.state = match (&self.meta_block.header.is_last.unwrap(), &self.header.bltype_codes) {
						(&false, _) => match self.parse_is_uncompressed() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						(&true, &None) => match Self::create_block_type_codes() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						(&true, &Some(_)) => match self.parse_n_bltypes_l() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
					};
				},
				State::IsUncompressed(true) => {
					self.meta_block.header.is_uncompressed = Some(true);

					println!("UNCOMPRESSED = true");

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = match self.parse_mlen_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::MLenLiterals(m_len_literals) => {
					for literal in m_len_literals {
						self.buf.push_front(literal);
						self.output_window[self.count_output % self.header.window_size.unwrap() as usize] = literal;
						self.count_output += 1;
					}

					self.state = State::MetaBlockEnd;
					return Ok(self.buf.len());
				},
				State::IsUncompressed(false) => {
					self.meta_block.header.is_uncompressed = Some(false);

					println!("UNCOMPRESSED = false");

					unimplemented!();
				},
				State::BltypeCodes(bltype_codes) => {
					self.header.bltype_codes = Some(bltype_codes);

					self.state = match self.parse_n_bltypes_l() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::NBltypesL(n_bltypes_l) => {
					self.meta_block.header.n_bltypes_l = Some(n_bltypes_l);

					println!("NBLTYPESL = {:?}", n_bltypes_l);

					self.state = if n_bltypes_l >= 2 {
						unimplemented!();
					} else {
						match self.parse_n_bltypes_i() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					}
				},
				State::NBltypesI(n_bltypes_i) => {
					self.meta_block.header.n_bltypes_i = Some(n_bltypes_i);

					println!("NBLTYPESI = {:?}", n_bltypes_i);

					self.state = if n_bltypes_i >= 2 {
						unimplemented!();
					} else {
						match self.parse_n_bltypes_d() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					}
				},
				State::NBltypesD(n_bltypes_d) => {
					self.meta_block.header.n_bltypes_d = Some(n_bltypes_d);

					println!("NBLTYPESD = {:?}", n_bltypes_d);

					self.state = if n_bltypes_d >= 2 {
						unimplemented!();
					} else {
						match self.parse_n_postfix() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::NPostfix(n_postfix) => {
					self.meta_block.header.n_postfix = Some(n_postfix);

					println!("NPOSTFIX = {:?}", n_postfix);

					self.state = match self.parse_n_direct() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::NDirect(n_direct) => {
					self.meta_block.header.n_direct = Some(n_direct);

					println!("NDIRECT = {:?}", n_direct);

					self.state = match self.parse_context_modes_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::ContextModesLiterals(context_modes) => {
					self.meta_block.header.context_modes_literals = Some(context_modes);

					println!("Context Modes Literals = {:?}", self.meta_block.header.context_modes_literals);

					self.state = match self.parse_n_trees_l() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::NTreesL(n_trees_l) => {
					self.meta_block.header.n_trees_l = Some(n_trees_l);

					println!("NTREESL = {:?}", n_trees_l);

					self.state = if n_trees_l >= 2 {
						unimplemented!();
					} else {
						match self.parse_n_trees_d() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::NTreesD(n_trees_d) => {
					self.meta_block.header.n_trees_d = Some(n_trees_d);

					println!("NTREESD = {:?}", n_trees_d);

					self.state = if n_trees_d >= 2 {
						unimplemented!();
					} else {
						match self.parse_prefix_code_literals_kind() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::PrefixCodeLiteralsKind(PrefixCodeKind::Simple) => {
					self.meta_block.header.prefix_code_literals = Some(PrefixCode::new_simple());

					println!("Prefix Code Literals = {:?}", self.meta_block.header.prefix_code_literals);

					self.state = match self.parse_n_sym_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::NSymLiterals(n_sym) => {
					match self.meta_block.header.prefix_code_literals.as_mut().unwrap() {
						&mut PrefixCode::Simple(ref mut code) => code.n_sym = Some(n_sym),
						_ => unreachable!(),
					};

					println!("Prefix Code Literals = {:?}", self.meta_block.header.prefix_code_literals);

					self.state = match self.parse_symbols_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::SymbolsLiterals(symbols) => {
					match self.meta_block.header.prefix_code_literals.as_mut().unwrap() {
						&mut PrefixCode::Simple(ref mut code) => code.symbols = Some(symbols),
						_ => unreachable!(),
					};

					println!("Prefix Code Literals = {:?}", self.meta_block.header.prefix_code_literals);

					self.state = match self.meta_block.header.prefix_code_literals.as_ref().unwrap() {
						&PrefixCode::Simple(PrefixCodeSimple{
								n_sym: Some(4),
								symbols: _,
								tree_select: _,

						}) => unimplemented!(),
						&PrefixCode::Simple(PrefixCodeSimple{
								n_sym: Some(_),
								symbols: _,
								tree_select: _,

						}) => unimplemented!(),
						_ => unreachable!(),
					}
				},
				State::PrefixCodeLiteralsKind(PrefixCodeKind::Complex) => {

					println!("Prefix Codes Kind Literals = {:?}", PrefixCodeKind::Complex);

					unimplemented!();
				},
				State::MetaBlockEnd => {
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
