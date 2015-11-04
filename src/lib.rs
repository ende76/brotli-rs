#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces, unused_qualifications)]
//! brotli-rs provides Read adapter implementations the Brotli compression scheme.
//!
//! This allows a consumer to wrap a Brotli-compressed Stream into a Decompressor,
//! using the familiar methods provided by the Read trait for processing
//! the uncompressed stream.

/// bitreader wraps a Read to provide bit-oriented read access to a stream.
mod bitreader;
mod huffman;
/// ringbuffer provides a data structure RingBuffer that uses a single, fixed-size buffer as if it were connected end-to-end.
/// This structure lends itself easily to buffering data streams.
mod ringbuffer;


mod dictionary;
use ::dictionary::{ BROTLI_DICTIONARY_OFFSETS_BY_LENGTH, BROTLI_DICTIONARY_SIZE_BITS_BY_LENGTH, BROTLI_DICTIONARY };
mod lookuptable;
use ::lookuptable::{ LUT_0, LUT_1, LUT_2, INSERT_LENGTHS_AND_COPY_LENGTHS };
mod transformation;
use ::transformation::transformation;

use ::bitreader::{ BitReader, BitReaderError };
use ::huffman::tree::Tree;
use ::ringbuffer::RingBuffer;

use std::collections::VecDeque;
use std::cmp;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;

type WBits = u8;
type CodeLengths = Vec<usize>;
type HuffmanCodes = Tree;
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
type NBltypes = u16;
type NTrees = NBltypes;
type BLen = u32;
type BlockSwitch = (NBltypes, BLen);
type NPostfix = u8;
type NDirect = u8;
type ContextMode = u16;
type ContextModes = Vec<ContextMode>;
type ContextMap = Vec<u8>;
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
struct Header {
	wbits: Option<WBits>,
	wbits_codes: HuffmanCodes,
	window_size: Option<usize>,
	bit_lengths_code: HuffmanCodes,
	bltype_codes: HuffmanCodes,
}

impl Header {
	fn new() -> Header {
		Header{
			wbits: None,
			wbits_codes: Tree::from_raw_data(
				vec![None, Some(16), None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None,
				     Some(21), Some(19), Some(23), Some(18), Some(22), Some(20), Some(24), None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, Some(17), Some(12),
				     Some(10), Some(14), None, Some(13), Some(11), Some(15), None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None, None],
				15, Some(24)),
			bit_lengths_code: Tree::from_raw_data(
				vec![None, None, None, Some(0), Some(3), Some(4), None, None,
				     None, None, None, None, None, Some(2), None, None,
				     None, None, None, None, None, None, None, None,
				     None, None, None, None, None, Some(1), Some(5)],
				6, Some(5)),
			bltype_codes: Tree::from_raw_data(
				vec![None, Some(1), None, None, None, None, None, None,
				     None, None, None, None, None, None, None, None,
				     None, None, None, None, None, None, None, Some(2),
				     Some(17), Some(5), Some(65), Some(3), Some(33), Some(9), Some(129)],
				9, Some(129)
			),
			window_size: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
struct MetaBlock {
	header: MetaBlockHeader,
	count_output: usize,
	context_modes_literals: Option<ContextModes>,
	prefix_tree_block_types_literals: Option<HuffmanCodes>,
	prefix_tree_block_counts_literals: Option<HuffmanCodes>,
	prefix_tree_block_types_insert_and_copy_lengths: Option<HuffmanCodes>,
	prefix_tree_block_counts_insert_and_copy_lengths: Option<HuffmanCodes>,
	prefix_tree_block_types_distances: Option<HuffmanCodes>,
	prefix_tree_block_counts_distances: Option<HuffmanCodes>,
	prefix_trees_literals: Option<Vec<HuffmanCodes>>,
	prefix_trees_insert_and_copy_lengths: Option<Vec<HuffmanCodes>>,
	prefix_trees_distances: Option<Vec<HuffmanCodes>>,
	btype_l: NBltypes,
	btype_l_prev: NBltypes,
	blen_l: Option<BLen>,
	btype_i: NBltypes,
	btype_i_prev: NBltypes,
	blen_i: Option<BLen>,
	btype_d: NBltypes,
	btype_d_prev: NBltypes,
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
			btype_l: 0,
			btype_l_prev: 1,
			blen_l: None,
			btype_i: 0,
			btype_i_prev: 1,
			blen_i: None,
			btype_d: 0,
			btype_d_prev: 1,
			blen_d: None,
			context_modes_literals: None,
			prefix_tree_block_types_literals: None,
			prefix_tree_block_counts_literals: None,
			prefix_tree_block_types_insert_and_copy_lengths: None,
			prefix_tree_block_counts_insert_and_copy_lengths: None,
			prefix_tree_block_types_distances: None,
			prefix_tree_block_counts_distances: None,
			prefix_trees_literals: None,
			prefix_trees_insert_and_copy_lengths: None,
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
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum State {
	StreamBegin,
	HeaderBegin,
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
	NBltypesL(NBltypes),
	PrefixCodeBlockTypesLiterals(HuffmanCodes),
	PrefixCodeBlockCountsLiterals(HuffmanCodes),
	FirstBlockCountLiterals(BLen),
	NBltypesI(NBltypes),
	PrefixCodeBlockTypesInsertAndCopyLengths(HuffmanCodes),
	PrefixCodeBlockCountsInsertAndCopyLengths(HuffmanCodes),
	FirstBlockCountInsertAndCopyLengths(BLen),
	NBltypesD(NBltypes),
	PrefixCodeBlockTypesDistances(HuffmanCodes),
	PrefixCodeBlockCountsDistances(HuffmanCodes),
	FirstBlockCountDistances(BLen),
	NPostfix(NPostfix),
	NDirect(NDirect),
	ContextModesLiterals(ContextModes),
	NTreesL(NTrees),
	NTreesD(NTrees),
	ContextMapDistances(ContextMap),
	ContextMapLiterals(ContextMap),
	PrefixCodesLiterals(Vec<HuffmanCodes>),
	PrefixCodesInsertAndCopyLengths(Vec<HuffmanCodes>),
	PrefixCodesDistances(Vec<HuffmanCodes>),
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
	CodeLengthsChecksum,
	ExpectedEndOfStream,
	ExceededExpectedBytes,
	InvalidBlockCountCode,
	InvalidBlockSwitchCommandCode,
	InvalidBlockType,
	InvalidBlockTypeCode,
	InvalidLengthInStaticDictionary,
	InvalidMSkipLen,
	InvalidSymbol,
	InvalidTransformId,
	InvalidNonPositiveDistance,
	LessThanTwoNonZeroCodeLengths,
	NoCodeLength,
	NonZeroFillBit,
	NonZeroReservedBit,
	NonZeroTrailerBit,
	NonZeroTrailerNibble,
	ParseErrorContextMap,
	ParseErrorComplexPrefixCodeLengths,
	ParseErrorDistanceCode,
	ParseErrorInsertAndCopyLength,
	ParseErrorInsertLiterals,
	RingBufferError,
	RunLengthExceededSizeOfContextMap,
	UnexpectedEOF,
}

impl Display for DecompressorError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {

		fmt.write_str(self.description())
	}
}

impl Error for DecompressorError {
	fn description(&self) -> &str {
		match *self {
			DecompressorError::CodeLengthsChecksum => "Code length check sum did not add up in complex prefix code",
			DecompressorError::ExpectedEndOfStream => "Expected end-of-stream, but stream did not end",
			DecompressorError::ExceededExpectedBytes => "More uncompressed bytes than expected in meta-block",
			DecompressorError::InvalidBlockCountCode => "Encountered invalid value for block count code",
			DecompressorError::InvalidBlockSwitchCommandCode => "Encountered invalid value for block switch command code",
			DecompressorError::InvalidBlockType => "Encountered invalid value for block type",
			DecompressorError::InvalidBlockTypeCode => "Encountered invalid value for block type code",
			DecompressorError::InvalidLengthInStaticDictionary => "Encountered invalid length in reference to static dictionary",
			DecompressorError::InvalidMSkipLen => "Most significant byte of MSKIPLEN was zero",
			DecompressorError::InvalidSymbol => "Encountered invalid symbol in prefix code",
			DecompressorError::InvalidTransformId => "Encountered invalid transform id in reference to static dictionary",
			DecompressorError::InvalidNonPositiveDistance => "Encountered invalid non-positive distance",
			DecompressorError::LessThanTwoNonZeroCodeLengths => "Encountered invalid complex prefix code with less than two non-zero codelengths",
			DecompressorError::NoCodeLength => "Encountered invalid complex prefix code with all zero codelengths",
			DecompressorError::NonZeroFillBit => "Enocuntered non-zero fill bit",
			DecompressorError::NonZeroReservedBit => "Enocuntered non-zero reserved bit",
			DecompressorError::NonZeroTrailerBit => "Enocuntered non-zero bit trailing the stream",
			DecompressorError::NonZeroTrailerNibble => "Enocuntered non-zero nibble trailing",
			DecompressorError::ParseErrorContextMap => "Error parsing context map",
			DecompressorError::ParseErrorComplexPrefixCodeLengths => "Error parsing code lengths for complex prefix code",
			DecompressorError::ParseErrorDistanceCode => "Error parsing DistanceCode",
			DecompressorError::ParseErrorInsertAndCopyLength => "Error parsing Insert And Copy Length",
			DecompressorError::ParseErrorInsertLiterals => "Error parsing Insert Literals",
			DecompressorError::RingBufferError => "Error accessing distance ring buffer",
			DecompressorError::RunLengthExceededSizeOfContextMap => "Run length excceeded declared length of context map",
			DecompressorError::UnexpectedEOF => "Encountered unexpected EOF",
		}
	}
}

/// Wraps an input stream and provides methods for decompressing.
///
/// # Examples
/// ```
/// use std::io::{ Read, stdout, Write };
/// use brotli::Decompressor;
///
/// let brotli_stream = std::fs::File::open("data/64x.compressed").unwrap();
///
/// let mut decompressed = &mut Vec::new();
/// let _ = Decompressor::new(brotli_stream).read_to_end(&mut decompressed);
///
/// let mut expected = &mut Vec::new();
/// let _ = std::fs::File::open("data/64x").unwrap().read_to_end(&mut expected);
///
/// assert_eq!(expected, decompressed);
///
/// stdout().write_all(decompressed).ok();
#[derive(Debug)]
pub struct Decompressor<R: Read> {
	in_stream: BitReader<R>,
	header: Header,
	buf: VecDeque<Literal>,
	output_window: Option<RingBuffer<Literal>>,
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
	/// Creates Decompressor from Read.
	pub fn new(r: R) -> Decompressor<R> {
		Decompressor{
			in_stream: BitReader::new(r),
			header: Header::new(),
			buf: VecDeque::new(),
			output_window: None,
			state: State::StreamBegin,
			meta_block: MetaBlock::new(),
			count_output: 0,
			literal_buf: RingBuffer::from_vec(vec![0, 0]),
			distance_buf: RingBuffer::from_vec(vec![4, 11, 15, 16]),
		}
	}

	fn parse_wbits(&mut self) -> Result<State, DecompressorError> {
		match self.header.wbits_codes.lookup_symbol(&mut self.in_stream) {
			Ok(Some(symbol)) => Ok(State::WBits(symbol as WBits)),
			Ok(None) => Err(DecompressorError::UnexpectedEOF),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_is_last(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsLast(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_is_last_empty(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsLastEmpty(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_nibbles(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(3) => Ok(State::MNibbles(0)),
			Ok(my_u8) => Ok(State::MNibbles(my_u8 + 4)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_skip_bytes(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => Ok(State::MSkipBytes(my_u8)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_m_skip_len(&mut self) -> Result<State, DecompressorError> {
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
			for (i, byte) in bytes.iter().enumerate() {
				m_skip_len = m_skip_len | ((*byte as MSkipLen) << i);
			}
			m_skip_len + 1
		}))
	}

	fn parse_m_len(&mut self) -> Result<State, DecompressorError> {
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

	fn parse_is_uncompressed(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_bit() {
			Ok(bit) => Ok(State::IsUncompressed(bit)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_mlen_literals(&mut self) -> Result<State, DecompressorError> {
		let bytes = match self.in_stream.read_fixed_length_string(self.meta_block.header.m_len.unwrap() as usize) {
			Ok(bytes) => bytes,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		Ok(State::MLenLiterals(bytes))
	}

	fn parse_n_bltypes(&mut self) -> Result<NBltypes, DecompressorError> {

		let (value, extra_bits) = match self.header.bltype_codes.lookup_symbol(&mut self.in_stream) {
			Ok(Some(symbol @ 1...2)) => (symbol, 0),
			Ok(Some(symbol @     3)) => (symbol, 1),
			Ok(Some(symbol @     5)) => (symbol, 2),
			Ok(Some(symbol @     9)) => (symbol, 3),
			Ok(Some(symbol @    17)) => (symbol, 4),
			Ok(Some(symbol @    33)) => (symbol, 5),
			Ok(Some(symbol @    65)) => (symbol, 6),
			Ok(Some(symbol @   129)) => (symbol, 7),
			Ok(Some(_)) => unreachable!(), // confirmed unreachable, the possible symbols are defined in code
			Ok(None) => return Err(DecompressorError::UnexpectedEOF),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		if extra_bits > 0 {
			match self.in_stream.read_u16_from_n_bits(extra_bits) {
				Ok(extra) => Ok(value + extra),
				Err(_) => Err(DecompressorError::UnexpectedEOF),
			}
		} else {
			Ok(value)
		}
	}

	fn parse_n_bltypes_l(&mut self) -> Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesL(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_bltypes_i(&mut self) -> Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesI(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_bltypes_d(&mut self) -> Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NBltypesD(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_postfix(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => Ok(State::NPostfix(my_u8)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_n_direct(&mut self) -> Result<State, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(4) {
			Ok(my_u8) => Ok(State::NDirect(my_u8 << self.meta_block.header.n_postfix.unwrap())),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_context_modes_literals(&mut self) -> Result<State, DecompressorError> {
		let mut context_modes = vec![0; self.meta_block.header.n_bltypes_l.unwrap() as usize];

		for mut mode in &mut context_modes {
			match self.in_stream.read_u8_from_n_bits(2) {
				Ok(my_u8) => *mode = my_u8 as ContextMode,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		Ok(State::ContextModesLiterals(context_modes))
	}

	fn parse_n_trees_l(&mut self) -> Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NTreesL(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_n_trees_d(&mut self) -> Result<State, DecompressorError> {
		match self.parse_n_bltypes() {
			Ok(value) => Ok(State::NTreesD(value)),
			Err(e) => Err(e)
		}
	}

	fn parse_prefix_code_kind(&mut self) -> Result<PrefixCodeKind, DecompressorError> {
		match self.in_stream.read_u8_from_n_bits(2) {
			Ok(1) => Ok(PrefixCodeKind::Simple),
			Ok(h_skip) => Ok(PrefixCodeKind::Complex(h_skip)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_simple_prefix_code(&mut self, alphabet_size: usize) -> Result<HuffmanCodes, DecompressorError> {
		let bit_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as usize;

		// println!("Alphabet Size = {:?}", alphabet_size);
		// println!("Bit Width = {:?}", bit_width);

		let n_sym = match self.in_stream.read_u8_from_n_bits(2) {
			Ok(my_u8) => (my_u8 + 1) as usize,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		// println!("NSYM = {:?}", n_sym);
		// println!("global bit pos = {:?}", self.in_stream.global_bit_pos);

		let mut symbols = vec![0; n_sym];
		for symbol in &mut symbols {
			*symbol = match self.in_stream.read_u16_from_n_bits(bit_width) {
				Ok(symbol) if (symbol as usize) < alphabet_size => symbol,
				Ok(_) => return Err(DecompressorError::InvalidSymbol),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}
		}

		for i in 0..(symbols.len() - 1) {
			for j in (i + 1)..symbols.len() {
				if symbols[i] == symbols[j] {
					return Err(DecompressorError::InvalidSymbol);
				}
			}
		}

		// println!("Symbols = {:?}", symbols);
		// println!("global bit pos = {:?}", self.in_stream.global_bit_pos);


		let tree_select = match n_sym {
			4 => match self.in_stream.read_bit() {
				Ok(v) => Some(v),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			},
			_ => None,
		};

		let code_lengths = match (n_sym, tree_select) {
			(1, None) => vec![0],
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
			_ => unreachable!(), // confirmed unreachable, NSYM is read from 2 bits, tree_select is Some(_) iff NSYM == 4
		};

		// println!("Sorted Symbols = {:?}", symbols);
		// println!("Code Lengths = {:?}", code_lengths);

		Ok(huffman::codes_from_lengths_and_symbols(&code_lengths, &symbols))
	}

	fn parse_complex_prefix_code(&mut self, h_skip: u8, alphabet_size: usize)
			-> Result<HuffmanCodes, DecompressorError> {
		let mut symbols = vec![1, 2, 3, 4, 0, 5, 17, 6, 16, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let bit_lengths_code = &self.header.bit_lengths_code;

		let mut code_lengths = vec![0; symbols.len()];
		let mut sum = 0usize;
		let mut len_non_zero_codelengths = 0usize;

		for i in (h_skip as usize)..symbols.len() {

			code_lengths[i] = match bit_lengths_code.lookup_symbol(&mut self.in_stream) {
				Ok(Some(code_length)) => code_length as usize,
				Ok(None) => return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			};

			if code_lengths[i] > 0 {

				sum += 32 >> code_lengths[i];
				len_non_zero_codelengths += 1;

				// println!("code length = {:?}", code_lengths[i]);
				// println!("32 >> code length = {:?}", 32 >> code_lengths[i]);
				// println!("sum = {:?}", sum);

				if sum == 32 {
					break;
				}

				if sum > 32 {
					return Err(DecompressorError::CodeLengthsChecksum)
				}
			}
		}

		if len_non_zero_codelengths == 0 {
			return Err(DecompressorError::NoCodeLength);
		}

		if len_non_zero_codelengths >= 2 && sum < 32 {
			return Err(DecompressorError::CodeLengthsChecksum);
		}

		// println!("Code Lengths = {:?}", code_lengths);
		// println!("Symbols = {:?}", symbols);

		code_lengths = vec![code_lengths[4], code_lengths[0], code_lengths[1], code_lengths[2], code_lengths[3], code_lengths[5], code_lengths[7], code_lengths[9], code_lengths[10], code_lengths[11], code_lengths[12], code_lengths[13], code_lengths[14], code_lengths[15], code_lengths[16], code_lengths[17], code_lengths[8], code_lengths[6]];
		symbols = (0..18).collect::<Vec<_>>();

		// debug(&format!("Code Lengths = {:?}", code_lengths));
		// debug(&format!("Symbols = {:?}", symbols));

		let lone_symbol = {
			if sum < 32 {
				let mut i = 0;
				while code_lengths[i] == 0 {
					i += 1;
				}
				Some(symbols[i])
			} else {
				None
			}
		};

		// println!("Code Lengths = {:?}", code_lengths);
		// debug(&format!("Symbols = {:?}", symbols));

		let prefix_code_code_lengths = huffman::codes_from_lengths_and_symbols(&code_lengths, &symbols);

		// println!("Prefix Code CodeLengths = {:?}", prefix_code_code_lengths);
		// println!("Prefix Code CodeLengths = {:?}", prefix_code_code_lengths.buf.iter().enumerate().filter(|&(_, l)| *l != None).collect::<Vec<_>>());

		let mut actual_code_lengths = vec![0usize; alphabet_size];
		let mut sum = 0usize;
		let mut last_symbol = None;
		let mut last_repeat = None;
		let mut last_non_zero_codelength = 8;
		let mut i = 0;

		while i < alphabet_size {
			// println!("global bit pos = {:?}", self.in_stream.global_bit_pos);
			// println!("Lone Symbol = {:?}", lone_symbol);

			let code_length_code = if lone_symbol == None {
				match prefix_code_code_lengths.lookup_symbol(&mut self.in_stream) {
					Ok(symbol) => symbol,
					Err(_) => return Err(DecompressorError::UnexpectedEOF),
				}
			} else { lone_symbol };

			// debug(&format!("lone_symbol = {:?}", lone_symbol));

			// println!("code length code = {:?}", code_length_code);

			match code_length_code {
				Some(new_code_length @ 0...15) => {
					actual_code_lengths[i] = new_code_length as usize;
					i += 1;
					last_symbol = Some(new_code_length);
					last_repeat = None;

					if new_code_length > 0 {
						last_non_zero_codelength = new_code_length;

						sum += 32768 >> new_code_length;

						// debug(&format!("32768 >> code length == {:?}, sum == {:?}", 32768 >> new_code_length, sum));

						if sum == 32768 {
							break;
						} else if sum > 32768 {
							return Err(DecompressorError::CodeLengthsChecksum)
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
							let new_repeat: usize = (4 * (last_repeat - 2)) + extra_bits + 3;

							if i + new_repeat - last_repeat > alphabet_size {
								return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths);
							}

							for _ in 0..new_repeat - last_repeat {
								actual_code_lengths[i] = last_non_zero_codelength as usize;
								i += 1;

								sum += 32768 >> last_non_zero_codelength;

								// debug(&format!("32768 >> code length == {:?}, sum == {:?}", 32768 >> last_non_zero_codelength, sum));
							}

							if sum == 32768 {
								break;
							} else if sum > 32768 {
								return Err(DecompressorError::CodeLengthsChecksum)
							}

							Some(new_repeat)
						},
						(_, _) => {
							let repeat = 3 + extra_bits;

							if i + repeat > alphabet_size {
								return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths);
							}

							for _ in 0..repeat {
								actual_code_lengths[i] = last_non_zero_codelength as usize;
								i += 1;

								sum += 32768 >> last_non_zero_codelength;

								// debug(&format!("32768 >> code length == {:?}, sum == {:?}", 32768 >> last_non_zero_codelength, sum));
							}

							if sum == 32768 {
								break;
							} else if sum > 32768 {
								return Err(DecompressorError::CodeLengthsChecksum)
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

					// debug(&format!("code length = 17, extra bits = {:?}", extra_bits));

					last_repeat = match (last_symbol, last_repeat) {
						(Some(17), Some(last_repeat)) => {
							let new_repeat = (8 * (last_repeat - 2)) + extra_bits as usize + 3;
							i += new_repeat - last_repeat;

							Some(new_repeat)
						},
						(_, _) => {
							let repeat = 3 + extra_bits as usize;
							i += repeat;

							Some(repeat)
						},
					};

					if i > alphabet_size {
						return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths);
					}

					last_symbol = Some(17);
				},
				Some(_) => unreachable!(), // confirmed unreachable, the possible symbols are defined in code above
				None => return Err(DecompressorError::ParseErrorComplexPrefixCodeLengths),
			};
		}

		// debug(&format!(""));

		// println!("Actual Code Lengths = {:?}", actual_code_lengths);

		if actual_code_lengths.iter().filter(|&l| *l > 0).collect::<Vec<_>>().len() < 2 {
			return Err(DecompressorError::LessThanTwoNonZeroCodeLengths);
		}

		Ok(huffman::codes_from_lengths(&actual_code_lengths))
	}

	fn parse_prefix_code(&mut self, alphabet_size: usize) -> Result<HuffmanCodes, DecompressorError> {
		let prefix_code_kind = match self.parse_prefix_code_kind() {
			Ok(kind) => kind,
			Err(e) => return Err(e),
		};

		// println!("Prefix Code Kind = {:?}", prefix_code_kind);

		match prefix_code_kind {
			PrefixCodeKind::Complex(h_skip) => self.parse_complex_prefix_code(h_skip, alphabet_size),
			PrefixCodeKind::Simple => self.parse_simple_prefix_code(alphabet_size),
		}
	}

	fn parse_prefix_code_block_types_literals(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = (self.meta_block.header.n_bltypes_l.unwrap() as usize) + 2;

		Ok(State::PrefixCodeBlockTypesLiterals(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_prefix_code_block_counts_literals(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = 26;

		Ok(State::PrefixCodeBlockCountsLiterals(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_prefix_code_block_types_insert_and_copy_lengths(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = (self.meta_block.header.n_bltypes_i.unwrap() as usize) + 2;

		Ok(State::PrefixCodeBlockTypesInsertAndCopyLengths(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_prefix_code_block_counts_insert_and_copy_lengths(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = 26;

		Ok(State::PrefixCodeBlockCountsInsertAndCopyLengths(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_prefix_code_block_types_distances(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = (self.meta_block.header.n_bltypes_d.unwrap() as usize) + 2;

		Ok(State::PrefixCodeBlockTypesDistances(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_prefix_code_block_counts_distances(&mut self) -> Result<State, DecompressorError> {
		let alphabet_size = 26;

		Ok(State::PrefixCodeBlockCountsDistances(
			match self.parse_prefix_code(alphabet_size) {
					Ok(prefix_code) => prefix_code,
					Err(e) => return Err(e),
			}
		))
	}

	fn parse_block_count(&mut self, prefix_code: &HuffmanCodes) -> Result<BLen, DecompressorError> {
		let symbol = prefix_code.lookup_symbol(&mut self.in_stream);

		// debug(&format!("block count symbol = {:?}", symbol));

		let (base_length, extra_bits) = match symbol {
			Ok(Some(symbol @  0... 3)) => (    1 + ((symbol as BLen)      <<  2),  2usize),
			Ok(Some(symbol @  4... 7)) => (   17 + ((symbol as BLen -  4) <<  3),  3),
			Ok(Some(symbol @  8...11)) => (   49 + ((symbol as BLen -  8) <<  4),  4),
			Ok(Some(symbol @ 12...15)) => (  113 + ((symbol as BLen - 12) <<  5),  5),
			Ok(Some(symbol @ 16...17)) => (  241 + ((symbol as BLen - 16) <<  6),  6),
			Ok(Some(18)) => (  369,  7),
			Ok(Some(19)) => (  497,  8),
			Ok(Some(20)) => (  753,  9),
			Ok(Some(21)) => ( 1265, 10),
			Ok(Some(22)) => ( 2289, 11),
			Ok(Some(23)) => ( 4337, 12),
			Ok(Some(24)) => ( 8433, 13),
			Ok(Some(25)) => (16625, 24),
			Ok(Some(_)) => return Err(DecompressorError::InvalidBlockCountCode),
			Ok(None) => return Err(DecompressorError::UnexpectedEOF),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		// debug(&format!("(base_length, extra_bits) = {:?}", (base_length, extra_bits)));

		match self.in_stream.read_u32_from_n_bits(extra_bits) {
			Ok(my_u32) => Ok(base_length + my_u32),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_first_block_count_literals(&mut self) -> Result<State, DecompressorError> {
		let prefix_code = self.meta_block.prefix_tree_block_counts_literals.as_ref().unwrap().clone();

		match self.parse_block_count(&prefix_code) {
			Ok(block_count) => Ok(State::FirstBlockCountLiterals(block_count)),
			Err(e) => Err(e),
		}
	}

	fn parse_first_block_count_insert_and_copy_lengths(&mut self) -> Result<State, DecompressorError> {
		let prefix_code = self.meta_block.prefix_tree_block_counts_insert_and_copy_lengths.as_ref().unwrap().clone();

		match self.parse_block_count(&prefix_code) {
			Ok(block_count) => Ok(State::FirstBlockCountInsertAndCopyLengths(block_count)),
			Err(e) => Err(e),
		}
	}

	fn parse_first_block_count_distances(&mut self) -> Result<State, DecompressorError> {
		let prefix_code = self.meta_block.prefix_tree_block_counts_distances.as_ref().unwrap().clone();

		match self.parse_block_count(&prefix_code) {
			Ok(block_count) => Ok(State::FirstBlockCountDistances(block_count)),
			Err(e) => Err(e),
		}
	}

	fn parse_prefix_codes_literals(&mut self) -> Result<State, DecompressorError> {
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

	fn parse_prefix_codes_insert_and_copy_lengths(&mut self) -> Result<State, DecompressorError> {
		let n_bltypes_i = self.meta_block.header.n_bltypes_i.unwrap() as usize;
		let mut prefix_codes = Vec::with_capacity(n_bltypes_i);
		let alphabet_size = 704;

		for _ in 0..n_bltypes_i {
			prefix_codes.push(match self.parse_prefix_code(alphabet_size) {
				Ok(prefix_code) => prefix_code,
				Err(e) => return Err(e),
			});
		}

		Ok(State::PrefixCodesInsertAndCopyLengths(prefix_codes))
	}

	fn parse_prefix_codes_distances(&mut self) -> Result<State, DecompressorError> {
		let n_trees_d = self.meta_block.header.n_trees_d.unwrap() as usize;
		let mut prefix_codes = Vec::with_capacity(n_trees_d);
		let alphabet_size = 16 + self.meta_block.header.n_direct.unwrap() as usize + (48 << self.meta_block.header.n_postfix.unwrap());

		// println!("NDIRECT = {:?}", self.meta_block.header.n_direct.unwrap());
		// println!("NPOSTFIX = {:?}", self.meta_block.header.n_postfix.unwrap());

		for _ in 0..n_trees_d {
			prefix_codes.push(match self.parse_prefix_code(alphabet_size) {
				Ok(prefix_code) => prefix_code,
				Err(e) => return Err(e),
			});
		}

		Ok(State::PrefixCodesDistances(prefix_codes))
	}

	fn parse_context_map(&mut self, n_trees: NTrees, len: usize) -> Result<ContextMap, DecompressorError> {
		let rlemax = match self.in_stream.read_bit() {
			Ok(false) => 0u16,
			Ok(true) => match self.in_stream.read_u16_from_n_bits(4) {
				Ok(my_u16) => my_u16 + 1,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			},
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		// debug(&format!("RLEMAX = {:?}", rlemax));

		let alphabet_size = (rlemax + n_trees) as usize;

		// debug(&format!("Alphabet Size = {:?}", alphabet_size));

		let prefix_tree = match self.parse_prefix_code(alphabet_size) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};

		// !("Prefix Tree Context Map = {:?}", prefix_tree);

		let mut c_map = Vec::with_capacity(len);
		let mut c_pushed = 0;

		while c_pushed < len {
			match prefix_tree.lookup_symbol(&mut self.in_stream) {
				Ok(Some(run_length_code)) if run_length_code > 0 && run_length_code <= rlemax => {
					// debug(&format!("run length code = {:?}", run_length_code));

					let repeat = match self.in_stream.read_u16_from_n_bits(run_length_code as usize) {
						Ok(my_u16) => (1u32 << run_length_code) + my_u16 as u32,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					// debug(&format!("repeat = {:?}", repeat));

					for _ in 0..repeat {
						c_map.push(0);
						c_pushed += 1;

						if c_pushed > len {
							return Err(DecompressorError::RunLengthExceededSizeOfContextMap);
						}
					}
				},
				Ok(Some(context_id)) => {
					c_map.push(if context_id == 0 { 0 } else { (context_id - rlemax) as u8 });

					// debug(&format!("context id == {:?}", if context_id == 0 { 0 } else { (context_id - rlemax) as u8 }));

					c_pushed += 1;
				},
				Ok(None) => return Err(DecompressorError::ParseErrorContextMap),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			}

			// debug(&format!("{:?}", (c_pushed, len)));
		}

		let imtf_bit = match self.in_stream.read_bit() {
			Ok(v) => v,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		// debug(&format!("IMTF BIT = {:?}", imtf_bit));

		if imtf_bit {

			Self::inverse_move_to_front_transform(&mut c_map);
		}

		Ok(c_map)
	}

	fn parse_context_map_literals(&mut self) -> Result<State, DecompressorError> {
		let n_trees = self.meta_block.header.n_trees_l.unwrap();
		let len = self.meta_block.header.n_bltypes_l.unwrap() as usize * 64;
		match self.parse_context_map(n_trees, len) {
			Ok(c_map_l) => Ok(State::ContextMapLiterals(c_map_l)),
			Err(e) => Err(e),
		}
	}

	fn parse_context_map_distances(&mut self) -> Result<State, DecompressorError> {
		let n_trees = self.meta_block.header.n_trees_d.unwrap();
		let len = (self.meta_block.header.n_bltypes_d.unwrap() * 4) as usize;
		match self.parse_context_map(n_trees, len) {
			Ok(c_map_d) => Ok(State::ContextMapDistances(c_map_d)),
			Err(e) => Err(e),
		}
	}

	fn inverse_move_to_front_transform(v: &mut[u8]) {
		let mut mtf: Vec<u8> = (0usize..256).map(|x| x as u8).collect();

		for mut item in v.iter_mut() {
			let index = *item as usize;
			let value = mtf[index];
			*item = value;

			for j in (1..index+1).rev() {
				mtf[j] = mtf[j - 1];
			}
			mtf[0] = value;
		}
	}

	fn parse_insert_and_copy_length(&mut self) -> Result<State, DecompressorError> {
		// debug(&format!("parse_insert_and_copy_length(): blen_i = {:?}", self.meta_block.blen_i));

		match self.meta_block.blen_i {
			None => {},
			Some(0) => {
				// debug(&format!("BLENI == 0, parsing switch command for insert and copy length"));

				match self.parse_block_switch_command_insert_and_copy_lengths() {
							Ok((block_type, block_count)) => {
								self.meta_block.btype_i_prev = self.meta_block.btype_i;
								self.meta_block.btype_i = block_type;

								self.meta_block.blen_i = Some(block_count - 1);
							},
							Err(e) => return Err(e),
						}},
			Some(ref mut blen_i) => *blen_i -= 1,
		};

		let btype = self.meta_block.btype_i as usize;

		// debug(&format!("btype_i = {:?}", btype));

		match self.meta_block.prefix_trees_insert_and_copy_lengths.as_ref().unwrap()[btype].lookup_symbol(&mut self.in_stream) {
			Ok(Some(symbol)) => Ok(State::InsertAndCopyLength(symbol)),
			Ok(None) => Err(DecompressorError::ParseErrorInsertAndCopyLength),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn decode_insert_and_copy_length(&mut self) -> Result<State, DecompressorError> {
		let ((mut insert_length, extra_bits_insert), (mut copy_length, extra_bits_copy)) = INSERT_LENGTHS_AND_COPY_LENGTHS[self.meta_block.insert_and_copy_length.unwrap() as usize];

		insert_length += match self.in_stream.read_u32_from_n_bits(extra_bits_insert) {
			Ok(my_u32) => my_u32,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		copy_length += match self.in_stream.read_u32_from_n_bits(extra_bits_copy) {
			Ok(my_u32) => my_u32,
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		Ok(State::InsertLengthAndCopyLength((insert_length, copy_length)))
	}

	fn parse_block_switch_command(&mut self, prefix_tree_types: HuffmanCodes, btype: NBltypes, btype_prev: NBltypes, n_bltypes: NBltypes, prefix_tree_counts: HuffmanCodes) -> Result<BlockSwitch, DecompressorError> {
		let block_type_code = match prefix_tree_types.lookup_symbol(&mut self.in_stream) {
			Ok(Some(block_type_code)) => block_type_code,
			Ok(None) => return Err(DecompressorError::InvalidBlockSwitchCommandCode),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		let block_type = match block_type_code {
			0 => btype_prev,
			1 => (btype + 1) % n_bltypes,
			2...258 => block_type_code - 2,
			_ => return Err(DecompressorError::InvalidBlockTypeCode),
		};

		if block_type >= n_bltypes {
			return Err(DecompressorError::InvalidBlockType);
		}

		// debug(&format!("block type = {:?}", block_type));

		let block_count = match self.parse_block_count(&prefix_tree_counts) {
			Ok(block_count) => block_count,
			Err(e) => return Err(e),
		};

		// debug(&format!("block count = {:?}", block_count));

		Ok((block_type, block_count))
	}

	fn parse_block_switch_command_literals(&mut self) -> Result<BlockSwitch, DecompressorError> {
		let prefix_tree_types = self.meta_block.prefix_tree_block_types_literals.as_ref().unwrap().clone();
		let btype = self.meta_block.btype_l;
		let btype_prev = self.meta_block.btype_l_prev;
		let n_bltypes = self.meta_block.header.n_bltypes_l.unwrap();

		let prefix_tree_counts = self.meta_block.prefix_tree_block_counts_literals.as_ref().unwrap().clone();

		self.parse_block_switch_command(prefix_tree_types, btype, btype_prev, n_bltypes, prefix_tree_counts)
	}

	fn parse_block_switch_command_insert_and_copy_lengths(&mut self) -> Result<BlockSwitch, DecompressorError> {
		// debug(&format!("Parsing block switch command insert and copy lengths"));
		let prefix_tree_types = self.meta_block.prefix_tree_block_types_insert_and_copy_lengths.as_ref().unwrap().clone();
		let btype = self.meta_block.btype_i;
		let btype_prev = self.meta_block.btype_i_prev;
		let n_bltypes = self.meta_block.header.n_bltypes_i.unwrap();

		let prefix_tree_counts = self.meta_block.prefix_tree_block_counts_insert_and_copy_lengths.as_ref().unwrap().clone();

		self.parse_block_switch_command(prefix_tree_types, btype, btype_prev, n_bltypes, prefix_tree_counts)
	}

	fn parse_block_switch_command_distances(&mut self) -> Result<BlockSwitch, DecompressorError> {
		let prefix_tree_types = self.meta_block.prefix_tree_block_types_distances.as_ref().unwrap().clone();
		let btype = self.meta_block.btype_d;
		let btype_prev = self.meta_block.btype_d_prev;
		let n_bltypes = self.meta_block.header.n_bltypes_d.unwrap();

		let prefix_tree_counts = self.meta_block.prefix_tree_block_counts_distances.as_ref().unwrap().clone();

		self.parse_block_switch_command(prefix_tree_types, btype, btype_prev, n_bltypes, prefix_tree_counts)
	}

	fn parse_insert_literals(&mut self) -> Result<State, DecompressorError> {

		let insert_length = self.meta_block.insert_length.unwrap() as usize;
		let mut literals = vec![0; insert_length];

		for mut lit in &mut literals {
			// debug(&format!("parse_insert_literals(): blen_l = {:?}", self.meta_block.blen_l));

			match self.meta_block.blen_l {
				None => {},
				Some(0) => match self.parse_block_switch_command_literals() {
					Ok((block_type, block_count)) => {
						self.meta_block.btype_l_prev = self.meta_block.btype_l;
						self.meta_block.btype_l = block_type;

						self.meta_block.blen_l = Some(block_count - 1);
					},
					Err(e) => return Err(e),
				},
				Some(ref mut blen_l) => *blen_l -= 1,
			};

			let btype = self.meta_block.btype_l as usize;

			// println!("btype = {:?}", btype);

			let context_mode = self.meta_block.context_modes_literals.as_ref().unwrap()[btype];

			// debug(&format!("[p1, p2] = {:?}", self.literal_buf));
			// debug(&format!("Context Mode = {:?}", context_mode));

			let cid = match context_mode {
				0 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;

					p1 & 0x3f
				},
				1 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;

					p1 >> 2
				},
				2 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;
					let p2 = *self.literal_buf.nth(1).unwrap() as usize;

					LUT_0[p1] | LUT_1[p2]
				},
				3 => {
					let p1 = *self.literal_buf.nth(0).unwrap() as usize;
					let p2 = *self.literal_buf.nth(1).unwrap() as usize;

					(LUT_2[p1] << 3) | LUT_2[p2]
				},
				_ => unreachable!(), // confirmed unreachable, context_mode is always read from two bits
			};

			// println!("(btype, cid) = {:?}", (btype, cid));

			let index = self.meta_block.header.c_map_l.as_ref().unwrap()[btype * 64 + cid] as usize;

			// debug(&format!("global bit pos = {:?}", self.in_stream.global_bit_pos));

			// println!("literal prefix code index = {:?}", index);



			*lit = match self.meta_block.prefix_trees_literals.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
				Ok(Some(symbol)) => symbol as Literal,
				Ok(None) => return Err(DecompressorError::ParseErrorInsertLiterals),
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			};

			// debug(&format!("Literal = {:?}", String::from_utf8(vec![lit])));

			self.literal_buf.push(*lit);
		}

		Ok(State::InsertLiterals(literals))
	}

	fn parse_distance_code(&mut self) -> Result<State, DecompressorError> {
		// debug(&format!("parse_distance_code(): blen_d = {:?}", self.meta_block.blen_d));

		// check for implicit distance 0 ([…]"as indicated by the insert-and-copy length code")
		match self.meta_block.distance {
			Some(0) => return Ok(State::DistanceCode(0)),
			Some(_) => unreachable!(), // confirmed unreachable, code sets meta_block.distance to None|Some(0) before this portion
			None => {}
		}

		match self.meta_block.blen_d {
			None => {},
			Some(0) => match self.parse_block_switch_command_distances() {
				Ok((block_type, block_count)) => {
					self.meta_block.btype_d_prev = self.meta_block.btype_d;
					self.meta_block.btype_d = block_type;

					self.meta_block.blen_d = Some(block_count - 1);
				},
				Err(e) => return Err(e),
			},
			Some(ref mut blen_d) => *blen_d -= 1,
		}

		let cid = match self.meta_block.copy_length {
			Some(0...1) => unreachable!(), // confirmed unreachable, copy_length will always be >= 2
			Some(c @ 2...4) => c - 2,
			Some(_) => 3,
			_ => unreachable!(), // confirmed unreachable, copy_length will always be set to Some(_) at this point
		};

		let index = self.meta_block.header.c_map_d.as_ref().unwrap()[self.meta_block.btype_d as usize * 4 + cid as usize] as usize;

		// debug(&format!("distance prefix code index = {:?}", index));
		// debug(&format!("distance prefix code = {:?}", self.meta_block.header.prefix_codes_distances.as_ref().unwrap()[index]));

		let distance_code = match self.meta_block.prefix_trees_distances.as_ref().unwrap()[index].lookup_symbol(&mut self.in_stream) {
			Ok(Some(symbol)) => symbol as DistanceCode,
			Ok(None) => return Err(DecompressorError::ParseErrorDistanceCode),
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		Ok(State::DistanceCode(distance_code))
	}

	fn decode_distance(&mut self) -> Result<State, DecompressorError> {
		let distance = match self.meta_block.distance_code {
			Some(d @ 0...3) => match self.distance_buf.nth(d as usize) {
				Ok(distance) => *distance,
				Err(_) => return Err(DecompressorError::RingBufferError),
			},
			Some(d @ 4...9) => {
				match (self.distance_buf.nth(0), 2 * (d as i64 % 2) - 1, (d - 2) >> 1) {
					(Ok(distance), sign, d) => match *distance as i64 + (sign * d as i64) {
						distance if distance <= 0 => return Err(DecompressorError::InvalidNonPositiveDistance),
						distance => distance as u32
					},
					(Err(_), _, _) => return Err(DecompressorError::RingBufferError),
				}
			},
			// reference distance_buf here, to get the decoded distance
			Some(d @ 10...15) => {
				match (self.distance_buf.nth(1), 2 * (d as i64 % 2) - 1, (d - 8) >> 1) {
					(Ok(distance), sign, d) => match *distance as i64 + (sign * d as i64) {
						distance if distance <= 0 => return Err(DecompressorError::InvalidNonPositiveDistance),
						distance => distance as u32
					},
					(Err(_), _, _) => return Err(DecompressorError::RingBufferError),
				}
			},
			Some(dcode) if dcode <= (15 + self.meta_block.header.n_direct.unwrap() as DistanceCode) => dcode - 15,
			Some(dcode) => {
				let (n_direct, n_postfix) = (self.meta_block.header.n_direct.unwrap() as DistanceCode, self.meta_block.header.n_postfix.unwrap());
				let ndistbits = 1 + ((dcode - (n_direct) - 16) >> (n_postfix + 1));

				// debug(&format!("NDISTBITS = {:?}", ndistbits));

				let dextra = match self.in_stream.read_u32_from_n_bits(ndistbits as usize) {
					Ok(my_u32) => my_u32,
					Err(_) => return Err(DecompressorError::UnexpectedEOF),
				};

				// debug(&format!("DEXTRA = {:?}", dextra));

				let hcode = (dcode - n_direct - 16) >> n_postfix;

				// debug(&format!("HCODE = {:?}", hcode));

				let postfix_mask = (1 << n_postfix) - 1;
				let lcode = (dcode - n_direct - 16) & postfix_mask;

				// debug(&format!("LCODE = {:?}", lcode));

				let offset = ((2 + (hcode & 1)) << ndistbits) - 4;

				// debug(&format!("Offset = {:?}", offset));

				//let distance =
				((offset + dextra) << n_postfix) + lcode + n_direct + 1

				// debug(&format!("Distance = {:?}", distance));

				//distance
			},
			None => unreachable!(), // confirmed unreachable, distance_code is always set to Some(_) at this point
		};

		// println!("(dc, db, d) = {:?}", (self.meta_block.distance_code, self.distance_buf.clone(), distance));

		if self.meta_block.distance_code.unwrap() > 0 && distance as usize <= cmp::min(self.header.window_size.unwrap(), self.count_output) {
			self.distance_buf.push(distance);
		}

		Ok(State::Distance(distance))
	}

	fn copy_literals(&mut self) -> Result<State, DecompressorError> {
		let window_size = self.header.window_size.unwrap();
		let copy_length = self.meta_block.copy_length.unwrap() as usize;
		let count_output = self.count_output;
		let distance = self.meta_block.distance.unwrap() as usize;
		let output_window = self.output_window.as_ref().unwrap();
		let max_allowed_distance = cmp::min(count_output, window_size);

		if distance <=  max_allowed_distance {
			let mut window = vec![0; copy_length];
			let l = cmp::min(distance, copy_length);

			match output_window.slice_tail(distance - 1, &mut window) {
				Ok(()) => {},
				Err(_) => return Err(DecompressorError::RingBufferError),
			}

			for i in l..copy_length {

				window[i] = window[i % l];
			}

			Ok(State::CopyLiterals(window))
		} else {
			if copy_length < 4 || copy_length > 24 {
				return Err(DecompressorError::InvalidLengthInStaticDictionary);
			}

			let word_id = distance - max_allowed_distance - 1;
			let n_words_length = if copy_length < 4 {
				0
			} else {
				1 << BROTLI_DICTIONARY_SIZE_BITS_BY_LENGTH[copy_length]
			};
			let index = word_id % n_words_length;
			let offset_from = BROTLI_DICTIONARY_OFFSETS_BY_LENGTH[copy_length] + index * copy_length;
			let offset_to = BROTLI_DICTIONARY_OFFSETS_BY_LENGTH[copy_length] + (index + 1) * copy_length;
			let base_word = &BROTLI_DICTIONARY[offset_from..offset_to];
			let transform_id = word_id >> BROTLI_DICTIONARY_SIZE_BITS_BY_LENGTH[copy_length];

			if transform_id > 120 {
				return Err(DecompressorError::InvalidTransformId);
			}

			// debug(&format!("base word = {:?}", String::from_utf8(Vec::from(base_word))));
			// debug(&format!("transform id = {:?}", transform_id));


			let transformed_word = transformation(transform_id, base_word);

			Ok(State::CopyLiterals(transformed_word))
		}

	}


	fn decompress(&mut self, buf: &mut [u8]) -> Result<usize, DecompressorError> {
		let mut buf_pos = 0;

		loop {
			match self.state.clone() {
				State::StreamBegin => {

					self.state = State::HeaderBegin;
				},
				State::HeaderBegin => {
					self.state = match self.parse_wbits() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::WBits(wbits) => {
					self.header.wbits = Some(wbits);
					self.header.window_size = Some((1 << wbits) - 16);
					self.output_window = Some(RingBuffer::with_capacity(self.header.window_size.unwrap()));

					// println!("(WBITS, Window Size) = {:?}", (wbits, self.header.window_size));

					self.state = State::HeaderEnd;
				},
				State::HeaderEnd => {
					self.state = State::HeaderMetaBlockBegin;
				},
				State::HeaderMetaBlockBegin => {
					self.meta_block.header = MetaBlockHeader::new();

					self.state = match self.parse_is_last() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::IsLast(true) => {
					self.meta_block.header.is_last = Some(true);

					// debug(&format!("ISLAST = true"));

					self.state = match self.parse_is_last_empty() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::IsLast(false) => {
					self.meta_block.header.is_last = Some(false);

					// debug(&format!("ISLAST = false"));

					self.state = match self.parse_m_nibbles() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::IsLastEmpty(true) => {
					self.meta_block.header.is_last_empty = Some(true);

					// debug(&format!("ISLASTEMPTY = true"));


					self.state = State::StreamEnd;
				},
				State::IsLastEmpty(false) => {
					self.meta_block.header.is_last_empty = Some(false);

					// debug(&format!("ISLASTEMPTY = false"));

					self.state = match self.parse_m_nibbles() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::MNibbles(0) => {
					match self.in_stream.read_bit() {
						Ok(true) => return Err(DecompressorError::NonZeroReservedBit),
						Ok(false) => {},
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}

					// debug(&format!("MNibbles = 0"));

					self.meta_block.header.m_nibbles = Some(0);

					self.state = match self.parse_m_skip_bytes() {
						Ok(state) => state,
						Err(e) => return Err(e),
					}
				},
				State::MNibbles(m_nibbles) => {
					self.meta_block.header.m_nibbles = Some(m_nibbles);

					// debug(&format!("MNibbles = {:?}", m_nibbles));

					self.state = match self.parse_m_len() {
						Ok(state) => state,
						Err(e) => return Err(e),
					}
				},
				State::MSkipBytes(0) => {
					self.meta_block.header.m_skip_bytes = Some(0);

					// debug(&format!("MSKIPBYTES = 0"));

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = State::MetaBlockEnd;
				},
				State::MSkipBytes(m_skip_bytes) => {
					self.meta_block.header.m_skip_bytes = Some(m_skip_bytes);

					// debug(&format!("MSKIPBYTES = {:?}", m_skip_bytes));

					self.state = match self.parse_m_skip_len() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				},
				State::MSkipLen(m_skip_len) => {
					self.meta_block.header.m_skip_len = Some(m_skip_len);

					// debug(&format!("MSKIPLEN = {:?}", m_skip_len));

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

					// println!("MLEN = {:?}", m_len);

					self.state = if self.meta_block.header.is_last.unwrap() {
						match self.parse_n_bltypes_l() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					} else {
						match self.parse_is_uncompressed() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::IsUncompressed(true) => {
					self.meta_block.header.is_uncompressed = Some(true);

					// println!("UNCOMPRESSED = true");

					match self.in_stream.read_u8_from_byte_tail() {
						Ok(0) => {},
						Ok(_) => return Err(DecompressorError::NonZeroFillBit),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};

					self.state = match self.parse_mlen_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::MLenLiterals(m_len_literals) => {
					for literal in &m_len_literals {
						if buf_pos < buf.len() {
							buf[buf_pos] = *literal;
							buf_pos += 1;
						} else {
							self.buf.push_front(*literal);
						}
						self.output_window.as_mut().unwrap().push(*literal);
						self.count_output += 1;
					}

					self.state = State::MetaBlockEnd;
					if buf_pos == buf.len() {
						return Ok(buf_pos);
					}
				},
				State::IsUncompressed(false) => {
					self.meta_block.header.is_uncompressed = Some(false);

					// println!("UNCOMPRESSED = false");

					self.state = match self.parse_n_bltypes_l() {
							Ok(state) => state,
							Err(e) => return Err(e),
					};
				},
				State::NBltypesL(n_bltypes_l) => {
					self.meta_block.header.n_bltypes_l = Some(n_bltypes_l);

					// println!("NBLTYPESL = {:?}", n_bltypes_l);

					self.state = if n_bltypes_l >= 2 {
						match self.parse_prefix_code_block_types_literals() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					} else {
						match self.parse_n_bltypes_i() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					}
				},
				State::PrefixCodeBlockTypesLiterals(prefix_tree) => {
					self.meta_block.prefix_tree_block_types_literals = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Types Literals = {:?}", self.meta_block.prefix_tree_block_types_literals));

					self.state = match self.parse_prefix_code_block_counts_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodeBlockCountsLiterals(prefix_tree) => {
					self.meta_block.prefix_tree_block_counts_literals = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Counts Literals = {:?}", self.meta_block.prefix_tree_block_counts_literals));

					self.state = match self.parse_first_block_count_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::FirstBlockCountLiterals(blen) => {
					self.meta_block.blen_l = Some(blen);

					// debug(&format!("Block count literals = {:?}", blen));

					self.state = match self.parse_n_bltypes_i() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NBltypesI(n_bltypes_i) => {
					self.meta_block.header.n_bltypes_i = Some(n_bltypes_i);

					// println!("NBLTYPESI = {:?}", n_bltypes_i);

					self.state = if n_bltypes_i >= 2 {
						match self.parse_prefix_code_block_types_insert_and_copy_lengths() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					} else {
						match self.parse_n_bltypes_d() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					}
				},
				State::PrefixCodeBlockTypesInsertAndCopyLengths(prefix_tree) => {
					self.meta_block.prefix_tree_block_types_insert_and_copy_lengths = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Types Insert And Copy Lengths = {:?}", self.meta_block.prefix_tree_block_types_insert_and_copy_lengths));

					self.state = match self.parse_prefix_code_block_counts_insert_and_copy_lengths() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodeBlockCountsInsertAndCopyLengths(prefix_tree) => {
					self.meta_block.prefix_tree_block_counts_insert_and_copy_lengths = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Counts Insert And Copy Lengths = {:?}", self.meta_block.prefix_tree_block_counts_insert_and_copy_lengths));

					self.state = match self.parse_first_block_count_insert_and_copy_lengths() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::FirstBlockCountInsertAndCopyLengths(blen) => {
					self.meta_block.blen_i = Some(blen);

					// debug(&format!("Block count insert and copy lengths = {:?}", blen));

					self.state = match self.parse_n_bltypes_d() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NBltypesD(n_bltypes_d) => {
					self.meta_block.header.n_bltypes_d = Some(n_bltypes_d);

					// println!("NBLTYPESD = {:?}", n_bltypes_d);

					self.state = if n_bltypes_d >= 2 {
						match self.parse_prefix_code_block_types_distances() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					} else {
						match self.parse_n_postfix() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					};
				},
				State::PrefixCodeBlockTypesDistances(prefix_tree) => {
					self.meta_block.prefix_tree_block_types_distances = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Types Distances = {:?}", self.meta_block.prefix_tree_block_types_distances));

					self.state = match self.parse_prefix_code_block_counts_distances() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodeBlockCountsDistances(prefix_tree) => {
					self.meta_block.prefix_tree_block_counts_distances = Some(prefix_tree);

					// debug(&format!("Prefix Tree Block Counts Distances = {:?}", self.meta_block.prefix_tree_block_counts_distances));

					self.state = match self.parse_first_block_count_distances() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::FirstBlockCountDistances(blen) => {
					self.meta_block.blen_d = Some(blen);

					// debug(&format!("Block count distances = {:?}", blen));

					self.state = match self.parse_n_postfix() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NPostfix(n_postfix) => {
					self.meta_block.header.n_postfix = Some(n_postfix);

					// debug(&format!("NPOSTFIX = {:?}", n_postfix));

					self.state = match self.parse_n_direct() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NDirect(n_direct) => {
					self.meta_block.header.n_direct = Some(n_direct);

					// debug(&format!("NDIRECT = {:?}", n_direct));

					self.state = match self.parse_context_modes_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::ContextModesLiterals(context_modes) => {
					self.meta_block.context_modes_literals = Some(context_modes);

					// println!("Context Modes Literals = {:?}", self.meta_block.context_modes_literals);

					self.state = match self.parse_n_trees_l() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NTreesL(n_trees_l) => {
					self.meta_block.header.n_trees_l = Some(n_trees_l);
					self.meta_block.header.c_map_l = Some(vec![0; 64 * self.meta_block.header.n_bltypes_l.unwrap() as usize]);

					// println!("NTREESL = {:?}", n_trees_l);

					self.state = if n_trees_l >= 2 {
						match self.parse_context_map_literals() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					} else {
						match self.parse_n_trees_d() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					};
				},
				State::ContextMapLiterals(c_map_l) => {
					self.meta_block.header.c_map_l = Some(c_map_l);

					// println!("CMAPL = {:?}", self.meta_block.header.c_map_l);

					self.state = match self.parse_n_trees_d() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::NTreesD(n_trees_d) => {
					self.meta_block.header.n_trees_d = Some(n_trees_d);
					self.meta_block.header.c_map_d = Some(vec![0; 4 * self.meta_block.header.n_bltypes_d.unwrap() as usize]);

					// println!("NTREESD = {:?}", n_trees_d);

					self.state = if n_trees_d >= 2 {
						match self.parse_context_map_distances() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					} else {
						match self.parse_prefix_codes_literals() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					};
				},
				State::ContextMapDistances(c_map_d) => {
					self.meta_block.header.c_map_d = Some(c_map_d);

					// debug(&format!("CMAPD = {:?}", self.meta_block.header.c_map_d));

					self.state = match self.parse_prefix_codes_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodesLiterals(prefix_trees) => {
					self.meta_block.prefix_trees_literals = Some(prefix_trees);

					// debug(&format!("Prefix Trees Literals = {:?}", self.meta_block.prefix_trees_literals));

					self.state = match self.parse_prefix_codes_insert_and_copy_lengths() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodesInsertAndCopyLengths(prefix_trees) => {
					self.meta_block.prefix_trees_insert_and_copy_lengths = Some(prefix_trees);

					// println!("Prefix Trees Insert And Copy Lengths = {:?}", self.meta_block.prefix_trees_insert_and_copy_lengths);

					self.state = match self.parse_prefix_codes_distances() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::PrefixCodesDistances(prefix_trees) => {
					self.meta_block.prefix_trees_distances = Some(prefix_trees);

					// debug(&format!("Prefix Trees Distances = {:?}", self.meta_block.prefix_trees_distances));

					self.state = State::DataMetaBlockBegin;
				},
				State::DataMetaBlockBegin => {
					self.state =  match self.parse_insert_and_copy_length() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::InsertAndCopyLength(insert_and_copy_length) => {
					self.meta_block.insert_and_copy_length = Some(insert_and_copy_length);

					self.meta_block.distance = match insert_and_copy_length {
						0...127 => Some(0),
						_ => None,
					};

					// println!("Insert And Copy Length = {:?}", insert_and_copy_length);

					self.state = match self.decode_insert_and_copy_length() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::InsertLengthAndCopyLength(insert_length_and_copy_length) => {
					let m_len = self.meta_block.header.m_len.unwrap() as usize;

					match insert_length_and_copy_length {
						(in_len, co_len) => {
							self.meta_block.insert_length = Some(in_len);
							self.meta_block.copy_length = Some(co_len);
						},
					};

					// println!("(m_len, insert_length, copy_length) = {:?}", (m_len, self.meta_block.insert_length.unwrap() as usize, self.meta_block.copy_length.unwrap() as usize));

					if (m_len < self.meta_block.count_output + self.meta_block.insert_length.unwrap() as usize) ||
					   (m_len > self.meta_block.count_output + self.meta_block.insert_length.unwrap() as usize &&
					    m_len < self.meta_block.count_output +
					            self.meta_block.insert_length.unwrap() as usize +
					            self.meta_block.copy_length.unwrap() as usize)
					{

						return Err(DecompressorError::ExceededExpectedBytes);
					}

					// println!("Insert Length and Copy Length = {:?}", insert_length_and_copy_length);

					self.state = match self.parse_insert_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::InsertLiterals(insert_literals) => {
					let m_len = self.meta_block.header.m_len.unwrap() as usize;

					// println!("MLEN = {:?}", m_len);
					if m_len < self.meta_block.count_output + insert_literals.len() {

						return Err(DecompressorError::ExceededExpectedBytes);
					}

					for literal in &insert_literals {
						if buf_pos < buf.len() {
							buf[buf_pos] = *literal;
							buf_pos += 1;
						} else {
							self.buf.push_front(*literal);
						}
						self.output_window.as_mut().unwrap().push(*literal);
						self.count_output += 1;
						self.meta_block.count_output += 1;
					}

					self.state = if self.meta_block.header.m_len.unwrap() as usize == self.meta_block.count_output {
						State::DataMetaBlockEnd
					} else {
						match self.parse_distance_code() {
							Ok(state) => state,
							Err(e) => return Err(e),
						}
					};

					if buf_pos == buf.len() {
						return Ok(buf_pos);
					}
				},
				State::DistanceCode(distance_code) => {
					self.meta_block.distance_code = Some(distance_code);

					// println!("Distance Code = {:?}", distance_code);

					self.state = match self.decode_distance() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::Distance(distance) => {
					self.meta_block.distance = Some(distance);

					// println!("Distance = {:?}", distance);

					self.state = match self.copy_literals() {
						Ok(state) => state,
						Err(e) => return Err(e),
					};
				},
				State::CopyLiterals(copy_literals) => {
					let m_len = self.meta_block.header.m_len.unwrap() as usize;

					if m_len < self.meta_block.count_output + copy_literals.len() {

						return Err(DecompressorError::ExceededExpectedBytes);
					}

					for literal in &copy_literals {
						if buf_pos < buf.len() {
							buf[buf_pos] = *literal;
							buf_pos += 1;
						} else {
							self.buf.push_front(*literal);
						}
						self.literal_buf.push(*literal);

						// debug(&format!("copy literal = {:?}", String::from_utf8(vec![literal])));

						self.output_window.as_mut().unwrap().push(*literal);
						self.count_output += 1;
						self.meta_block.count_output += 1;
					}

					// debug(&format!("output = {:?}", self.buf));

					self.state = if self.meta_block.header.m_len.unwrap() as usize == self.meta_block.count_output {

						State::DataMetaBlockEnd
					} else {

						State::DataMetaBlockBegin
					};

					// debug(&format!("output so far = {}", String::from_utf8(self.output_window.unwrap().clone().iter().filter(|&b| *b > 0).map(|b| *b).collect::<Vec<_>>()).unwrap()));

					if buf_pos == buf.len() {
						return Ok(buf_pos);
					}
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
						Err(BitReaderError::EOF) => return Ok(buf_pos),
						Ok(_) => return Err(DecompressorError::ExpectedEndOfStream),
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					}
				}
			};
		}
	}
}

impl<R: Read> Read for Decompressor<R> {
	fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
		if self.buf.is_empty() {
			match self.decompress(&mut buf) {
				Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.description())),
				Ok(l) => {
					Ok(l)
				},
			}
		} else {
			let l = cmp::min(self.buf.len(), buf.len());

			for i in 0..l {
				buf[i] = self.buf.pop_back().unwrap();
			}

			Ok(l)
		}

	}
}

