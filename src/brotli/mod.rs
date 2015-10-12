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
type ContextMode = u8;
type ContextModes = Vec<ContextMode>;
type ContextMap = Vec<ContextMode>;
type NTreesL = u8;
type NTreesD = u8;
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
	count_output: usize,
	context_modes_literals: Option<ContextModes>,
	prefix_tree_literals: Option<HuffmanCodes>,
	prefix_tree_insert_and_copy_lengths: Option<HuffmanCodes>,
	prefix_tree_distances: Option<HuffmanCodes>,
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
			prefix_tree_literals: None,
			prefix_tree_insert_and_copy_lengths: None,
			prefix_tree_distances: None,
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
	n_trees_l: Option<NTreesL>,
	n_trees_d: Option<NTreesD>,
	c_map_d: Option<ContextMap>,
	prefix_code_literals: Option<PrefixCode>,
	prefix_code_insert_and_copy_lengths: Option<PrefixCode>,
	prefix_code_distances: Option<PrefixCode>,
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
			prefix_code_literals: None,
			prefix_code_insert_and_copy_lengths: None,
			prefix_code_distances: None,
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
	PrefixCodeLiterals((PrefixCode, HuffmanCodes)),
	PrefixCodeInsertAndCopyLengths((PrefixCode, HuffmanCodes)),
	PrefixCodeDistances((PrefixCode, HuffmanCodes)),
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
	ExceededExpectedBytes,
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
			&DecompressorError::ExceededExpectedBytes => "More uncompressed bytes than expected in meta-block",
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
	/// most recent byte in output stream
	/// used to determine context mode for literals
	p1: u8,
	/// second most recent byte in output stream
	/// used to determine context mode for literals
	p2: u8,
	/// ring buffer for last 4 distances, gets set
	/// at the beginning of the stream, and then
	/// lives until the end
	distance_buf: Vec<Distance>,
	distance_buf_pos: usize,
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
			p1: 0,
			p2: 0,
			distance_buf: vec![4, 11, 15, 16],
			distance_buf_pos: 0,
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

	fn parse_prefix_code(&mut self, alphabet_size: usize) -> result::Result<(PrefixCode, HuffmanCodes), DecompressorError> {
		let bit_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as usize;

		println!("Bit Width = {:?}", bit_width);

		let prefix_code_kind = match self.parse_prefix_code_kind() {
			Ok(kind) => kind,
			Err(e) => return Err(e),
		};

		if prefix_code_kind == PrefixCodeKind::Complex {
			unimplemented!();
		}

		println!("Prefix Code Kind = {:?}", prefix_code_kind);

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
			(2, None) => vec![1, 1],
			(3, None) => vec![1, 2, 2],
			(4, Some(false)) => vec![2, 2, 2, 2],
			(4, Some(true)) => vec![1, 2, 3, 3],
			_ => unreachable!(),
		};

		println!("Code Lengths = {:?}", code_lengths);

		Ok((PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(n_sym as u8),
				symbols: Some(symbols.clone()),
				tree_select: tree_select,
			}),
            huffman::codes_from_lengths_and_symbols(code_lengths, &symbols)))
	}

	fn parse_prefix_code_literals(&mut self) -> result::Result<State, DecompressorError> {
		let alphabet_size = 256;

		match self.parse_prefix_code(alphabet_size) {
			Ok(prefix_code) => Ok(State::PrefixCodeLiterals(prefix_code)),
			Err(e) => Err(e),
		}
	}

	fn parse_prefix_code_insert_and_copy_lengths(&mut self) -> result::Result<State, DecompressorError> {
		let alphabet_size = 704;

		match self.parse_prefix_code(alphabet_size) {
			Ok(prefix_code) => Ok(State::PrefixCodeInsertAndCopyLengths(prefix_code)),
			Err(e) => Err(e),
		}
	}

	fn parse_prefix_code_distances(&mut self) -> result::Result<State, DecompressorError> {
		let alphabet_size = 16 + self.meta_block.header.n_direct.unwrap() as usize + 48 << self.meta_block.header.n_postfix.unwrap() as usize;

		match self.parse_prefix_code(alphabet_size) {
			Ok(prefix_code) => Ok(State::PrefixCodeDistances(prefix_code)),
			Err(e) => Err(e),
		}
	}

	fn parse_context_map_distances(&mut self) -> result::Result<State, DecompressorError> {
		let rlemax = match self.in_stream.read_bit() {
			Ok(false) => 0,
			Ok(true) => match self.in_stream.read_u8_from_n_bits(4) {
				Ok(my_u8) => my_u8 + 1,
				Err(_) => return Err(DecompressorError::UnexpectedEOF),
			},
			Err(_) => return Err(DecompressorError::UnexpectedEOF),
		};

		println!("RLEMAX = {:?}", rlemax);

		let alphabet_size = (rlemax + self.meta_block.header.n_trees_d.unwrap()) as usize;

		println!("Alphabet Size = {:?}", alphabet_size);


		unimplemented!();
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
			})) => {
				match self.meta_block.prefix_tree_insert_and_copy_lengths.as_ref().unwrap().lookup_symbol(&mut self.in_stream) {
					Some(symbol) => Ok(State::InsertAndCopyLength(symbol)),
					None => Err(DecompressorError::ParseErrorInsertAndCopyLength),
				}
			},
			_ => unimplemented!()
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
				Some(0) => unimplemented!(),
				Some(ref mut blen_l) => *blen_l -= 1,
			}

			literals[i] = match self.meta_block.header.prefix_code_literals {
				Some(PrefixCode::Simple(PrefixCodeSimple {
					n_sym: Some(1),
					symbols: Some(ref symbols),
					tree_select: _,
				})) => symbols[0] as Literal,
				Some(PrefixCode::Simple(PrefixCodeSimple {
					n_sym: Some(2...4),
					symbols: _,
					tree_select: _,
				})) => {
					match self.meta_block.prefix_tree_literals.as_ref().unwrap().lookup_symbol(&mut self.in_stream) {
						Some(symbol) => symbol as Literal,
						None => return Err(DecompressorError::ParseErrorInsertLiterals),
					}
				},
				_ => unimplemented!(),
			}
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
			Some(0) => unimplemented!(),
			Some(ref mut blen_d) => *blen_d -= 1,
		}

		let distance_code = match self.meta_block.header.prefix_code_distances {
			Some(PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(1),
				symbols: Some(ref symbols),
				tree_select: _,
			})) => symbols[0] as DistanceCode,
			Some(PrefixCode::Simple(PrefixCodeSimple {
				n_sym: Some(2...4),
				symbols: _,
				tree_select: _,
			})) => {
				unimplemented!();
				// @NOTE Need to create prefix tree for distances first
				// match self.meta_block.prefix_tree_distances.as_ref().unwrap().lookup_symbol(&mut self.in_stream) {
				// 	Some(symbol) => symbol as DistanceCode,
				// 	None => return Err(DecompressorError::ParseErrorInsertLiterals),
				// }
			},
			_ => unimplemented!(),
		};

		Ok(State::DistanceCode(distance_code))
	}

	fn decode_distance(&mut self) -> result::Result<State, DecompressorError> {
		let distance = match self.meta_block.distance_code {
			Some(0) => {
				self.distance_buf[self.distance_buf_pos]
			},
			// reference distance_buf here, to get the decoded distance
			Some(1...15) => unimplemented!(),
			Some(dcode) if dcode <= (15 + self.meta_block.header.n_direct.unwrap() as DistanceCode) => dcode - 15,
			// use NDIRECT and NPOSTFIX calculations, as described in the RFC, section 4.
			// to calculate the decoded distance
			Some(dcode) => {
				let (n_direct, n_postfix) = (self.meta_block.header.n_direct.unwrap() as DistanceCode, self.meta_block.header.n_postfix.unwrap());
				let ndistbits = 1 + ((dcode - (n_direct) - 16) >> (n_postfix + 1));

				println!("NDISTBITS = {:?}", ndistbits);

				let dextra = match self.in_stream.read_bit() {
					Ok(true) => 1,
					Ok(false) => 0,
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

		if self.meta_block.distance_code.unwrap() > 0 {
			self.distance_buf_pos = (self.distance_buf_pos + 3) % 4;
			self.distance_buf[self.distance_buf_pos] = distance;
		}

		Ok(State::Distance(distance))
	}

	fn copy_literals(&mut self) -> result::Result<State, DecompressorError> {
		let window_size = self.header.window_size.unwrap();
		let copy_length = self.meta_block.copy_length.unwrap() as usize;
		let count_output = self.count_output;
		let distance = self.meta_block.distance.unwrap() as usize;
		let ref output_window = self.output_window;

		let mut window = vec![0; copy_length];
		let l = if copy_length > distance {
			distance
		} else {
			copy_length
		};

		for i in (count_output + window_size - distance)..(count_output + window_size - distance + l) {

			window[i - (count_output + window_size - distance)] = output_window[i % window_size];
		}

		for i in l..copy_length {
			window[i] = window[i % l];
		}

		println!("window = {:?}", window);

		Ok(State::CopyLiterals(window))
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
					self.meta_block.btype_l = Some(0);

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
					self.meta_block.btype_i = Some(0);

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
					self.meta_block.btype_d = Some(0);

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
					self.meta_block.context_modes_literals = Some(context_modes);

					println!("Context Modes Literals = {:?}", self.meta_block.context_modes_literals);

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
					self.meta_block.header.c_map_d = Some(vec![0; self.meta_block.header.n_bltypes_d.unwrap() as usize]);

					println!("NTREESD = {:?}", n_trees_d);

					self.state = if n_trees_d >= 2 {
						match self.parse_context_map_distances() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					} else {
						match self.parse_prefix_code_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						}
					};
				},
				State::PrefixCodeLiterals((prefix_code, prefix_tree)) => {
					self.meta_block.header.prefix_code_literals = Some(prefix_code);
					self.meta_block.prefix_tree_literals = Some(prefix_tree);

					println!("Prefix Code Literals = {:?}", self.meta_block.header.prefix_code_literals);
					println!("Prefix Tree literals = {:?}", self.meta_block.prefix_tree_literals);

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

					self.state = match self.parse_prefix_code_distances() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::PrefixCodeDistances((prefix_code, prefix_tree)) => {
					self.meta_block.header.prefix_code_distances = Some(prefix_code);
					self.meta_block.prefix_tree_distances = Some(prefix_tree);

					println!("Prefix Code Distances = {:?}", self.meta_block.header.prefix_code_distances);
					println!("Prefix Tree Distances = {:?}", self.meta_block.prefix_tree_distances);

					self.state = State::DataMetaBlockBegin;
				},
				State::DataMetaBlockBegin => {
					self.state = match (self.meta_block.header.n_bltypes_i, self.meta_block.blen_i) {
						(Some(1), _) => match self.parse_insert_and_copy_length() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
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
						(Some(1), _) => match self.parse_insert_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						// should parse block type code for literals here
						(_, Some(0)) => unimplemented!(),
						(Some(_), Some(_)) =>  match self.parse_insert_literals() {
							Ok(state) => state,
							Err(_) => return Err(DecompressorError::UnexpectedEOF),
						},
						_ => unreachable!(),
					};
				},
				State::InsertLiterals(insert_literals) => {
					for literal in insert_literals {
						self.buf.push_front(literal);
						self.p2 = self.p1;
						self.p1 = literal;
						self.output_window[self.count_output % self.header.window_size.unwrap() as usize] = literal;
						self.count_output += 1;
						self.meta_block.count_output += 1;
					}

					println!("output = {:?}", self.buf);

					self.state = if self.meta_block.header.m_len.unwrap() as usize == self.meta_block.count_output {
						State::DataMetaBlockEnd
					} else {
						match (self.meta_block.header.n_bltypes_d, self.meta_block.blen_d) {
							(Some(0), _) => unreachable!(),
							(Some(1), _) => match self.parse_distance_code() {
								Ok(state) => state,
								Err(_) => return Err(DecompressorError::UnexpectedEOF),
							},
							// should parse block type code for distances here
							(_, Some(0)) => unimplemented!(),
							(Some(_), Some(_)) =>  match self.parse_distance_code() {
								Ok(state) => state,
								Err(_) => return Err(DecompressorError::UnexpectedEOF),
							},
							_ => unreachable!(),
						}
					};

					return Ok(self.buf.len());
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

					if (distance as usize) > self.header.window_size.unwrap() || (distance as usize) > self.count_output {
						// need to read from static dictionary
						// and do transformations
						unimplemented!();
					}

					self.state = match self.copy_literals() {
						Ok(state) => state,
						Err(_) => return Err(DecompressorError::UnexpectedEOF),
					};
				},
				State::CopyLiterals(copy_literals) => {
					for literal in copy_literals {
						self.buf.push_front(literal);
						self.p2 = self.p1;
						self.p1 = literal;
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
