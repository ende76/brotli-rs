use ::bitreader::{ BitReader, BitReaderError };
use ::huffman;
use ::huffman::tree::Tree;

use std::collections::VecDeque;
use std::cmp;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;
use std::result;

const LUT_0: [usize; 256] = [
	 0,  0,  0,  0,  0,  0,  0,  0,  0,  4,  4,  0,  0,  4,  0,  0,
	 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
	 8, 12, 16, 12, 12, 20, 12, 16, 24, 28, 12, 12, 32, 12, 36, 12,
	44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 32, 32, 24, 40, 28, 12,
	12, 48, 52, 52, 52, 48, 52, 52, 52, 48, 52, 52, 52, 52, 52, 48,
	52, 52, 52, 52, 52, 48, 52, 52, 52, 52, 52, 24, 12, 28, 12, 12,
	12, 56, 60, 60, 60, 56, 60, 60, 60, 56, 60, 60, 60, 60, 60, 56,
	60, 60, 60, 60, 60, 56, 60, 60, 60, 60, 60, 24, 12, 28, 12,  0,
	 0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
	 0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
	 0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
	 0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
	 2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
	 2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
	 2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
	 2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3
];

const LUT_1: [usize; 256] =[
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1,
 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1,
 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
];

const LUT_2: [usize; 256] = [
   0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
   2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
   2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
   2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
   3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
   3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
   3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
   3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
   4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
   4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
   4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
   4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
   5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
   5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
   5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
   6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7
];

#[derive(Debug, Clone, PartialEq)]
struct RingBuffer<T> {
	buf: Vec<T>,
	pos: usize,
}

impl<T> RingBuffer<T> {
	fn from_vec(v: Vec<T>) -> RingBuffer<T> {
		RingBuffer{
			buf: v,
			pos: 0,
		}
	}

	fn nth(&self, n: usize) -> Result<&T, RingBufferError> {
		if n >= self.buf.len() {
			Err(RingBufferError::ParameterExceededSize)
		} else {
			Ok(&self.buf[(self.pos + n) % self.buf.len()])
		}
	}

	fn push(&mut self, item: T) {
		self.pos = (self.pos + self.buf.len() - 1) % self.buf.len();
		self.buf[self.pos] = item;
	}
}


#[derive(Debug, Clone, PartialEq)]
enum RingBufferError {
	ParameterExceededSize,
}

impl Display for RingBufferError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {

		fmt.write_str(self.description())
	}
}

impl Error for RingBufferError {
	fn description(&self) -> &str {
		match self {
			&RingBufferError::ParameterExceededSize => "Index parameter exceeded ring buffer size",
		}
	}
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
type Literal = u8;
type Literals = Vec<Literal>;
type MLenLiterals = Literals;
type InsertLiterals = Literals;
type NBltypes = u8;
type BLen = u32;
type NPostfix = u8;
type NDirect = u8;
type ContextMode = u16;
type ContextModes = Vec<ContextMode>;
type ContextMap = Vec<u8>;
type NTrees = u8;
type NSym = u8;
type Symbol = u16;
type Symbols = Vec<Symbol>;
type TreeSelect = bool;
type InsertAndCopyLength = Symbol;
type InsertLength = u32;
type CopyLength = u32;
type InsertLengthAndCopyLength = (InsertLength, CopyLength);
type DistanceCode = u32;
type Distance = u32;
type HSkip = u8;

#[derive(Debug, Clone, PartialEq)]
enum PrefixCodeKind {
	Simple,
	Complex(HSkip),
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
	fn new_simple(n_sym: Option<NSym>, symbols: Option<Symbols>, tree_select:Option<TreeSelect>) -> PrefixCode {
		PrefixCode::Simple(PrefixCodeSimple {
			n_sym: n_sym,
			symbols: symbols,
			tree_select: tree_select,
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
	count_output: usize,
	context_modes_literals: Option<ContextModes>,
	prefix_trees_literals: Option<Vec<HuffmanCodes>>,
	prefix_tree_insert_and_copy_lengths: Option<HuffmanCodes>,
	prefix_trees_distances: Option<Vec<HuffmanCodes>>,
	btype_l: Option<NBltypes>,
	blen_l: Option<BLen>,
	btype_i: Option<NBltypes>,
	blen_i: Option<BLen>,
	btype_d: Option<NBltypes>,
	blen_d: Option<BLen>,
	insert_and_copy_length: Option<Symbol>,
	insert_length: Option<InsertLength>,
	copy_length: Option<CopyLength>,
	distance_code: Option<DistanceCode>,
	distance: Option<Distance>,
}

impl MetaBlock {
	fn new() -> MetaBlock {
		MetaBlock{
			header: MetaBlockHeader::new(),
			count_output: 0,
			btype_l: None,
			blen_l: None,
			btype_i: None,
			blen_i: None,
			btype_d: None,
			blen_d: None,
			context_modes_literals: None,
			prefix_trees_literals: None,
			prefix_tree_insert_and_copy_lengths: None,
			prefix_trees_distances: None,
			insert_and_copy_length: None,
			insert_length: None,
			copy_length: None,
			distance_code: None,
			distance: None,
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
	n_trees_l: Option<NTrees>,
	n_trees_d: Option<NTrees>,
	c_map_d: Option<ContextMap>,
	c_map_l: Option<ContextMap>,
	prefix_codes_literals: Option<Vec<PrefixCode>>,
	prefix_code_insert_and_copy_lengths: Option<PrefixCode>,
	prefix_codes_distances: Option<Vec<PrefixCode>>,
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
			n_trees_l: None,
			n_trees_d: None,
			c_map_d: None,
			c_map_l: None,
			prefix_codes_literals: None,
			prefix_code_insert_and_copy_lengths: None,
			prefix_codes_distances: None,
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
	NTreesL(NTrees),
	NTreesD(NTrees),
	ContextMapDistances(ContextMap),
	ContextMapLiterals(ContextMap),
	PrefixCodesLiterals(Vec<(PrefixCode, HuffmanCodes)>),
	PrefixCodeInsertAndCopyLengths((PrefixCode, HuffmanCodes)),
	PrefixCodesDistances(Vec<(PrefixCode, HuffmanCodes)>),
	DataMetaBlockBegin,
	InsertAndCopyLength(InsertAndCopyLength),
	InsertLengthAndCopyLength(InsertLengthAndCopyLength),
	InsertLiterals(Literals),
	DistanceCode(DistanceCode),
	Distance(Distance),
	CopyLiterals(Literals),
	DataMetaBlockEnd,
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
	ParseErrorInsertAndCopyLength,
	ParseErrorInsertLiterals,
	ParseErrorContextMap,
	ParseErrorDistanceCode,
	ExceededExpectedBytes,
	ParseErrorComplexPrefixCodeLengths,
	RingBufferError,
	RunLengthExceededSizeOfContextMap,
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
			&DecompressorError::ParseErrorInsertAndCopyLength => "Error parsing Insert And Copy Length",
			&DecompressorError::ParseErrorInsertLiterals => "Error parsing Insert Literals",
			&DecompressorError::ParseErrorDistanceCode => "Error parsing DistanceCode",
			&DecompressorError::ExceededExpectedBytes => "More uncompressed bytes than expected in meta-block",
			&DecompressorError::ParseErrorContextMap => "Error parsing context map",
			&DecompressorError::ParseErrorComplexPrefixCodeLengths => "Error parsing code lengths for complex prefix code",
			&DecompressorError::RingBufferError => "Error accessing distance ring buffer",
			&DecompressorError::RunLengthExceededSizeOfContextMap => "Run length excceeded declared length of context map",
		}
	}
}

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
	/// ring buffer for last 2 literals, gets set
	/// at the beginning of the stream, and then
	/// lives until the end
	literal_buf: RingBuffer<Literal>,
	/// ring buffer for last 4 distances, gets set
	/// at the beginning of the stream, and then
	/// lives until the end
	distance_buf: RingBuffer<Distance>,
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
			literal_buf: RingBuffer::from_vec(vec![0, 0]),
			distance_buf: RingBuffer::from_vec(vec![4, 11, 15, 16]),
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
			Ok(h_skip) => Ok(PrefixCodeKind::Complex(h_skip)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_simple_prefix_code(&mut self, alphabet_size: usize) -> result::Result<(PrefixCode, HuffmanCodes), DecompressorError> {
		let bit_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as usize;

		println!("Bit Width = {:?}", bit_width);

		let n_sym = match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => (my_u8 + 1) as usize,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		println!("NSYM = {:?}", n_sym);

		let tree_select = match n_sym {
			4 => match self.in_stream.read_bit() {
				Ok(v) => Some(v),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			},
			_ => None,
		};

		let mut symbols = vec![0; n_sym];
		for i in 0..n_sym {
			symbols[i] = match self.in_stream.read_u16_from_n_bits(bit_width) {
				Ok(my_u8) => my_u8 as Symbol,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		println!("Symbols = {:?}", symbols);

		let code_lengths = match (n_sym, tree_select) {
			(1, None) => vec![],
			(2, None) => {
				symbols.sort();
				vec![1, 1]
			},
			(3, None) => {
				symbols[1..3].sort();
				vec![1, 2, 2]
			},
			(4, Some(false)) => {
				symbols.sort();
				vec![2, 2, 2, 2]
			},
			(4, Some(true)) => {
				symbols[2..4].sort();
				vec![1, 2, 3, 3]
			},
			_ => unreachable!(),
		};

		println!("Sorted Symbols = {:?}", symbols);
		println!("Code Lengths = {:?}", code_lengths);

		Ok((PrefixCode::new_simple(Some(n_sym as u8), Some(symbols.clone()), tree_select),
            huffman::codes_from_lengths_and_symbols(code_lengths, &symbols)))
	}

	fn parse_complex_prefix_code(&mut self, h_skip: u8, alphabet_size: usize) -> result::Result<(PrefixCode, HuffmanCodes), DecompressorError> {
		// @TODO: probably need to use parameter alphabet_size here to be able to
		//        reject streams with excessive repeated trailing zeros, as per section
		//        3.5. of the RFC:
		//        "If the number of times to repeat the previous length
		//         or repeat a zero length would result in more lengths in
		//         total than the number of symbols in the alphabet, then the
		//         stream should be rejected as invalid."

		let mut symbols = vec![1, 2, 3, 4, 0, 5, 17, 6, 16, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let bit_lengths_code = {
			let bit_lengths_patterns = vec![
				vec![false, false],
				vec![true, true, true, false],
				vec![true, true, false],
				vec![false, true],
				vec![true, false],
				vec![true, true, true, true],
			];
			let symbols = vec![0, 1, 2, 3, 4, 5];
			let mut codes = Tree::new();

			for i in 0..bit_lengths_patterns.len() {
				codes.insert(bit_lengths_patterns[i].clone(), symbols[i]);
			}

			codes
		};

		println!("Bit Lengths Code {:?}", bit_lengths_code);

		let mut code_lengths = vec![0; symbols.len()];
		let mut sum = 0usize;

		for i in (h_skip as usize)..symbols.len() {

			code_lengths[i] = match bit_lengths_code.lookup_symbol(&mut self.in_stream) {
				Some(code_length) => code_length as usize,
				None => return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths),
			};

			if code_lengths[i] > 0 {

				sum += 32 >> code_lengths[i];

				// println!("code length = {:?}", code_lengths[i]);
				// println!("32 >> code length = {:?}", 32 >> code_lengths[i]);
				// println!("sum = {:?}", sum);

				if sum == 32 {
					break;
				}
			}
		}

		println!("Code Lengths = {:?}", code_lengths);
		println!("Symbols = {:?}", symbols);

		code_lengths = vec![code_lengths[4], code_lengths[0], code_lengths[1], code_lengths[2], code_lengths[3], code_lengths[5], code_lengths[7], code_lengths[9], code_lengths[10], code_lengths[11], code_lengths[12], code_lengths[13], code_lengths[14], code_lengths[15], code_lengths[16], code_lengths[17], code_lengths[8], code_lengths[6]];
		symbols = (0..18).collect::<Vec<_>>();

		println!("Code Lengths = {:?}", code_lengths);
		println!("Symbols = {:?}", symbols);

		let prefix_code_code_lengths = huffman::codes_from_lengths_and_symbols(code_lengths, &symbols);

		println!("Prefix Code CodeLengths = {:?}", prefix_code_code_lengths);

		let mut actual_code_lengths = Vec::new();
		let mut sum = 0usize;
		let mut last_symbol = None;
		let mut last_repeat = None;
		let mut last_non_zero_codelength = 8;

		loop {
			match prefix_code_code_lengths.lookup_symbol(&mut self.in_stream) {
				Some(new_code_length @ 0...15) => {
					print!(" {:?},", (actual_code_lengths.len() as u8) as char);

					actual_code_lengths.push(new_code_length as usize);
					last_symbol = Some(new_code_length);
					last_repeat = None;

					// println!("code length = {:?}", new_code_length);

					if new_code_length > 0 {
						last_non_zero_codelength = new_code_length;
						sum += 32768 >> new_code_length;

						// println!("32768 >> code length = {:?}", 32768 >> new_code_length);
						// println!("sum = {:?}", sum);

						if sum == 32768 {
							break;
						}
					}
				},
				Some(16) => {
					let extra_bits = match self.in_stream.read_u8_from_n_bits(2) {
						Ok(my_u8) => my_u8 as usize,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					last_repeat = match (last_symbol, last_repeat) {
						(Some(16), Some(last_repeat)) => {
							let new_repeat: usize = (4 * (last_repeat - 2)) + extra_bits as usize + 3;

							for _ in 0..new_repeat - last_repeat {
								actual_code_lengths.push(last_non_zero_codelength as usize);

								sum += 32768 >> last_non_zero_codelength;

								// println!("32768 >> code length = {:?}", 32768 >> new_code_length);
								// println!("sum = {:?}", sum);

								if sum == 32768 {
									break;
								}
							}

							if sum == 32768 {
								break;
							}

							Some(new_repeat)
						},
						(_, _) => {
							let repeat = 3 + extra_bits as usize;

							for _ in 0..repeat {
								actual_code_lengths.push(last_non_zero_codelength as usize);

								sum += 32768 >> last_non_zero_codelength;

								// println!("32768 >> code length = {:?}", 32768 >> new_code_length);
								// println!("sum = {:?}", sum);

								if sum == 32768 {
									break;
								}
							}

							if sum == 32768 {
								break;
							}

							Some(repeat)
						},
					};

					last_symbol = Some(16);
				},
				Some(17) => {
					let extra_bits = match self.in_stream.read_u8_from_n_bits(3) {
						Ok(my_u8) => my_u8,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					// println!("code length = 17, extra bits = {:?}", extra_bits);


					last_repeat = match (last_symbol, last_repeat) {
						(Some(17), Some(last_repeat)) => {
							let new_repeat = (8 * (last_repeat - 2)) + extra_bits as usize + 3;

							for _ in 0..new_repeat - last_repeat {
								actual_code_lengths.push(0);
							}

							Some(new_repeat)
						},
						(_, _) => {
							let repeat = 3 + extra_bits as usize;

							for _ in 0..repeat {
								actual_code_lengths.push(0);
							}

							Some(repeat)
						},
					};

					last_symbol = Some(17);
				},
				Some(_) => unreachable!(),
				None => return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths),
			};
		}

		println!("");

		println!("Actual Code Lengths = {:?}", actual_code_lengths);

		Ok((PrefixCode::Complex,
			huffman::codes_from_lengths(actual_code_lengths)))
	}

	fn parse_prefix_code(&mut self, alphabet_size: usize) -> result::Result<(PrefixCode, HuffmanCodes), DecompressorError> {
		let prefix_code_kind = match self.parse_prefix_code_kind() {
			Ok(kind) => kind,
			Err(e) => return Err(e),
		};

		println!("Prefix Code Kind = {:?}", prefix_code_kind);

		match prefix_code_kind {
			PrefixCodeKind::Complex(h_skip) => self.parse_complex_prefix_code(h_skip, alphabet_size),
			PrefixCodeKind::Simple => self.parse_simple_prefix_code(alphabet_size),
		}
	}

	fn parse_prefix_codes_literals(&mut self) -> result::Result<State, DecompressorError> {
		let n_trees_l = self.meta_block.header.n_trees_l.unwrap() as usize;
		let mut prefix_codes = Vec::with_capacity(n_trees_l);
		let alphabet_size = 256;

		for _ in 0..n_trees_l {

			prefix_codes.push(match self.parse_prefix_code(alphabet_size) {
				Ok(prefix_code) => prefix_code,
				Err(e) => return Err(e),
			});
		}

		Ok(State::PrefixCodesLiterals(prefix_codes))
	}

	fn parse_prefix_code_insert_and_copy_lengths(&mut self) -> result::Result<State, DecompressorError> {
		let alphabet_size = 704;

		match self.parse_prefix_code(alphabet_size) {
			Ok(prefix_code) => Ok(State::PrefixCodeInsertAndCopyLengths(prefix_code)),
			Err(e) => Err(e),
		}
	}

	fn parse_prefix_codes_distances(&mut self) -> result::Result<State, DecompressorError> {
		let n_trees_d = self.meta_block.header.n_trees_d.unwrap() as usize;
		let mut prefix_codes = Vec::with_capacity(n_trees_d);
		let alphabet_size = 16 + self.meta_block.header.n_direct.unwrap() as usize + 48 << self.meta_block.header.n_postfix.unwrap() as usize;

		for _ in 0..n_trees_d {
			prefix_codes.push(match self.parse_prefix_code(alphabet_size) {
				Ok(prefix_code) => prefix_code,
				Err(e) => return Err(e),
			});
		}

		Ok(State::PrefixCodesDistances(prefix_codes))
	}

	fn parse_context_map(&mut self, n_trees: NTrees, len: usize) -> result::Result<ContextMap, DecompressorError> {
		let rlemax = match self.in_stream.read_bit() {
			Ok(false) => 0u16,
			Ok(true) => match self.in_stream.read_u16_from_n_bits(4) {
				Ok(my_u16) => my_u16 + 1,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			},
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		println!("RLEMAX = {:?}", rlemax);

		let alphabet_size = (rlemax + n_trees as u16) as usize;

		println!("Alphabet Size = {:?}", alphabet_size);

		let (prefix_code, prefix_tree) = match self.parse_prefix_code(alphabet_size) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};

		println!("Prefix Code Context Map = {:?}", prefix_code);
		println!("Prefix Tree Context Map = {:?}", prefix_tree);

		let mut c_map = Vec::with_capacity(len);
		let mut c_pushed = 0;

		while c_pushed < len {
			match prefix_tree.lookup_symbol(&mut self.in_stream) {
				Some(run_length_code) if run_length_code > 0 && run_length_code <= rlemax => {
					println!("run length code = {:?}", run_length_code);

					let repeat = match self.in_stream.read_u16_from_n_bits(run_length_code as usize) {
						Ok(my_u16) => (1 << run_length_code) + my_u16,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					println!("repeat = {:?}", repeat);

					for _ in 0..repeat {
						c_map.push(0);
						c_pushed += 1;

						if c_pushed > len {
							return Err(DecompressorError::RunLengthExceededSizeOfContextMap);
						}
					}
				},
				Some(context_id) => {
					println!("context id = {:?}", context_id - rlemax);
					c_map.push((context_id - rlemax) as u8);
					c_pushed += 1;
				},
				None => return Err(DecompressorError::ParseErrorContextMap),
			}

			println!("{:?}", (c_pushed, len));
		}

		let imtf_bit = match self.in_stream.read_bit() {
			Ok(v) => v,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		println!("IMTF BIT = {:?}", imtf_bit);

		Self::inverse_move_to_front_transform(&mut c_map);

		Ok(c_map)
	}

	fn parse_context_map_literals(&mut self) -> result::Result<State, DecompressorError> {
		let n_trees = self.meta_block.header.n_trees_l.unwrap();
		let len = (self.meta_block.header.n_bltypes_l.unwrap() * 64) as usize;
		match self.parse_context_map(n_trees, len) {
			Ok(c_map_l) => Ok(State::ContextMapLiterals(c_map_l)),
			Err(e) => Err(e),
		}
	}

	fn parse_context_map_distances(&mut self) -> result::Result<State, DecompressorError> {
		let n_trees = self.meta_block.header.n_trees_d.unwrap();
		let len = (self.meta_block.header.n_bltypes_d.unwrap() * 4) as usize;
		match self.parse_context_map(n_trees, len) {
			Ok(c_map_d) => Ok(State::ContextMapDistances(c_map_d)),
			Err(e) => Err(e),
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

	fn parse_insert_and_copy_length(&mut self) -> result::Result<State, DecompressorError> {
		match self.meta_block.header.prefix_code_insert_and_copy_lengths {
			Some(PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(1),
				symbols: Some(ref symbols),
				tree_select: _,
			})) => Ok(State::InsertAndCopyLength(symbols[0])),
			Some(PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(2...4),
				symbols: _,
				tree_select: _,
			})) => match self.meta_block.prefix_tree_insert_and_copy_lengths.as_ref().unwrap().lookup_symbol(&mut self.in_stream) {
				Some(symbol) => Ok(State::InsertAndCopyLength(symbol)),
				None => Err(DecompressorError::ParseErrorInsertAndCopyLength),
			},
			Some(PrefixCode::Complex) => match self.meta_block.prefix_tree_insert_and_copy_lengths.as_ref().unwrap().lookup_symbol(&mut self.in_stream) {
				Some(symbol) => Ok(State::InsertAndCopyLength(symbol)),
				None => Err(DecompressorError::ParseErrorInsertAndCopyLength),
			},
			_ => unreachable!()
		}
	}

	fn decode_insert_and_copy_length(&mut self) -> result::Result<State, DecompressorError> {
		let (mut insert_length_code, mut copy_length_code) = match self.meta_block.insert_and_copy_length {
			Some(0...63) => (0, 0),
			Some(64...127) => (0, 8),
			Some(128...191) => (0, 0),
			Some(192...255) => (0, 8),
			Some(256...319) => (8, 0),
			Some(320...383) => (8, 8),
			Some(384...447) => (0, 16),
			Some(448...511) => (16, 0),
			Some(512...575) => (8, 16),
			Some(576...639) => (16, 8),
			Some(640...703) => (16, 16),
			_ => unreachable!(),
		};

		insert_length_code += 0x07 & (self.meta_block.insert_and_copy_length.unwrap() as u8 >> 3);
		copy_length_code += 0x07 & self.meta_block.insert_and_copy_length.unwrap() as u8;

		println!("(insert code, copy code) = {:?}", (insert_length_code, copy_length_code));

		let (mut insert_length, extra_bits_insert): (InsertLength, _) = match insert_length_code {
			0...5 => (insert_length_code as InsertLength, 0),
			6...7 => (6 + 2 * (insert_length_code as InsertLength - 6) , 1),
			8...9 => (10 + 4 * (insert_length_code as InsertLength - 8) , 2),
			10...11 => (18 + 8 * (insert_length_code as InsertLength - 10) , 3),
			12...13 => (34 + 16 * (insert_length_code as InsertLength - 12) , 4),
			14...15 => (66 + 32 * (insert_length_code as InsertLength - 14) , 5),
			16 => (130, 6),
			17 => (194, 7),
			18 => (322, 8),
			19 => (578, 9),
			20 => (1090, 10),
			21 => (2114, 12),
			22 => (6210, 14),
			23 => (22594, 24),
			_ => unreachable!(),
		};

		insert_length += match self.in_stream.read_u32_from_n_bits(extra_bits_insert) {
			Ok(my_u32) => my_u32 as InsertLength,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		let (mut copy_length, extra_bits_insert): (CopyLength, _) = match copy_length_code {
			0...7 => (copy_length_code as CopyLength + 2, 0),
			8...9 => (10 + 2 * (copy_length_code as CopyLength - 8) , 1),
			10...11 => (14 + 4 * (copy_length_code as CopyLength - 10) , 2),
			12...13 => (22 + 8 * (copy_length_code as CopyLength - 12) , 3),
			14...15 => (38 + 16 * (copy_length_code as CopyLength - 14) , 4),
			16...17 => (70 + 32 * (copy_length_code as CopyLength - 16) , 5),
			18 => (134, 6),
			19 => (198, 7),
			20 => (326, 8),
			21 => (582, 9),
			22 => (1094, 10),
			23 => (2118, 24),
			_ => unreachable!(),
		};

		copy_length += match self.in_stream.read_u32_from_n_bits(extra_bits_insert) {
			Ok(my_u32) => my_u32 as CopyLength,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		Ok(State::InsertLengthAndCopyLength((insert_length, copy_length)))
	}

	fn parse_insert_literals(&mut self) -> result::Result<State, DecompressorError> {
		let insert_length = self.meta_block.insert_length.unwrap() as usize;
		let mut literals = vec![0; insert_length];

		for i in 0..insert_length {
			match self.meta_block.blen_l {
				None => {},
				Some(0) => {
					// @TODO read new block type and length (switch command)
					unimplemented!()
				},
				Some(ref mut blen_l) => *blen_l -= 1,
			}

			let btype = self.meta_block.btype_l.unwrap() as usize;
			let context_mode = self.meta_block.context_modes_literals.as_ref().unwrap()[btype];
			let cid = match context_mode {
				0 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;

					p1 & 0x3f
				},
				1 => unimplemented!(),
				2 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;
					let p2 = *self.literal_buf.nth(1).unwrap() as usize;

					LUT_0[p1] | LUT_1[p2]
				},
				3 => unimplemented!(),
				_ => unreachable!(),
			};

			let index = self.meta_block.header.c_map_l.as_ref().unwrap()[btype * 64 + cid as usize] as usize;

			println!("literal prefix code index = {:?}", index);

			literals[i] = match self.meta_block.header.prefix_codes_literals.as_ref().unwrap()[index] {
				PrefixCode::Simple(PrefixCodeSimple {
					n_sym: Some(1),
					symbols: Some(ref symbols),
					tree_select: _,
				}) => symbols[0] as Literal,
				PrefixCode::Simple(PrefixCodeSimple {
					n_sym: Some(2...4),
					symbols: _,
					tree_select: _,
				}) => match self.meta_block.prefix_trees_literals.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
					Some(symbol) => symbol as Literal,
					None => return Err(DecompressorError::ParseErrorInsertLiterals),
				},
				PrefixCode::Complex => match self.meta_block.prefix_trees_literals.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
					Some(symbol) => symbol as Literal,
					None => return Err(DecompressorError::ParseErrorInsertLiterals),
				},
				_ => unreachable!(),
			};

			self.literal_buf.push(literals[i]);
		}

		Ok(State::InsertLiterals(literals))
	}

	fn parse_distance_code(&mut self) -> result::Result<State, DecompressorError> {
		// check for implicit distance 0 ([…]"as indicated by the insert-and-copy length code")
		match self.meta_block.distance {
			Some(0) => return Ok(State::DistanceCode(0)),
			Some(_) => unreachable!(),
			None => {}
		};

		match self.meta_block.blen_d {
			None => {},
			Some(0) => unreachable!(), // Note: blen_d == 0 should have been caught before calling this method
			Some(ref mut blen_d) => *blen_d -= 1,
		}

		let cid = match self.meta_block.copy_length {
			Some(0...1) => unreachable!(),
			Some(c @ 2...4) => c - 2,
			Some(_) => 3,
			_ => unreachable!(),
		};

		let index = self.meta_block.header.c_map_d.as_ref().unwrap()[self.meta_block.btype_d.unwrap() as usize * 4 + cid as usize] as usize;

		println!("distance prefix code index = {:?}", index);
		println!("distance prefix code = {:?}", self.meta_block.header.prefix_codes_distances.as_ref().unwrap()[index]);

		let distance_code = match self.meta_block.header.prefix_codes_distances.as_ref().unwrap()[index] {
			PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(1),
				symbols: Some(ref symbols),
				tree_select: _,
			}) => symbols[0] as DistanceCode,
			PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(2...4),
				symbols: _,
				tree_select: _,
			}) => match self.meta_block.prefix_trees_distances.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
				Some(symbol) => symbol as DistanceCode,
				None => return Err(DecompressorError::ParseErrorDistanceCode),
			},
			PrefixCode::Complex => match self.meta_block.prefix_trees_distances.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
				Some(symbol) => symbol as DistanceCode,
				None => return Err(DecompressorError::ParseErrorDistanceCode),
			},
			_ => unreachable!(),
		};

		Ok(State::DistanceCode(distance_code))
	}

	fn decode_distance(&mut self) -> result::Result<State, DecompressorError> {
		let distance = match self.meta_block.distance_code {
			Some(d @ 0...3) => match self.distance_buf.nth(d as usize) {
				Ok(distance) => *distance,
				Err(_) => return Err(DecompressorError::RingBufferError),
			},
			Some(d @ 4...9) => {
				match (self.distance_buf.nth(0), 2 * (d as i64 % 2) - 1, (d - 2) >> 1) {
					(Ok(distance), sign, d) => (*distance as i64 + (sign as i64 * d as i64)) as u32,
					(Err(_), _, _) => return Err(DecompressorError::RingBufferError),
				}
			},
			// reference distance_buf here, to get the decoded distance
			Some(d @ 10...15) => {
				match (self.distance_buf.nth(1), 2 * (d as i64 % 2) - 1, (d - 8) >> 1) {
					(Ok(distance), sign, d) => {
						(*distance as i64 + (sign as i64 * d as i64)) as u32
					},
					(Err(_), _, _) => return Err(DecompressorError::RingBufferError),
				}
			},
			Some(dcode) if dcode <= (15 + self.meta_block.header.n_direct.unwrap() as DistanceCode) => dcode - 15,
			Some(dcode) => {
				let (n_direct, n_postfix) = (self.meta_block.header.n_direct.unwrap() as DistanceCode, self.meta_block.header.n_postfix.unwrap());
				let ndistbits = 1 + ((dcode - (n_direct) - 16) >> (n_postfix + 1));

				println!("NDISTBITS = {:?}", ndistbits);

				let dextra = match self.in_stream.read_u32_from_n_bits(ndistbits as usize) {
					Ok(my_u32) => my_u32,
					Err(_) => return Err(DecompressorError::UnexpectedEOF),
				};

				println!("DEXTRA = {:?}", dextra);

				let hcode = (dcode - n_direct - 16) >> n_postfix;

				println!("HCODE = {:?}", hcode);

				let postfix_mask = (1 << n_postfix) - 1;
				let lcode = (dcode - n_direct - 16) & postfix_mask;

				println!("LCODE = {:?}", lcode);

				let offset = ((2 + (hcode & 1)) << ndistbits) - 4;

				println!("Offset = {:?}", offset);

				let distance = ((offset + dextra) << n_postfix) + lcode + n_direct + 1;

				println!("Distance = {:?}", distance);

				distance
			},
			None => unreachable!()
		};

		println!("(dc, db, d) = {:?}", (self.meta_block.distance_code, self.distance_buf.clone(), distance));

		if self.meta_block.distance_code.unwrap() > 0 && distance as usize <= cmp::min(self.header.window_size.unwrap(), self.count_output) {
			self.distance_buf.push(distance);
		}

		Ok(State::Distance(distance))
	}

	fn copy_literals(&mut self) -> result::Result<State, DecompressorError> {
		let window_size = self.header.window_size.unwrap();
		let copy_length = self.meta_block.copy_length.unwrap() as usize;
		let count_output = self.count_output;
		let distance = self.meta_block.distance.unwrap() as usize;
		let ref output_window = self.output_window;

		if distance <= cmp::min(count_output, window_size) {
			let mut window = vec![0; copy_length];
			let l = cmp::min(distance, copy_length);

			for i in (count_output + window_size - distance)..(count_output + window_size - distance + l) {

				window[i - (count_output + window_size - distance)] = output_window[i % window_size];
			}

			for i in l..copy_length {

				window[i] = window[i % l];
			}

			Ok(State::CopyLiterals(window))
		} else {

			// @TODO handle distance for static dictionary
			unimplemented!();
		}

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

					println!("ISLAST = true");

					self.state = match self.parse_is_last_empty() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::IsLast(false) => {
					self.meta_block.header.is_last = Some(false);

					println!("ISLAST = false");

					self.state = match self.parse_m_nibbles() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::IsLastEmpty(true) => {
					self.meta_block.header.is_last_empty = Some(true);

					println!("ISLASTEMPTY = true");


					self.state = State::StreamEnd;
				},
				State::IsLastEmpty(false) => {
					self.meta_block.header.is_last_empty = Some(false);

					println!("ISLASTEMPTY = false");

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
					self.meta_block.btype_l = Some(0);

					println!("NBLTYPESL = {:?}", n_bltypes_l);

					self.state = if n_bltypes_l >= 2 {
						// @TODO parse prefix codes for block type and block count etc.
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
					self.meta_block.btype_i = Some(0);

					println!("NBLTYPESI = {:?}", n_bltypes_i);

					self.state = if n_bltypes_i >= 2 {
						// @TODO parse prefix codes for block type and block count etc.
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
					self.meta_block.btype_d = Some(0);

					println!("NBLTYPESD = {:?}", n_bltypes_d);

					self.state = if n_bltypes_d >= 2 {
						// @TODO parse prefix codes for block type and block count etc.
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
					self.meta_block.context_modes_literals = Some(context_modes);

					println!("Context Modes Literals = {:?}", self.meta_block.context_modes_literals);

					self.state = match self.parse_n_trees_l() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::NTreesL(n_trees_l) => {
					self.meta_block.header.n_trees_l = Some(n_trees_l);
					self.meta_block.header.c_map_l = Some(vec![0; 64 * self.meta_block.header.n_bltypes_l.unwrap() as usize]);

					println!("NTREESL = {:?}", n_trees_l);

					self.state = if n_trees_l >= 2 {
						match self.parse_context_map_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					} else {
						match self.parse_n_trees_d() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::ContextMapLiterals(c_map_l) => {
					self.meta_block.header.c_map_l = Some(c_map_l);

					println!("CMAPL = {:?}", self.meta_block.header.c_map_l);

					self.state = match self.parse_n_trees_d() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::NTreesD(n_trees_d) => {
					self.meta_block.header.n_trees_d = Some(n_trees_d);
					self.meta_block.header.c_map_d = Some(vec![0; 4 * self.meta_block.header.n_bltypes_d.unwrap() as usize]);

					println!("NTREESD = {:?}", n_trees_d);

					self.state = if n_trees_d >= 2 {
						match self.parse_context_map_distances() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					} else {
						match self.parse_prefix_codes_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::ContextMapDistances(c_map_d) => {
					self.meta_block.header.c_map_d = Some(c_map_d);

					println!("CMAPD = {:?}", self.meta_block.header.c_map_d);

					self.state = match self.parse_prefix_codes_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::PrefixCodesLiterals(prefix_codes_and_trees) => {
					let (prefix_codes, prefix_trees): (Vec<_>, Vec<_>) = prefix_codes_and_trees.iter().cloned().unzip();

					self.meta_block.header.prefix_codes_literals = Some(prefix_codes);
					self.meta_block.prefix_trees_literals = Some(prefix_trees);

					println!("Prefix Code Literals = {:?}", self.meta_block.header.prefix_codes_literals);
					println!("Prefix Tree literals = {:?}", self.meta_block.prefix_trees_literals);

					self.state = match self.parse_prefix_code_insert_and_copy_lengths() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::PrefixCodeInsertAndCopyLengths((prefix_code, prefix_tree)) => {
					self.meta_block.header.prefix_code_insert_and_copy_lengths = Some(prefix_code);
					self.meta_block.prefix_tree_insert_and_copy_lengths = Some(prefix_tree);

					println!("Prefix Code Insert And Copy Lengths = {:?}", self.meta_block.header.prefix_code_insert_and_copy_lengths);
					println!("Prefix Tree Insert And Copy Lengths = {:?}", self.meta_block.prefix_tree_insert_and_copy_lengths);

					self.state = match self.parse_prefix_codes_distances() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::PrefixCodesDistances(prefix_codes_and_trees) => {
					let (prefix_codes, prefix_trees): (Vec<_>, Vec<_>) = prefix_codes_and_trees.iter().cloned().unzip();

					self.meta_block.header.prefix_codes_distances = Some(prefix_codes);
					self.meta_block.prefix_trees_distances = Some(prefix_trees);

					println!("Prefix Codes Distances = {:?}", self.meta_block.header.prefix_codes_distances);
					println!("Prefix Trees Distances = {:?}", self.meta_block.prefix_trees_distances);

					self.state = State::DataMetaBlockBegin;
				},
				State::DataMetaBlockBegin => {
					self.state = match (self.meta_block.header.n_bltypes_i, self.meta_block.blen_i) {
						(Some(1), _) => match self.parse_insert_and_copy_length() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						// @TODO parse block switch command for insert and copy length
						(_, Some(0)) => unimplemented!(),
						(Some(_), Some(_)) =>  match self.parse_insert_and_copy_length() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						_ => unreachable!(),
					};
				},
				State::InsertAndCopyLength(insert_and_copy_length) => {
					self.meta_block.insert_and_copy_length = Some(insert_and_copy_length);

					self.meta_block.distance = match insert_and_copy_length {
						0...127 => Some(0),
						_ => None,
					};

					println!("Insert And Copy Length = {:?}", insert_and_copy_length);

					self.state = match self.decode_insert_and_copy_length() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::InsertLengthAndCopyLength(insert_length_and_copy_length) => {
					match insert_length_and_copy_length {
						(in_len, co_len) => {
							self.meta_block.insert_length = Some(in_len);
							self.meta_block.copy_length = Some(co_len);
						},
					};

					println!("Insert Length and Copy Length = {:?}", insert_length_and_copy_length);

					self.state = match (self.meta_block.header.n_bltypes_l, self.meta_block.blen_l) {
						(Some(0), _) => unreachable!(),
						// @TODO should parse block switch command for literals here
						(_, Some(0)) => unimplemented!(),
						(Some(_), _) =>  match self.parse_insert_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						_ => unreachable!(),
					};
				},
				State::InsertLiterals(insert_literals) => {
					for literal in insert_literals {
						self.buf.push_front(literal);
						self.output_window[self.count_output % self.header.window_size.unwrap() as usize] = literal;
						self.count_output += 1;
						self.meta_block.count_output += 1;
					}

					self.state = if self.meta_block.header.m_len.unwrap() as usize == self.meta_block.count_output {
						State::DataMetaBlockEnd
					} else {
						match (self.meta_block.header.n_bltypes_d, self.meta_block.blen_d) {
							(Some(0), _) => unreachable!(),
							// @TODO should parse block switch command for distances here
							(_, Some(0)) => unimplemented!(),
							(Some(_), _) =>  match self.parse_distance_code() {
								Ok(state) => state,
								Err(_) => return Err(DecompressorError::UnexpectedEOF),
							},
							_ => unreachable!(),
						}
					};

					if self.buf.len() > 0 {
						return Ok(self.buf.len());
					}
				},
				State::DistanceCode(distance_code) => {
					self.meta_block.distance_code = Some(distance_code);

					println!("Distance Code = {:?}", distance_code);

					self.state = match self.decode_distance() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::Distance(distance) => {
					self.meta_block.distance = Some(distance);

					println!("Distance = {:?}", distance);

					self.state = match self.copy_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::CopyLiterals(copy_literals) => {
					for literal in copy_literals {
						self.buf.push_front(literal);
						self.literal_buf.push(literal);
						self.output_window[self.count_output % self.header.window_size.unwrap() as usize] = literal;
						self.count_output += 1;
						self.meta_block.count_output += 1;
					}

					println!("output = {:?}", self.buf);


					if (self.meta_block.header.m_len.unwrap() as usize) < self.meta_block.count_output {

						return Err(DecompressorError::ExceededExpectedBytes);
					}

					self.state = if self.meta_block.header.m_len.unwrap() as usize == self.meta_block.count_output {

						State::DataMetaBlockEnd
					} else {

						State::DataMetaBlockBegin
					};


					let output_so_far = String::from_utf8(self.output_window.clone().into_iter().filter(|&b| b > 0).collect::<Vec<_>>()).unwrap();
					let expected = String::from("znxcvnmz,xvnm.,zxcnv.,xcn.z,vn.zvn.zxcvn.,zxcn.vn.v,znm.,vnzx.,vnzxc.vn.z,vnz.,nv.z,nvmzxc,nvzxcvcnm.,vczxvnzxcnvmxc.zmcnvzm.,nvmc,nzxmc,vn.mnnmzxc,vnxcnmv,znvzxcnmv,.xcnvm,zxcnzxv.zx,qweryweurqioweupropqwutioweupqrioweutiopweuriopweuriopqwurioputiopqwuriowuqerioupqweropuweropqwurweuqriopuropqwuriopuqwriopuqweopruioqweurqweuriouqweopruioupqiytioqtyiowtyqptypryoqweutioioqtweqruowqeytiowquiourowetyoqwupiotweuqiorweuqroipituqwiorqwtioweuriouytuioerytuioweryuitoweytuiweyuityeruirtyuqriqweuropqweiruioqweurioqwuerioqwyuituierwotueryuiotweyrtuiwertyioweryrueioqptyioruyiopqwtjkasdfhlafhlasdhfjklashjkfhasjklfhklasjdfhklasdhfjkalsdhfklasdhjkflahsjdkfhklasfhjkasdfhasfjkasdhfklsdhalghhaf;hdklasfhjklashjklfasdhfasdjklfhsdjklafsd;hkldadfjjklasdhfjasddfjklfhakjklasdjfkl;asdjfasfljasdfhjklasdfhjkaghjkashf;djfklasdjfkljasdklfjklasdjfkljasdfkljaklfj");

					println!("{}", &expected[0..output_so_far.len()]);
					println!("{}", output_so_far);

					return Ok(self.buf.len());
				},
				State::DataMetaBlockEnd => {

					self.state = State::MetaBlockEnd;
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

		let l = cmp::min(self.buf.len(), buf.len());

		for i in 0..l {

			buf[i] = self.buf.pop_back().unwrap();
		}

		Ok(l)
	}
}
